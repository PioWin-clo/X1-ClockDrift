use std::env;
use std::time::Duration;
use tokio::net::UnixDatagram;

pub async fn run(watchdog_secs: u64) -> anyhow::Result<()> {
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
        tokio::time::sleep(interval).await;
        if let Err(e) = sock.send_to(b"WATCHDOG=1", &socket_path).await {
            tracing::error!(error = %e, "watchdog ping failed");
        }
    }
}
