use crate::db::{self, Pool, VoteRecord};
use crate::rpc_client::{RpcClient, RpcError};
use crate::vote_parser;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Semaphore;

const MAX_INFLIGHT: usize = 16;

pub async fn run(pool: Pool, rpc: Arc<RpcClient>, mut rx: Receiver<(u64, i64)>) {
    let limiter = Arc::new(Semaphore::new(MAX_INFLIGHT));
    tracing::info!("vote_collector starting");

    while let Some((slot, ts_us)) = rx.recv().await {
        if let Err(e) = db::record_slot_obs(&pool, slot, ts_us).await {
            tracing::warn!(error = %e, slot, "record_slot_obs failed");
            let _ = db::record_error(&pool, "vote_collector", &format!("slot_obs: {e}")).await;
            continue;
        }

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
        for v in vote_parser::extract_votes_from_tx(tx) {
            if let Some(ts) = v.ts_chain {
                all.push(VoteRecord {
                    validator: v.validator,
                    slot_voted: v.slot_voted,
                    ts_chain: ts,
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
