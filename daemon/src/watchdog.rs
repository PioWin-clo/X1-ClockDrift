use std::env;
use std::time::Duration;
use tokio::net::UnixDatagram;
use tokio_util::sync::CancellationToken;

pub async fn run(watchdog_secs: u64, shutdown: CancellationToken) -> anyhow::Result<()> {
    let socket_path = match env::var("NOTIFY_SOCKET") {
        Ok(p) => p,
        Err(_) => {
            tracing::warn!("NOTIFY_SOCKET not set — outside systemd, watchdog disabled");
            return Ok(());
        }
    };

    let sock = UnixDatagram::unbound()?;
    sock.send_to(b"READY=1", &socket_path).await?;
    tracing::info!(socket = %socket_path, "sd_notify READY=1");

    let interval = Duration::from_secs((watchdog_secs / 3).max(10));
    loop {
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => {
                let _ = sock.send_to(b"STOPPING=1", &socket_path).await;
                tracing::info!("watchdog shutting down (sent STOPPING=1)");
                return Ok(());
            }
            _ = tokio::time::sleep(interval) => {
                if let Err(e) = sock.send_to(b"WATCHDOG=1", &socket_path).await {
                    tracing::error!(error = %e, "watchdog ping failed");
                }
            }
        }
    }
}
