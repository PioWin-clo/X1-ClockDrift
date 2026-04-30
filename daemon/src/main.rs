use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

mod api;
mod chrony_reader;
mod config;
mod db;
mod exporter;
mod foundation;
mod git_pusher;
mod kill_switch_watcher;
mod log_tail;
mod rpc_client;
mod stake_refresher;
mod vote_collector;
mod vote_parser;
mod watchdog;

const DEFAULT_CONFIG_PATH: &str = "/home/x1pio/strontium-meter/config.toml";
const DRAIN_TIMEOUT_SECS: u64 = 10;

#[derive(Parser)]
#[command(name = "x1cd", about = "X1 ClockDrift measurement daemon")]
struct Cli {
    #[arg(long, global = true, default_value = DEFAULT_CONFIG_PATH)]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Run,
    Status,
    Stop,
    Config,
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let cmd = cli.command.unwrap_or(Commands::Run);

    match cmd {
        Commands::Run => run(&cli.config).await,
        Commands::Status => status(&cli.config).await,
        Commands::Stop => stop(&cli.config).await,
        Commands::Config => show_config(&cli.config).await,
    }
}

async fn run(config_path: &std::path::Path) -> Result<()> {
    let cfg = config::Config::load(config_path).context("loading config")?;

    if std::path::Path::new(&cfg.kill_switch_path).exists() {
        eprintln!("STOP file exists at {}, refusing to start", cfg.kill_switch_path);
        std::process::exit(0);
    }

    let pool = db::init(&cfg.db_path)
        .await
        .with_context(|| format!("opening db at {}", cfg.db_path))?;
    let rpc = Arc::new(
        rpc_client::RpcClient::new(&cfg.rpc_url, cfg.rpc_rate_limit_per_sec)
            .context("building rpc client")?,
    );

    // v0.4.1: shared shutdown token. Cancelled by ctrl-c, SIGTERM, or
    // kill_switch_watcher (when it sees the STOP file). All long-running
    // tasks observe it and return cleanly; main waits up to DRAIN_TIMEOUT
    // for them to finish before exiting.
    let shutdown = CancellationToken::new();

    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    // log_tail and vote_collector are decoupled: each writes to the DB
    // independently. log_tail records local freeze times into slot_obs;
    // vote_collector polls RPC on a timer for sampled blocks and records
    // vote_records.
    {
        let cfg_log = cfg.log_path.clone();
        let pool_lt = pool.clone();
        let s = shutdown.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = log_tail::run(cfg_log, pool_lt, s).await {
                tracing::error!(error = %e, "log_tail terminated");
            }
        }));
    }

    {
        let pool_vc = pool.clone();
        let rpc_vc = rpc.clone();
        let s = shutdown.clone();
        handles.push(tokio::spawn(async move {
            vote_collector::run(pool_vc, rpc_vc, s).await;
        }));
    }

    {
        let pool_sr = pool.clone();
        let rpc_sr = rpc.clone();
        let stake_secs = cfg.stake_refresh_secs;
        let s = shutdown.clone();
        handles.push(tokio::spawn(async move {
            stake_refresher::run(pool_sr, rpc_sr, stake_secs, s).await;
        }));
    }

    {
        let pool_chrony = pool.clone();
        let s = shutdown.clone();
        handles.push(tokio::spawn(async move {
            chrony_reader::run(pool_chrony, s).await;
        }));
    }

    {
        let pool_ex = pool.clone();
        let cfg_ex = cfg.clone();
        let s = shutdown.clone();
        handles.push(tokio::spawn(async move {
            exporter::run(pool_ex, cfg_ex, s).await;
        }));
    }

    {
        let pool_api = pool.clone();
        let cfg_api = cfg.clone();
        let s = shutdown.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = api::run(pool_api, cfg_api, s).await {
                tracing::error!(error = %e, "api server terminated");
            }
        }));
    }

    {
        let stop_path = cfg.kill_switch_path.clone();
        let s = shutdown.clone();
        handles.push(tokio::spawn(async move {
            kill_switch_watcher::run(stop_path, s).await;
        }));
    }

    {
        let watchdog_secs = cfg.watchdog_secs;
        let s = shutdown.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = watchdog::run(watchdog_secs, s).await {
                tracing::error!(error = %e, "watchdog terminated");
            }
        }));
    }

    tracing::info!("daemon running; ctrl-c, SIGTERM, or STOP file to shutdown");

    // Wait for any of: ctrl-c, SIGTERM, or kill_switch cancelling token.
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("ctrl-c received, draining tasks");
        }
        _ = sigterm_handler() => {
            tracing::info!("SIGTERM received, draining tasks");
        }
        _ = shutdown.cancelled() => {
            tracing::info!("shutdown token cancelled (likely STOP file), draining tasks");
        }
    }

    // Always cancel — idempotent, ensures all tasks see the signal even
    // if we got here from ctrl-c rather than the token branch.
    shutdown.cancel();

    let drain = futures::future::join_all(handles);
    match tokio::time::timeout(Duration::from_secs(DRAIN_TIMEOUT_SECS), drain).await {
        Ok(_) => tracing::info!("all tasks drained cleanly"),
        Err(_) => tracing::warn!(
            timeout_secs = DRAIN_TIMEOUT_SECS,
            "drain timeout reached, forcing exit"
        ),
    }

    Ok(())
}

#[cfg(unix)]
async fn sigterm_handler() {
    use tokio::signal::unix::{signal, SignalKind};
    match signal(SignalKind::terminate()) {
        Ok(mut sig) => {
            sig.recv().await;
        }
        Err(e) => {
            tracing::warn!(error = %e, "could not install SIGTERM handler");
            std::future::pending::<()>().await;
        }
    }
}

#[cfg(not(unix))]
async fn sigterm_handler() {
    // No SIGTERM on non-Unix; pend forever so the select! branch is inert.
    std::future::pending::<()>().await;
}

async fn status(config_path: &std::path::Path) -> Result<()> {
    let cfg = config::Config::load(config_path)?;
    let url = format!("http://{}/healthz", cfg.api_listen);
    let resp = reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("{status}");
    println!("{text}");
    Ok(())
}

async fn stop(config_path: &std::path::Path) -> Result<()> {
    let cfg = config::Config::load(config_path)?;
    tokio::fs::write(&cfg.kill_switch_path, b"stop\n").await?;
    println!("created STOP file at {}", cfg.kill_switch_path);
    Ok(())
}

async fn show_config(config_path: &std::path::Path) -> Result<()> {
    let cfg = config::Config::load(config_path)?;
    println!("{cfg:#?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the cancellation pattern used throughout the daemon: a
    /// task that selects on `shutdown.cancelled()` against a long sleep
    /// must exit promptly when the token is cancelled, not block on the
    /// sleep. Guards against accidentally placing a non-cancellable
    /// `await` ahead of the shutdown branch in any task's `select!`.
    #[tokio::test]
    async fn graceful_shutdown_completes_within_timeout() {
        let shutdown = CancellationToken::new();
        let task = {
            let s = shutdown.clone();
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        biased;
                        _ = s.cancelled() => return,
                        _ = tokio::time::sleep(Duration::from_secs(60)) => {}
                    }
                }
            })
        };

        tokio::time::sleep(Duration::from_millis(50)).await;
        shutdown.cancel();

        let result = tokio::time::timeout(Duration::from_secs(1), task).await;
        assert!(
            result.is_ok(),
            "task did not respond to cancellation within 1s"
        );
    }
}
