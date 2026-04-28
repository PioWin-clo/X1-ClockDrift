use crate::config::Config;
use crate::db::{self, Pool};
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

const VALIDATORS_TOP_N: usize = 500;
const LAMPORTS_PER_XNT: f64 = 1_000_000_000.0;

pub async fn run(pool: Pool, config: Config) {
    let interval = std::time::Duration::from_secs(config.export_interval_secs);
    tracing::info!(secs = config.export_interval_secs, "exporter starting");

    loop {
        tokio::time::sleep(interval).await;
        if let Err(e) = cycle(&pool, &config).await {
            tracing::warn!(error = %e, "export cycle outer error");
            let _ = db::record_error(&pool, "exporter", &e.to_string()).await;
        }
    }
}

async fn cycle(pool: &Pool, config: &Config) -> Result<()> {
    if let Err(e) = db::recompute_validator_summaries(pool).await {
        tracing::warn!(error = %e, "recompute validator summaries");
        let _ = db::record_error(pool, "recompute_summaries", &e.to_string()).await;
    }

    let now = chrono::Utc::now().timestamp();
    let bucket = db::current_5min_bucket(now);
    if let Err(e) = db::recompute_network_history_bucket(pool, bucket).await {
        tracing::warn!(error = %e, bucket, "recompute network history bucket");
        let _ = db::record_error(pool, "recompute_bucket", &e.to_string()).await;
    }

    if let Err(e) = write_json_files(pool, Path::new(&config.git_repo_path)).await {
        tracing::warn!(error = %e, "write json files");
        let _ = db::record_error(pool, "write_json", &e.to_string()).await;
    }

    if let Err(e) = crate::git_pusher::commit_and_push(config).await {
        tracing::warn!(error = %e, "git push");
        let _ = db::record_error(pool, "git_push", &e.to_string()).await;
    }

    if let Err(e) = db::cleanup_old(pool, config.retention_days, config.history_retention_days).await {
        tracing::warn!(error = %e, "cleanup");
        let _ = db::record_error(pool, "cleanup", &e.to_string()).await;
    }

    if let Err(e) = crate::git_pusher::maybe_daily_squash(config).await {
        tracing::warn!(error = %e, "daily squash");
        let _ = db::record_error(pool, "daily_squash", &e.to_string()).await;
    }

    Ok(())
}

#[derive(Serialize)]
struct SummaryJson {
    generated_at_utc: String,
    n_validators_observed: i64,
    n_samples_24h: i64,
    median_drift_ms: f64,
    mean_drift_ms: f64,
    stake_weighted_drift_ms: f64,
    validators_with_drift_over_1s: i64,
    validators_with_drift_over_5s: i64,
    latest_slot: Option<i64>,
    earliest_slot_24h: Option<i64>,
}

#[derive(Serialize)]
struct ValidatorJson {
    pubkey: String,
    mean_drift_ms: f64,
    median_drift_ms: f64,
    stddev_drift_ms: f64,
    p10_drift_ms: f64,
    p90_drift_ms: f64,
    n_samples: i64,
    last_seen_slot: i64,
    stake_lamports: i64,
    stake_xnt: f64,
    weighted_impact_ms_xnt: f64,
}

#[derive(Serialize)]
struct HistoryEntryJson {
    bucket_ts: i64,
    bucket_iso: String,
    median_drift_ms: f64,
    mean_drift_ms: f64,
    stake_weighted_drift_ms: f64,
    n_validators: i64,
    n_samples: i64,
}

#[derive(Serialize)]
struct MetaJson {
    generated_at_utc: String,
    daemon_version: &'static str,
    total_slots_observed: i64,
    total_votes_collected: i64,
    earliest_slot_observed: Option<i64>,
    latest_slot_observed: Option<i64>,
}

