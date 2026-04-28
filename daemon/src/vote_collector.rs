use crate::db::{self, Pool, VoteRecord};
use crate::rpc_client::{RpcClient, RpcError};
use crate::vote_parser;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Semaphore;

const MAX_INFLIGHT: usize = 16;

/// Sample one block per this many slots. Public RPC is the only viable
/// source since the local validator does not expose `--full-rpc-api`,
/// so we keep our load light: ~600 getBlock calls/day at 500 slots.
const BLOCK_SAMPLE_INTERVAL_SLOTS: u64 = 500;

pub async fn run(pool: Pool, rpc: Arc<RpcClient>, mut rx: Receiver<(u64, i64)>) {
    let limiter = Arc::new(Semaphore::new(MAX_INFLIGHT));
    let mut last_sampled_slot: u64 = 0;
    tracing::info!(
        sample_interval_slots = BLOCK_SAMPLE_INTERVAL_SLOTS,
        "vote_collector starting"
    );

    while let Some((slot, ts_us)) = rx.recv().await {
        if let Err(e) = db::record_slot_obs(&pool, slot, ts_us).await {
            tracing::warn!(error = %e, slot, "record_slot_obs failed");
            let _ = db::record_error(&pool, "vote_collector", &format!("slot_obs: {e}")).await;
            continue;
        }

        if slot < last_sampled_slot.saturating_add(BLOCK_SAMPLE_INTERVAL_SLOTS) {
            continue;
        }
        last_sampled_slot = slot;

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
            if let Err(e) = fetch_and_record(slot, &rpc2, &pool2).await {
                tracing::warn!(error = %e, slot, "vote collection failed");
                let _ = db::record_error(&pool2, "vote_collector", &format!("slot {slot}: {e}"))
                    .await;
            }
        });
    }
    tracing::warn!("vote_collector input channel closed");
}

async fn fetch_and_record(slot: u64, rpc: &RpcClient, pool: &Pool) -> Result<(), RpcError> {
    let block = match rpc.get_block_with_votes(slot).await? {
        Some(b) => b,
        None => return Ok(()),
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

    if !all.is_empty() {
        match db::record_votes(pool, &all, slot).await {
            Ok(n) => {
                tracing::debug!(slot, votes = n, "stored votes");
            }
            Err(e) => {
                return Err(RpcError::Decode(format!("db record_votes: {e}")));
            }
        }
    }
    Ok(())
}
