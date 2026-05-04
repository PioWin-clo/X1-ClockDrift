use crate::chrony_reader;
use crate::config::Config;
use crate::db::{self, Pool};
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;
use tokio_util::sync::CancellationToken;

/// v0.5.0: bumped from 500 to 5000 to cover all ~2,259 active X1
/// mainnet validators (was excluding ~78% of network). Sort by impact
/// still puts highest-stake outliers first so the worst clocks lead.
const VALIDATORS_TOP_N: usize = 5000;
const BEST_SYNCED_TOP_N: i64 = 10;
/// v0.4.0: bumped from 5 to 100 — sub-100-sample validators are too noisy
/// for "best" claims. Foundation excluded inside the SQL.
const BEST_SYNCED_MIN_SAMPLES: i64 = 100;
const LAMPORTS_PER_XNT: f64 = 1_000_000_000.0;
/// v1.0.0 Layer 1/Layer 2 framework: the legacy combined "worst" ranking
/// is split into two tiers, each exported as its own JSON. The frontend
/// renders them as separate tables under "Anomalies & deviations" so
/// pipeline issues (operator should investigate infra) don't get
/// conflated with genuine clock drift (operator must fix NTP/chrony).
///
/// `worst_validators.json` continues to be written for one release so
/// external consumers don't 404. Removal scheduled for v1.1.0.
const PIPELINE_ANOMALIES_TOP_N: i64 = 100; // 500 ≤ |lag| < 5000 ms
const CLOCK_DRIFT_TOP_N: i64 = 50;         // |drift| ≥ 5000 ms
/// Legacy v0.6.0 worst combined ranking — DEPRECATED, kept for one
/// release. Only used to write `worst_validators.json` while consumers
/// migrate to the split tier endpoints.
const WORST_TOP_N: i64 = 100;
const WORST_MIN_SAMPLES: i64 = 20;
const WORST_MIN_ABS_DRIFT_MS: f64 = 500.0;
/// v0.5.0: foundation drift trend window — 14 days of 1-hour buckets.
const FOUNDATION_TREND_DAYS: u32 = 14;
const FOUNDATION_TREND_BUCKET_MINUTES: u32 = 60;
const HISTORY_BACKFILL_SECS: i64 = 7 * 86400;

