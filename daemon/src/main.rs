use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

mod api;
mod config;
mod db;
mod exporter;
mod git_pusher;
mod kill_switch_watcher;
mod log_tail;
mod rpc_client;
mod stake_refresher;
mod vote_collector;
mod vote_parser;
mod watchdog;

const DEFAULT_CONFIG_PATH: &str = "/home/x1pio/strontium-meter/config.toml";

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

    let (tx_slot, rx_slot) = tokio::sync::mpsc::channel::<(u64, i64)>(1024);

    let cfg_log = cfg.log_path.clone();
    tokio::spawn(async move {
        if let Err(e) = log_tail::run(cfg_log, tx_slot).await {
            tracing::error!(error = %e, "log_tail terminated");
        }
    });

    let pool_vc = pool.clone();
    let rpc_vc = rpc.clone();
    tokio::spawn(async move {
        vote_collector::run(pool_vc, rpc_vc, rx_slot).await;
    });

    let pool_sr = pool.clone();
    let rpc_sr = rpc.clone();
    let stake_secs = cfg.stake_refresh_secs;
    tokio::spawn(async move {
        stake_refresher::run(pool_sr, rpc_sr, stake_secs).await;
    });

    let pool_ex = pool.clone();
    let cfg_ex = cfg.clone();
    tokio::spawn(async move {
        exporter::run(pool_ex, cfg_ex).await;
    });

    let pool_api = pool.clone();
    let cfg_api = cfg.clone();
    tokio::spawn(async move {
        if let Err(e) = api::run(pool_api, cfg_api).await {
            tracing::error!(error = %e, "api server terminated");
        }
    });

    let stop_path = cfg.kill_switch_path.clone();
    tokio::spawn(async move {
        kill_switch_watcher::run(stop_path).await;
    });

    let watchdog_secs = cfg.watchdog_secs;
    tokio::spawn(async move {
        if let Err(e) = watchdog::run(watchdog_secs).await {
            tracing::error!(error = %e, "watchdog terminated");
        }
    });

    tracing::info!("daemon running; ctrl-c to shutdown");
    tokio::signal::ctrl_c().await?;
    tracing::info!("shutdown");
    Ok(())
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