pub async fn write_json_files(pool: &Pool, repo_root: &Path) -> Result<()> {
    let data_dir = repo_root.join("data");
    tokio::fs::create_dir_all(&data_dir)
        .await
        .with_context(|| format!("creating {}", data_dir.display()))?;

    let summaries = db::fetch_validator_summaries(pool).await?;
    let now_secs = chrono::Utc::now().timestamp();
    let now_us = now_secs * 1_000_000;
    let cutoff_us = now_us - 24 * 3600 * 1_000_000;

    let mut median_pool: Vec<f64> = summaries.iter().map(|s| s.median_drift_ms).collect();
    median_pool.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_of_medians = if median_pool.is_empty() {
        0.0
    } else {
        median_pool[median_pool.len() / 2]
    };
    let mean_of_means = if summaries.is_empty() {
        0.0
    } else {
        summaries.iter().map(|s| s.mean_drift_ms).sum::<f64>() / summaries.len() as f64
    };
    let total_samples_24h: i64 = summaries.iter().map(|s| s.n_samples).sum();
    let drift_over_1s = summaries.iter().filter(|s| s.mean_drift_ms.abs() >= 1000.0).count() as i64;
    let drift_over_5s = summaries.iter().filter(|s| s.mean_drift_ms.abs() >= 5000.0).count() as i64;

    let stake_weighted = stake_weighted_mean(&summaries);

    let latest = db::latest_slot(pool).await?;
    let earliest = db::earliest_slot_since(pool, cutoff_us).await?;

    let summary = SummaryJson {
        generated_at_utc: format_iso(now_secs),
        n_validators_observed: summaries.len() as i64,
        n_samples_24h: total_samples_24h,
        median_drift_ms: median_of_medians,
        mean_drift_ms: mean_of_means,
        stake_weighted_drift_ms: stake_weighted,
        validators_with_drift_over_1s: drift_over_1s,
        validators_with_drift_over_5s: drift_over_5s,
        latest_slot: latest,
        earliest_slot_24h: earliest,
    };

    let mut sorted = summaries.clone();
    sorted.sort_by(|a, b| {
        b.mean_drift_ms
            .abs()
            .partial_cmp(&a.mean_drift_ms.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.truncate(VALIDATORS_TOP_N);

    let validators_json: Vec<ValidatorJson> = sorted
        .into_iter()
        .map(|s| {
            let stake_xnt = s.last_stake_lamports as f64 / LAMPORTS_PER_XNT;
            let weighted_impact_ms_xnt = (s.mean_drift_ms * s.last_stake_lamports as f64) / LAMPORTS_PER_XNT;
            ValidatorJson {
                pubkey: s.validator,
                mean_drift_ms: s.mean_drift_ms,
                median_drift_ms: s.median_drift_ms,
                stddev_drift_ms: s.stddev_drift_ms,
                p10_drift_ms: s.p10_drift_ms,
                p90_drift_ms: s.p90_drift_ms,
                n_samples: s.n_samples,
                last_seen_slot: s.last_seen_slot,
                stake_lamports: s.last_stake_lamports,
                stake_xnt,
                weighted_impact_ms_xnt,
            }
        })
        .collect();

    let history_since = now_secs - 7 * 86400;
    let history = db::fetch_network_history(pool, history_since).await?;
    let history_json: Vec<HistoryEntryJson> = history
        .into_iter()
        .map(|h| HistoryEntryJson {
            bucket_ts: h.bucket_ts,
            bucket_iso: format_iso(h.bucket_ts),
            median_drift_ms: h.median_drift_ms,
            mean_drift_ms: h.mean_drift_ms,
            stake_weighted_drift_ms: h.stake_weighted_drift_ms,
            n_validators: h.n_validators,
            n_samples: h.n_samples,
        })
        .collect();

    let total_slots = db::slot_obs_count(pool).await?;
    let earliest_obs = db::earliest_slot_since(pool, 0).await?;
    let total_votes = total_votes_count(pool).await.unwrap_or(0);
    let meta = MetaJson {
        generated_at_utc: format_iso(now_secs),
        daemon_version: env!("CARGO_PKG_VERSION"),
        total_slots_observed: total_slots,
        total_votes_collected: total_votes,
        earliest_slot_observed: earliest_obs,
        latest_slot_observed: latest,
    };

    write_atomic(&data_dir.join("summary.json"), &summary).await?;
    write_atomic(&data_dir.join("validators.json"), &validators_json).await?;
    write_atomic(&data_dir.join("history.json"), &history_json).await?;
    write_atomic(&data_dir.join("meta.json"), &meta).await?;

    tracing::info!(
        n_validators = summary.n_validators_observed,
        n_samples_24h = summary.n_samples_24h,
        "wrote JSON exports"
    );
    Ok(())
}

async fn total_votes_count(pool: &Pool) -> Result<i64> {
    use sqlx::Row;
    let row = sqlx::query("SELECT COUNT(*) AS n FROM vote_records")
        .fetch_one(pool)
        .await?;
    Ok(row.try_get::<i64, _>("n")?)
}

fn stake_weighted_mean(summaries: &[db::ValidatorSummary]) -> f64 {
    let mut num = 0.0f64;
    let mut den = 0.0f64;
    for s in summaries {
        let w = s.last_stake_lamports as f64;
        if w <= 0.0 {
            continue;
        }
        num += s.mean_drift_ms * w;
        den += w;
    }
    if den > 0.0 {
        num / den
    } else {
        0.0
    }
}

fn format_iso(ts: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
        .map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".into())
}

async fn write_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    let tmp = path.with_extension("json.tmp");
    tokio::fs::write(&tmp, json.as_bytes()).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}