pub async fn run(pool: Pool, config: Config, shutdown: CancellationToken) {
    tracing::info!(
        lookback_days = HISTORY_BACKFILL_SECS / 86400,
        "exporter backfilling network drift history"
    );
    match db::backfill_history(&pool, HISTORY_BACKFILL_SECS).await {
        Ok(n) => tracing::info!(buckets = n, "backfill complete"),
        Err(e) => {
            tracing::warn!(error = %e, "history backfill failed");
            let _ = db::record_error(&pool, "exporter", &format!("backfill: {e}")).await;
        }
    }

    let interval = std::time::Duration::from_secs(config.export_interval_secs);
    tracing::info!(secs = config.export_interval_secs, "exporter starting");

    loop {
        tokio::select! {
            biased;
            _ = shutdown.cancelled() => {
                tracing::info!("exporter shutting down (cycle in flight, if any, runs to completion)");
                return;
            }
            _ = tokio::time::sleep(interval) => {}
        }
        // Cycle runs uninterrupted once started — shutdown can wait,
        // because aborting mid-write would risk a half-pushed git commit.
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

    // v0.4.0 Hero #1 — X1 network time right now
    chain_time_unix: i64,
    chain_time_iso: String,
    real_utc_iso: String,
    drift_ms_now: f64,
    drift_24h_mean_ms: f64,
    drift_24h_stddev_ms: f64,

    // v0.4.0 Hero #2 — validator clock health
    n_critical: i64,
    n_high: i64,
    n_healthy: i64,
    n_foundation: i64,
    n_total: i64,

    // Existing fields (kept for backward compat with v0.3.x consumers)
    n_validators_observed: i64,
    n_samples_24h: i64,
    median_drift_ms: f64,
    mean_drift_ms: f64,
    stake_weighted_drift_ms: f64,
    validators_with_drift_over_1s: i64,
    validators_with_drift_over_5s: i64,
    latest_slot: Option<i64>,
    earliest_slot_24h: Option<i64>,

    // Cluster info (de-emphasised in v0.4.0 narrative)
    n_clusters_detected: i64,
    n_validators_in_clusters: i64,
    n_singletons: i64,
    largest_cluster_size: i64,
    largest_cluster_total_stake_xnt: f64,
    n_signature_groups: i64,
    n_validators_in_groups: i64,
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
    cluster_id: Option<i64>,
    cluster_size: i64,
    is_multi_node: bool,
    // v0.4.0 narrative fields
    is_foundation: bool,
    foundation_label: Option<String>,
    severity: Option<String>,
}

#[derive(Serialize)]
struct HistoryEntryJson {
    bucket_ts: i64,
    bucket_iso: String,
    median_drift_ms: f64,
    mean_drift_ms: f64,
    stake_weighted_drift_ms: f64,
    sentinel_offset_us: Option<f64>,
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

#[derive(Serialize)]
struct BestValidatorJson {
    rank: i64,
    vote_account: String,
    n_samples: i64,
    mean_drift_ms: f64,
    median_drift_ms: f64,
    stddev_drift_ms: f64,
    p10_drift_ms: f64,
    p90_drift_ms: f64,
    stake_lamports: i64,
    stake_xnt: f64,
    cluster_id: Option<i64>,
    cluster_size: i64,
    is_multi_node: bool,
    // v0.4.0 narrative fields
    is_foundation: bool,
    foundation_label: Option<String>,
    severity: Option<String>,
}

/// v0.4.0: separate showcase for the 12 X1 Labs Foundation nodes.
#[derive(Serialize)]
struct FoundationJson {
    rank: i64,
    vote_account: String,
    label: String,
    mean_drift_ms: f64,
    median_drift_ms: f64,
    stddev_drift_ms: f64,
    n_samples: i64,
    stake_lamports: i64,
    stake_xnt: f64,
}

/// v0.5.0: server-side worst-validators ranking. Was client-side sort
/// over `validators.json`; now a dedicated export so frontend doesn't
/// have to re-sort every page load and filter thresholds are explicit.
#[derive(Serialize)]
struct WorstValidatorJson {
    rank: i64,
    vote_account: String,
    n_samples: i64,
    mean_drift_ms: f64,
    median_drift_ms: f64,
    stddev_drift_ms: f64,
    p10_drift_ms: f64,
    p90_drift_ms: f64,
    stake_lamports: i64,
    stake_xnt: f64,
    is_foundation: bool,
    foundation_label: Option<String>,
    severity: Option<String>,
}

/// v0.5.0: one bucket of foundation cluster drift over time. Frontend
/// renders these as a line chart with shaded min/max band.
/// `bucket_ms` is millisecond unix time (Date()-friendly).
#[derive(Serialize)]
struct FoundationDriftBucketJson {
    bucket_ms: i64,
    avg_drift_ms: f64,
    min_drift_ms: f64,
    max_drift_ms: f64,
    stddev_drift_ms: f64,
    nodes_active: i64,
    n_samples: i64,
}

#[derive(Serialize)]
struct ValidatorHistoryBucket {
    ts: i64,
    drift_ms: f64,
}

#[derive(Serialize)]
struct ValidatorHistoryJson {
    vote_account: String,
    lookback_days: u64,
    buckets: Vec<ValidatorHistoryBucket>,
}

#[derive(Serialize)]
struct ChronyTrackingJson {
    stratum: Option<i64>,
    reference_id: Option<String>,
    reference_ip: Option<String>,
    reference_hostname: Option<String>,
    reference_operator: Option<String>,
    reference_country_code: Option<String>,
    system_offset_us: Option<i64>,
    last_offset_us: Option<i64>,
    rms_offset_us: Option<i64>,
    frequency_ppm: Option<f64>,
    skew_ppm: Option<f64>,
    root_delay_ms: Option<f64>,
    root_dispersion_ms: Option<f64>,
    update_interval_secs: Option<f64>,
    leap_status: Option<String>,
    updated_at_utc: Option<String>,
}

#[derive(Serialize)]
struct ChronySourceJson {
    ip: String,
    hostname: String,
    operator: String,
    country_code: Option<String>,
    country_name: Option<String>,
    stratum: Option<i64>,
    state: Option<String>,
    state_label_en: String,
    state_label_pl: String,
    offset_us: Option<i64>,
    last_rx_secs: Option<i64>,
    reach_octal: Option<String>,
}

#[derive(Serialize)]
struct ChronyJson {
    wall_clock_utc: String,
    tracking: ChronyTrackingJson,
    sources: Vec<ChronySourceJson>,
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

    let (n_clusters, n_in_clusters, n_singletons, largest_size, largest_stake_lamports) =
        db::fetch_cluster_summary(pool).await.unwrap_or((0, 0, 0, 0, 0));
    let largest_stake_xnt = largest_stake_lamports as f64 / LAMPORTS_PER_XNT;

    // v0.4.0 Hero #1: chain time vs real UTC.
    // drift_ms_now = median of the most-recent 5min bucket (live network state).
    // 24h trend = mean and stddev across 5min buckets in last 24h.
    let history_since_24h = now_secs - 24 * 3600;
    let history_24h = db::fetch_network_history(pool, history_since_24h)
        .await
        .unwrap_or_default();
    let drift_ms_now = history_24h
        .last()
        .map(|b| b.median_drift_ms)
        .unwrap_or(0.0);
    let (drift_24h_mean_ms, drift_24h_stddev_ms) = mean_and_stddev(
        history_24h.iter().map(|b| b.median_drift_ms).collect::<Vec<_>>().as_slice(),
    );
    let real_utc_iso = format_iso_millis(now_secs);
    let chain_time_unix = now_secs + (drift_ms_now / 1000.0).round() as i64;
    let chain_time_iso = format_iso(chain_time_unix);

    // v0.4.0 Hero #2: severity breakdown of the validator population.
    let n_critical = summaries.iter().filter(|s| s.severity.as_deref() == Some("critical")).count() as i64;
    let n_high     = summaries.iter().filter(|s| s.severity.as_deref() == Some("high")).count() as i64;
    let n_healthy  = summaries.iter().filter(|s| s.severity.as_deref() == Some("healthy")).count() as i64;
    let n_foundation = summaries.iter().filter(|s| s.is_foundation).count() as i64;
    let n_total = summaries.len() as i64;

    let summary = SummaryJson {
        generated_at_utc: format_iso(now_secs),
        // Hero #1
        chain_time_unix,
        chain_time_iso,
        real_utc_iso,
        drift_ms_now,
        drift_24h_mean_ms,
        drift_24h_stddev_ms,
        // Hero #2
        n_critical,
        n_high,
        n_healthy,
        n_foundation,
        n_total,
        // Backward-compat
        n_validators_observed: summaries.len() as i64,
        n_samples_24h: total_samples_24h,
        median_drift_ms: median_of_medians,
        mean_drift_ms: mean_of_means,
        stake_weighted_drift_ms: stake_weighted,
        validators_with_drift_over_1s: drift_over_1s,
        validators_with_drift_over_5s: drift_over_5s,
        latest_slot: latest,
        earliest_slot_24h: earliest,
        // Cluster stats (de-emphasised, kept for analytics section)
        n_clusters_detected: n_clusters,
        n_validators_in_clusters: n_in_clusters,
        n_singletons,
        largest_cluster_size: largest_size,
        largest_cluster_total_stake_xnt: largest_stake_xnt,
        n_signature_groups: n_clusters,
        n_validators_in_groups: n_in_clusters,
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
                cluster_id: s.cluster_id,
                cluster_size: s.cluster_size,
                is_multi_node: s.is_multi_node,
                is_foundation: s.is_foundation,
                foundation_label: s.foundation_label,
                severity: s.severity,
            }
        })
        .collect();

    let history_since = now_secs - 7 * 86400;
    let history = db::fetch_network_history(pool, history_since).await?;
    let chrony_history_map = db::fetch_chrony_history_map(pool, history_since)
        .await
        .unwrap_or_default();
    let history_json: Vec<HistoryEntryJson> = history
        .into_iter()
        .map(|h| HistoryEntryJson {
            bucket_ts: h.bucket_ts,
            bucket_iso: format_iso(h.bucket_ts),
            median_drift_ms: h.median_drift_ms,
            mean_drift_ms: h.mean_drift_ms,
            stake_weighted_drift_ms: h.stake_weighted_drift_ms,
            sentinel_offset_us: chrony_history_map.get(&h.bucket_ts).copied(),
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

    // Best-synced validators: smallest abs(mean) with v0.6.0 filters —
    // min 100 samples, foundation excluded (dedicated showcase),
    // |drift| < 5000 ms (defensive). Stake-based filter dropped: a 2-XNT
    // validator with NTP-discipline can have a tighter clock than a
    // 100k-XNT one, and Capybara delegation gating is a Foundation
    // business decision out of scope for this dashboard.
    let best_rows = db::get_best_synced_validators(
        pool,
        BEST_SYNCED_TOP_N,
        BEST_SYNCED_MIN_SAMPLES,
    )
    .await
    .unwrap_or_default();
    let best_json: Vec<BestValidatorJson> = best_rows
        .into_iter()
        .enumerate()
        .map(|(idx, s)| {
            let stake_xnt = s.last_stake_lamports as f64 / LAMPORTS_PER_XNT;
            BestValidatorJson {
                rank: (idx as i64) + 1,
                vote_account: s.validator,
                n_samples: s.n_samples,
                mean_drift_ms: s.mean_drift_ms,
                median_drift_ms: s.median_drift_ms,
                stddev_drift_ms: s.stddev_drift_ms,
                p10_drift_ms: s.p10_drift_ms,
                p90_drift_ms: s.p90_drift_ms,
                stake_lamports: s.last_stake_lamports,
                stake_xnt,
                cluster_id: s.cluster_id,
                cluster_size: s.cluster_size,
                is_multi_node: s.is_multi_node,
                is_foundation: s.is_foundation,
                foundation_label: s.foundation_label,
                severity: s.severity,
            }
        })
        .collect();

    // Foundation showcase — separate JSON for the X1 Labs nodes.
    let foundation_rows = db::fetch_foundation_validators(pool).await.unwrap_or_default();
    let foundation_json: Vec<FoundationJson> = foundation_rows
        .into_iter()
        .enumerate()
        .map(|(idx, s)| {
            let stake_xnt = s.last_stake_lamports as f64 / LAMPORTS_PER_XNT;
            FoundationJson {
                rank: (idx as i64) + 1,
                vote_account: s.validator,
                label: s.foundation_label.unwrap_or_else(|| "X1 Labs".into()),
                mean_drift_ms: s.mean_drift_ms,
                median_drift_ms: s.median_drift_ms,
                stddev_drift_ms: s.stddev_drift_ms,
                n_samples: s.n_samples,
                stake_lamports: s.last_stake_lamports,
                stake_xnt,
            }
        })
        .collect();

    // v1.0.0 Layer 1/Layer 2 split: pipeline anomalies (Tier 1) and
    // genuine clock drift (Tier 2) are now separate exports. The
    // frontend renders them as distinct tables under "Anomalies &
    // deviations" so a 2-XNT validator with -23s drift (real Layer 2
    // misconfig) doesn't get visually mixed with a 100k-XNT validator
    // sitting at -1.2s lag (slow pipeline / network / CPU saturation).
    let pipeline_rows = db::get_pipeline_anomalies(pool, PIPELINE_ANOMALIES_TOP_N)
        .await
        .unwrap_or_default();
    let pipeline_json: Vec<WorstValidatorJson> = pipeline_rows
        .into_iter()
        .enumerate()
        .map(|(idx, s)| worst_to_json(idx, s))
        .collect();

    let clock_drift_rows = db::get_clock_drift_validators(pool, CLOCK_DRIFT_TOP_N)
        .await
        .unwrap_or_default();
    let clock_drift_json: Vec<WorstValidatorJson> = clock_drift_rows
        .into_iter()
        .enumerate()
        .map(|(idx, s)| worst_to_json(idx, s))
        .collect();

    // v1.0.0: legacy combined ranking — kept for one release while
    // external consumers migrate to the split tier endpoints. Logged
    // once per cycle so we notice if anyone is still relying on it.
    // Removal scheduled for v1.1.0.
    #[allow(deprecated)]
    let worst_rows = db::get_worst_validators(
        pool,
        WORST_TOP_N,
        WORST_MIN_SAMPLES,
        WORST_MIN_ABS_DRIFT_MS,
    )
    .await
    .unwrap_or_default();
    let worst_json: Vec<WorstValidatorJson> = worst_rows
        .into_iter()
        .enumerate()
        .map(|(idx, s)| worst_to_json(idx, s))
        .collect();
    tracing::warn!(
        target: "exporter",
        "worst_validators.json is DEPRECATED in v1.0.0; use \
         pipeline_anomalies.json (Tier 1) and clock_drift.json (Tier 2). \
         Removal scheduled for v1.1.0."
    );

    // v0.5.0: foundation drift trend — 14 days × 1h buckets for the
    // 12-node X1 Labs cluster. Lets operators see whether X1 Labs
    // changed Tachyon config or NTP source by watching for sudden
    // step changes in the avg drift line.
    let foundation_trend: Vec<FoundationDriftBucketJson> = db::get_foundation_drift_history(
        pool,
        FOUNDATION_TREND_DAYS,
        FOUNDATION_TREND_BUCKET_MINUTES,
    )
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|b| FoundationDriftBucketJson {
        bucket_ms: b.bucket_ts_secs * 1000,
        avg_drift_ms: b.avg_drift_ms,
        min_drift_ms: b.min_drift_ms,
        max_drift_ms: b.max_drift_ms,
        stddev_drift_ms: b.stddev_drift_ms,
        nodes_active: b.nodes_active,
        n_samples: b.n_samples,
    })
    .collect();

    let chrony_json = build_chrony_json(pool, now_secs).await;

    // Per-validator histories: top 500 by impact + top 10 best-synced.
    let mut top_validators: Vec<String> =
        validators_json.iter().map(|v| v.pubkey.clone()).collect();
    for b in &best_json {
        if !top_validators.contains(&b.vote_account) {
            top_validators.push(b.vote_account.clone());
        }
    }
    let n_histories = match export_validator_histories(pool, &data_dir, &top_validators).await {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(error = %e, "export_validator_histories failed");
            0
        }
    };

    write_atomic(&data_dir.join("summary.json"), &summary).await?;
    write_atomic(&data_dir.join("validators.json"), &validators_json).await?;
    write_atomic(&data_dir.join("history.json"), &history_json).await?;
    write_atomic(&data_dir.join("meta.json"), &meta).await?;
    write_atomic(&data_dir.join("best_validators.json"), &best_json).await?;
    // v1.0.0: split worst into Tier 1 (pipeline anomalies) + Tier 2
    // (clock drift). worst_validators.json kept for one release.
    write_atomic(&data_dir.join("pipeline_anomalies.json"), &pipeline_json).await?;
    write_atomic(&data_dir.join("clock_drift.json"), &clock_drift_json).await?;
    write_atomic(&data_dir.join("worst_validators.json"), &worst_json).await?;
    write_atomic(&data_dir.join("chrony.json"), &chrony_json).await?;
    write_atomic(&data_dir.join("foundation.json"), &foundation_json).await?;
    write_atomic(
        &data_dir.join("foundation_drift_trend.json"),
        &foundation_trend,
    )
    .await?;

    tracing::info!(
        n_validators = summary.n_validators_observed,
        n_samples_24h = summary.n_samples_24h,
        n_best = best_json.len(),
        n_pipeline_anomalies = pipeline_json.len(),
        n_clock_drift = clock_drift_json.len(),
        n_worst_legacy = worst_json.len(),
        n_foundation = foundation_json.len(),
        n_foundation_trend = foundation_trend.len(),
        chrony_sources = chrony_json.sources.len(),
        n_clusters = summary.n_clusters_detected,
        n_critical = summary.n_critical,
        n_high = summary.n_high,
        n_histories,
        "wrote JSON exports"
    );
    Ok(())
}

