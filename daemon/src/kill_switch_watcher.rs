use std::path::Path;
use std::time::Duration;

pub async fn run(kill_switch_path: String) {
    tracing::info!(path = %kill_switch_path, "kill_switch_watcher starting");
    loop {
        if Path::new(&kill_switch_path).exists() {
            tracing::warn!(path = %kill_switch_path, "STOP file detected, exiting");
            std::process::exit(0);
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
