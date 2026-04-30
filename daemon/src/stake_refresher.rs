use crate::db::{self, Pool};
use crate::rpc_client::RpcClient;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn run(
    pool: Pool,
    rpc: Arc<RpcClient>,
    refresh_secs: u64,
    shutdown: CancellationToken,
) {
    tracing::info!(refresh_secs, "stake_refresher starting");
    loop {
        if let Err(e) = refresh_once(&pool, &rpc).await {
            tracing::warn!(error = %e, "stake refresh failed");
            let _ = db::record_error(&pool, "stake_refresher", &e.to_string()).await;
        }
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => {
                tracing::info!("stake_refresher shutting down");
                return;
            }
            _ = tokio::time::sleep(Duration::from_secs(refresh_secs)) => {}
        }
    }
}

async fn refresh_once(pool: &Pool, rpc: &RpcClient) -> anyhow::Result<()> {
    let accounts = rpc.get_vote_accounts().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let now = chrono::Utc::now().timestamp();
    let entries: Vec<(String, i64)> = accounts
        .into_iter()
        .map(|a| (a.vote_pubkey, a.activated_stake))
        .collect();
    let count = entries.len();
    db::record_stake_snapshot(pool, now, &entries).await?;
    tracing::info!(count, "stake snapshot recorded");
    Ok(())
}