/// v1.0.0: shared mapper used by Tier 1, Tier 2, and the legacy combined
/// ranking. All three queries return the same `ValidatorSummary` shape
/// and project to the same `WorstValidatorJson` schema.
fn worst_to_json(idx: usize, s: db::ValidatorSummary) -> WorstValidatorJson {
    let stake_xnt = s.last_stake_lamports as f64 / LAMPORTS_PER_XNT;
    WorstValidatorJson {
        rank: (idx as i64) + 1,
        vote_account: s.validator,
        n_samples: s.n_samples,
        mean_drift_ms: s.mean_drift_ms,
        median_drift_ms: s.median_drift_ms,
        stddev_drift_ms: s.stddev_drift_ms,
        p10_drift_ms: s.p10_drift_ms,
        p90_drift_ms: s.p90_drift_ms,
        stake_lamports: s.last_stake_lamports,
        stake_xnt,
        is_foundation: s.is_foundation,
        foundation_label: s.foundation_label,
        severity: s.severity,
    }
}

/// Plain mean and population stddev. Returns (0, 0) for empty input.
fn mean_and_stddev(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    if values.len() < 2 {
        return (mean, 0.0);
    }
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, var.sqrt())
}

const VALIDATOR_HISTORY_LOOKBACK_SECS: u64 = 7 * 86400;

