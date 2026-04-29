use crate::db::{self, Pool, VoteRecord};
use crate::rpc_client::{RpcClient, RpcError};
use crate::vote_parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

const MAX_INFLIGHT: usize = 16;

/// Sample one block per this many slots. Public RPC is the only viable
/// source (local validator has no `--full-rpc-api`), so we keep load
/// light: ~600 getBlock calls/day at 500-slot spacing.
const BLOCK_SAMPLE_INTERVAL_SLOTS: u64 = 500;

/// How often we ask RPC for the current slot. With 500-slot sampling this
/// only matters as a heartbeat — actual sample rate is ~3 minutes.
const POLL_INTERVAL_SECS: u64 = 30;

pub async fn run(pool: Pool, rpc: Arc<RpcClient>) {
    let limiter = Arc::new(Semaphore::new(MAX_INFLIGHT));
    let mut last_sampled_slot: u64 = 0;
    tracing::info!(
        sample_interval_slots = BLOCK_SAMPLE_INTERVAL_SLOTS,
        poll_interval_secs = POLL_INTERVAL_SECS,
        "vote_collector starting (RPC poll mode)"
    );

    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

        let current_slot = match rpc.get_slot().await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "get_slot failed");
                let _ = db::record_error(
                    &pool,
                    "vote_collector",
                    &format!("get_slot: {e}"),
                )
                .await;
                continue;
            }
        };

        // First successful poll: anchor near the chain tip so we don't
        // back-sample old slots that we have no local-clock observation
        // for. We start emitting samples on the next interval.
        if last_sampled_slot == 0 {
            last_sampled_slot = current_slot;
            tracing::info!(anchor_slot = current_slot, "vote_collector anchored at chain tip");
            continue;
        }

        while current_slot >= last_sampled_slot.saturating_add(BLOCK_SAMPLE_INTERVAL_SLOTS) {
            let target = last_sampled_slot.saturating_add(BLOCK_SAMPLE_INTERVAL_SLOTS);
            last_sampled_slot = target;

            let permit = match limiter.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    tracing::error!("vote_collector limiter closed");
                    return;
                }
            };
            let pool2 = pool.clone();
            let rpc2 = rpc.clone();
            tokio::spawn(async move {
                let _p = permit;
                if let Err(e) = fetch_and_record(target, &rpc2, &pool2).await {
                    tracing::warn!(error = %e, slot = target, "vote collection failed");
                    let _ = db::record_error(
                        &pool2,
                        "vote_collector",
                        &format!("slot {target}: {e}"),
                    )
                    .await;
                }
            });
        }
    }
}

async fn fetch_and_record(slot: u64, rpc: &RpcClient, pool: &Pool) -> Result<(), RpcError> {
    let block = match rpc.get_block_with_votes(slot).await? {
        Some(b) => b,
        None => {
            tracing::info!(slot, "block unavailable (skipped or not yet finalized)");
            return Ok(());
        }
    };

    let mut all = Vec::new();
    for tx in &block.transactions {
        for ix in &tx.instructions {
            if let Some(parsed) = vote_parser::parse_instruction(ix) {
                all.push(VoteRecord {
                    validator: parsed.vote_account,
                    slot_voted: parsed.last_voted_slot,
                    ts_chain: parsed.timestamp,
                });
            }
        }
    }

    let n = all.len();
    if all.is_empty() {
        tracing::info!(slot, "block had no parseable vote instructions");
        return Ok(());
    }
    match db::record_votes(pool, &all, slot).await {
        Ok(inserted) => {
            tracing::info!(slot, votes_parsed = n, votes_inserted = inserted, "stored votes");
            Ok(())
        }
        Err(e) => Err(RpcError::Decode(format!("db record_votes: {e}"))),
    }
}
