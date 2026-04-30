use std::path::Path;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Polls the kill-switch file every 5 seconds. When detected, cancels
/// the shared shutdown token so `main()` runs the same drain logic as
/// for ctrl-c / SIGTERM. Pre-v0.4.1 this called `process::exit(0)`,
/// which bypassed the drain and could interrupt a git push mid-stream.
pub async fn run(kill_switch_path: String, shutdown: CancellationToken) {
    tracing::info!(path = %kill_switch_path, "kill_switch_watcher starting");
    loop {
        if Path::new(&kill_switch_path).exists() {
            tracing::warn!(
                path = %kill_switch_path,
                "STOP file detected, triggering graceful shutdown"
            );
            shutdown.cancel();
            return;
        }
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => return,
            _ = tokio::time::sleep(Duration::from_secs(5)) => {}
        }
    }
}