/// Bucket raw `(ts_us, drift_ms)` samples into 5-minute medians, sorted ASC.
fn bucket_into_5min(history: &[(i64, f64)]) -> Vec<ValidatorHistoryBucket> {
    let mut by_bucket: std::collections::HashMap<i64, Vec<f64>> =
        std::collections::HashMap::new();
    for (ts_us, drift) in history {
        let bucket = (ts_us / 1_000_000 / 300) * 300;
        by_bucket.entry(bucket).or_default().push(*drift);
    }
    let mut out: Vec<ValidatorHistoryBucket> = by_bucket
        .into_iter()
        .map(|(ts, mut drifts)| {
            drifts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median = drifts[drifts.len() / 2];
            ValidatorHistoryBucket { ts, drift_ms: median }
        })
        .collect();
    out.sort_by_key(|b| b.ts);
    out
}

/// Write `data/validators/{pubkey}.json` for every pubkey in `top_validators`,
/// then prune any leftover files from previous cycles whose pubkey is no
/// longer in the top set. Each file contains 7 days of 5-minute median drift.
async fn export_validator_histories(
    pool: &Pool,
    data_dir: &Path,
    top_validators: &[String],
) -> Result<usize> {
    let dir = data_dir.join("validators");
    tokio::fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("creating {}", dir.display()))?;

    // Cleanup: remove .json files whose stem isn't in top_validators.
    let included: std::collections::HashSet<&String> = top_validators.iter().collect();
    let mut entries = tokio::fs::read_dir(&dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(stem) = name.strip_suffix(".json") {
                if !included.contains(&stem.to_string()) {
                    let _ = tokio::fs::remove_file(&path).await;
                }
            }
        }
    }

    let mut written = 0usize;
    for pubkey in top_validators {
        let history = match db::get_validator_history(pool, pubkey, VALIDATOR_HISTORY_LOOKBACK_SECS).await {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!(error = %e, validator = %pubkey, "get_validator_history failed");
                continue;
            }
        };
        if history.is_empty() {
            continue;
        }
        let buckets = bucket_into_5min(&history);
        let json = ValidatorHistoryJson {
            vote_account: pubkey.clone(),
            lookback_days: VALIDATOR_HISTORY_LOOKBACK_SECS / 86400,
            buckets,
        };
        write_atomic(&dir.join(format!("{pubkey}.json")), &json).await?;
        written += 1;
    }
    Ok(written)
}

fn seconds_to_us(s: f64) -> i64 {
    (s * 1_000_000.0).round() as i64
}

fn seconds_to_ms(s: f64) -> f64 {
    s * 1000.0
}

async fn build_chrony_json(pool: &Pool, now_secs: i64) -> ChronyJson {
    let tracking = db::fetch_chrony_tracking(pool).await.unwrap_or(None);
    let sources = db::fetch_chrony_sources(pool).await.unwrap_or_default();

    let tracking_json = if let Some(t) = tracking {
        // Resolve hostname/operator/country for the reference IP.
        let (ref_hostname, ref_operator, ref_cc) = match &t.reference_ip {
            Some(ip) => {
                let (hostname, operator, cc, _cn) = chrony_reader::lookup_source(ip);
                (Some(hostname), Some(operator), cc)
            }
            None => (None, None, None),
        };
        ChronyTrackingJson {
            stratum: t.stratum,
            reference_id: t.reference_id,
            reference_ip: t.reference_ip,
            reference_hostname: ref_hostname,
            reference_operator: ref_operator,
            reference_country_code: ref_cc,
            system_offset_us: t.system_offset_seconds.map(seconds_to_us),
            last_offset_us: t.last_offset_seconds.map(seconds_to_us),
            rms_offset_us: t.rms_offset_seconds.map(seconds_to_us),
            frequency_ppm: t.frequency_ppm,
            skew_ppm: t.skew_ppm,
            root_delay_ms: t.root_delay_seconds.map(seconds_to_ms),
            root_dispersion_ms: t.root_dispersion_seconds.map(seconds_to_ms),
            update_interval_secs: t.update_interval_seconds,
            leap_status: t.leap_status,
            updated_at_utc: Some(format_iso(t.updated_at)),
        }
    } else {
        ChronyTrackingJson {
            stratum: None,
            reference_id: None,
            reference_ip: None,
            reference_hostname: None,
            reference_operator: None,
            reference_country_code: None,
            system_offset_us: None,
            last_offset_us: None,
            rms_offset_us: None,
            frequency_ppm: None,
            skew_ppm: None,
            root_delay_ms: None,
            root_dispersion_ms: None,
            update_interval_secs: None,
            leap_status: None,
            updated_at_utc: None,
        }
    };

    let sources_json: Vec<ChronySourceJson> = sources
        .into_iter()
        .map(|s| {
            let state_str = s.state.as_deref().unwrap_or("unknown");
            let (label_en, label_pl) = chrony_reader::state_labels(state_str);
            let offset_us = s.last_sample_offset_seconds.map(seconds_to_us);
            let reach_octal = s.reach.map(|r| r.to_string());
            ChronySourceJson {
                ip: s.ip,
                hostname: s.hostname,
                operator: s.operator,
                country_code: s.country_code,
                country_name: s.country_name,
                stratum: s.stratum,
                state: s.state,
                state_label_en: label_en.to_string(),
                state_label_pl: label_pl.to_string(),
                offset_us,
                last_rx_secs: s.last_rx_seconds,
                reach_octal,
            }
        })
        .collect();

    ChronyJson {
        wall_clock_utc: format_iso_millis(now_secs),
        tracking: tracking_json,
        sources: sources_json,
    }
}

fn format_iso_millis(ts: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
        .map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".into())
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
