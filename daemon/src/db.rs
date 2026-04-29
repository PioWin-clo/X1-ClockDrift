use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;

pub type Pool = SqlitePool;

#[derive(Debug, Clone)]
pub struct VoteRecord {
    pub validator: String,
    pub slot_voted: u64,
    pub ts_chain: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ValidatorSummary {
    pub validator: String,
    pub n_samples: i64,
    pub mean_drift_ms: f64,
    pub median_drift_ms: f64,
    pub stddev_drift_ms: f64,
    pub p10_drift_ms: f64,
    pub p90_drift_ms: f64,
    pub last_seen_slot: i64,
    pub last_stake_lamports: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct NetworkBucket {
    pub bucket_ts: i64,
    pub median_drift_ms: f64,
    pub mean_drift_ms: f64,
    pub stake_weighted_drift_ms: f64,
    pub n_validators: i64,
    pub n_samples: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChronyTracking {
    pub updated_at: i64,
    pub reference_id: Option<String>,
    pub reference_ip: Option<String>,
    pub stratum: Option<i64>,
    pub ref_time_unix: Option<f64>,
    pub system_offset_seconds: Option<f64>,
    pub last_offset_seconds: Option<f64>,
    pub rms_offset_seconds: Option<f64>,
    pub frequency_ppm: Option<f64>,
    pub residual_freq_ppm: Option<f64>,
    pub skew_ppm: Option<f64>,
    pub root_delay_seconds: Option<f64>,
    pub root_dispersion_seconds: Option<f64>,
    pub update_interval_seconds: Option<f64>,
    pub leap_status: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChronySource {
    pub ip: String,
    pub hostname: String,
    pub operator: String,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub mode: Option<String>,
    pub state: Option<String>,
    pub stratum: Option<i64>,
    pub poll_log2: Option<i64>,
    pub reach: Option<i64>,
    pub last_rx_seconds: Option<i64>,
    pub last_sample_offset_seconds: Option<f64>,
    pub last_sample_original_seconds: Option<f64>,
    pub last_sample_error_seconds: Option<f64>,
    pub updated_at: i64,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS slot_obs (
    slot INTEGER PRIMARY KEY,
    ts_local_us INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_slot_obs_ts ON slot_obs(ts_local_us);

CREATE TABLE IF NOT EXISTS vote_records (
    slot INTEGER NOT NULL,
    block_slot INTEGER NOT NULL,
    validator TEXT NOT NULL,
    ts_chain INTEGER NOT NULL,
    PRIMARY KEY (slot, validator, block_slot)
);
CREATE INDEX IF NOT EXISTS idx_vote_records_validator ON vote_records(validator);
CREATE INDEX IF NOT EXISTS idx_vote_records_slot ON vote_records(slot);

CREATE TABLE IF NOT EXISTS stake_snap (
    snapshot_ts INTEGER NOT NULL,
    validator TEXT NOT NULL,
    stake_lamports INTEGER NOT NULL,
    PRIMARY KEY (snapshot_ts, validator)
);

CREATE TABLE IF NOT EXISTS validator_drift_summary (
    validator TEXT PRIMARY KEY,
    n_samples INTEGER NOT NULL,
    mean_drift_ms REAL NOT NULL,
    median_drift_ms REAL NOT NULL,
    stddev_drift_ms REAL NOT NULL,
    p10_drift_ms REAL NOT NULL,
    p90_drift_ms REAL NOT NULL,
    last_seen_slot INTEGER NOT NULL,
    last_stake_lamports INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS network_drift_history (
    bucket_ts INTEGER PRIMARY KEY,
    median_drift_ms REAL NOT NULL,
    mean_drift_ms REAL NOT NULL,
    stake_weighted_drift_ms REAL NOT NULL,
    n_validators INTEGER NOT NULL,
    n_samples INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS error_log (
    ts INTEGER NOT NULL,
    source TEXT NOT NULL,
    message TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_error_log_ts ON error_log(ts);

CREATE TABLE IF NOT EXISTS chrony_tracking (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    updated_at INTEGER NOT NULL,
    reference_id TEXT,
    reference_ip TEXT,
    stratum INTEGER,
    ref_time_unix REAL,
    system_offset_seconds REAL,
    last_offset_seconds REAL,
    rms_offset_seconds REAL,
    frequency_ppm REAL,
    residual_freq_ppm REAL,
    skew_ppm REAL,
    root_delay_seconds REAL,
    root_dispersion_seconds REAL,
    update_interval_seconds REAL,
    leap_status TEXT
);

CREATE TABLE IF NOT EXISTS chrony_sources (
    ip TEXT PRIMARY KEY,
    hostname TEXT NOT NULL,
    operator TEXT NOT NULL,
    country_code TEXT,
    country_name TEXT,
    mode TEXT,
    state TEXT,
    stratum INTEGER,
    poll_log2 INTEGER,
    reach INTEGER,
    last_rx_seconds INTEGER,
    last_sample_offset_seconds REAL,
    last_sample_original_seconds REAL,
    last_sample_error_seconds REAL,
    updated_at INTEGER NOT NULL
);
"#;

pub async fn init(path: &str) -> Result<Pool> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }

    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{path}"))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await
        .with_context(|| format!("failed to open sqlite at {path}"))?;

    for stmt in SCHEMA.split(';') {
        let s = stmt.trim();
        if !s.is_empty() {
            sqlx::query(s).execute(&pool).await?;
        }
    }
    Ok(pool)
}

pub async fn record_slot_obs(pool: &Pool, slot: u64, ts_local_us: i64) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO slot_obs (slot, ts_local_us) VALUES (?1, ?2)")
        .bind(slot as i64)
        .bind(ts_local_us)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn record_votes(pool: &Pool, votes: &[VoteRecord], block_slot: u64) -> Result<usize> {
    if votes.is_empty() {
        return Ok(0);
    }
    let mut tx = pool.begin().await?;
    let mut inserted = 0usize;
    for v in votes {
        let res = sqlx::query(
            "INSERT OR IGNORE INTO vote_records (slot, block_slot, validator, ts_chain) \
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(v.slot_voted as i64)
        .bind(block_slot as i64)
        .bind(&v.validator)
        .bind(v.ts_chain)
        .execute(&mut *tx)
        .await?;
        if res.rows_affected() > 0 {
            inserted += 1;
        }
    }
    tx.commit().await?;
    Ok(inserted)
}

pub async fn record_stake_snapshot(
    pool: &Pool,
    snapshot_ts: i64,
    entries: &[(String, i64)],
) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    for (validator, stake) in entries {
        sqlx::query(
            "INSERT OR REPLACE INTO stake_snap (snapshot_ts, validator, stake_lamports) \
             VALUES (?1, ?2, ?3)",
        )
        .bind(snapshot_ts)
        .bind(validator)
        .bind(*stake)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn record_error(pool: &Pool, source: &str, message: &str) -> Result<()> {
    let ts = chrono::Utc::now().timestamp();
    sqlx::query("INSERT INTO error_log (ts, source, message) VALUES (?1, ?2, ?3)")
        .bind(ts)
        .bind(source)
        .bind(message)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn errors_in_last_hour(pool: &Pool) -> Result<i64> {
    let cutoff = chrono::Utc::now().timestamp() - 3600;
    let row = sqlx::query("SELECT COUNT(*) AS n FROM error_log WHERE ts >= ?1")
        .bind(cutoff)
        .fetch_one(pool)
        .await?;
    Ok(row.try_get::<i64, _>("n")?)
}

pub async fn slot_obs_count(pool: &Pool) -> Result<i64> {
    let row = sqlx::query("SELECT COUNT(*) AS n FROM slot_obs")
        .fetch_one(pool)
        .await?;
    Ok(row.try_get::<i64, _>("n")?)
}

pub async fn latest_slot(pool: &Pool) -> Result<Option<i64>> {
    let row = sqlx::query("SELECT MAX(slot) AS s FROM slot_obs")
        .fetch_one(pool)
        .await?;
    Ok(row.try_get::<Option<i64>, _>("s").unwrap_or(None))
}

pub async fn earliest_slot_since(pool: &Pool, cutoff_us: i64) -> Result<Option<i64>> {
    let row = sqlx::query("SELECT MIN(slot) AS s FROM slot_obs WHERE ts_local_us >= ?1")
        .bind(cutoff_us)
        .fetch_one(pool)
        .await?;
    Ok(row.try_get::<Option<i64>, _>("s").unwrap_or(None))
}

pub async fn latest_stake_per_validator(pool: &Pool) -> Result<Vec<(String, i64)>> {
    let rows = sqlx::query(
        "SELECT s.validator AS validator, s.stake_lamports AS stake \
         FROM stake_snap s \
         JOIN ( \
             SELECT validator, MAX(snapshot_ts) AS ts \
             FROM stake_snap \
             GROUP BY validator \
         ) latest ON latest.validator = s.validator AND latest.ts = s.snapshot_ts",
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push((r.try_get::<String, _>("validator")?, r.try_get::<i64, _>("stake")?));
    }
    Ok(out)
}

/// Drift sample joined with local time.
pub struct DriftSample {
    pub validator: String,
    pub slot: i64,
    pub drift_ms: f64,
}

/// Fetch all drift samples for the last `window_secs` seconds.
/// drift_ms = (ts_chain * 1000) - (ts_local_us / 1000)
pub async fn fetch_drift_samples_since(
    pool: &Pool,
    cutoff_local_us: i64,
) -> Result<Vec<DriftSample>> {
    let rows = sqlx::query(
        "SELECT v.validator AS validator, v.slot AS slot, \
                (CAST(v.ts_chain AS REAL) * 1000.0) - (CAST(s.ts_local_us AS REAL) / 1000.0) AS drift_ms \
         FROM vote_records v \
         JOIN slot_obs s ON s.slot = v.slot \
         WHERE s.ts_local_us >= ?1",
    )
    .bind(cutoff_local_us)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(DriftSample {
            validator: r.try_get::<String, _>("validator")?,
            slot: r.try_get::<i64, _>("slot")?,
            drift_ms: r.try_get::<f64, _>("drift_ms")?,
        });
    }
    Ok(out)
}

pub async fn recompute_validator_summaries(pool: &Pool) -> Result<usize> {
    let cutoff_us = (chrono::Utc::now().timestamp() - 24 * 3600) * 1_000_000;
    let samples = fetch_drift_samples_since(pool, cutoff_us).await?;

    if samples.is_empty() {
        return Ok(0);
    }

    let stake_map: std::collections::HashMap<String, i64> = latest_stake_per_validator(pool)
        .await?
        .into_iter()
        .collect();

    let mut by_validator: std::collections::HashMap<String, Vec<(i64, f64)>> =
        std::collections::HashMap::new();
    for s in samples {
        by_validator
            .entry(s.validator)
            .or_default()
            .push((s.slot, s.drift_ms));
    }

    let now = chrono::Utc::now().timestamp();
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM validator_drift_summary")
        .execute(&mut *tx)
        .await?;

    let mut written = 0usize;
    for (validator, mut points) in by_validator {
        if points.is_empty() {
            continue;
        }
        points.sort_by_key(|p| p.0);
        let last_seen_slot = points.last().map(|p| p.0).unwrap_or(0);
        let mut drifts: Vec<f64> = points.iter().map(|p| p.1).collect();
        drifts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let stats = Stats::from_sorted(&drifts);
        let stake = stake_map.get(&validator).copied().unwrap_or(0);

        sqlx::query(
            "INSERT INTO validator_drift_summary \
             (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
              p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .bind(&validator)
        .bind(stats.n as i64)
        .bind(stats.mean)
        .bind(stats.median)
        .bind(stats.stddev)
        .bind(stats.p10)
        .bind(stats.p90)
        .bind(last_seen_slot)
        .bind(stake)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        written += 1;
    }
    tx.commit().await?;
    Ok(written)
}

pub async fn recompute_network_history_bucket(pool: &Pool, bucket_ts: i64) -> Result<()> {
    let bucket_start_us = bucket_ts * 1_000_000;
    let bucket_end_us = (bucket_ts + 300) * 1_000_000;

    let rows = sqlx::query(
        "SELECT v.validator AS validator, \
                (CAST(v.ts_chain AS REAL) * 1000.0) - (CAST(s.ts_local_us AS REAL) / 1000.0) AS drift_ms \
         FROM vote_records v \
         JOIN slot_obs s ON s.slot = v.slot \
         WHERE s.ts_local_us >= ?1 AND s.ts_local_us < ?2",
    )
    .bind(bucket_start_us)
    .bind(bucket_end_us)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(());
    }

    let mut samples: Vec<(String, f64)> = Vec::with_capacity(rows.len());
    for r in rows {
        samples.push((
            r.try_get::<String, _>("validator")?,
            r.try_get::<f64, _>("drift_ms")?,
        ));
    }

    let stake_map: std::collections::HashMap<String, i64> = latest_stake_per_validator(pool)
        .await?
        .into_iter()
        .collect();

    let n_samples = samples.len() as i64;
    let mut drifts: Vec<f64> = samples.iter().map(|(_, d)| *d).collect();
    drifts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let stats = Stats::from_sorted(&drifts);

    let mut by_validator: std::collections::HashMap<String, Vec<f64>> =
        std::collections::HashMap::new();
    for (v, d) in samples {
        by_validator.entry(v).or_default().push(d);
    }
    let n_validators = by_validator.len() as i64;

    let mut weighted_sum = 0.0f64;
    let mut weight_total = 0.0f64;
    for (v, ds) in &by_validator {
        let stake = *stake_map.get(v).unwrap_or(&0) as f64;
        if stake <= 0.0 {
            continue;
        }
        let mean = ds.iter().sum::<f64>() / ds.len() as f64;
        weighted_sum += mean * stake;
        weight_total += stake;
    }
    let stake_weighted_drift_ms = if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        stats.mean
    };

    sqlx::query(
        "INSERT OR REPLACE INTO network_drift_history \
         (bucket_ts, median_drift_ms, mean_drift_ms, stake_weighted_drift_ms, n_validators, n_samples) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(bucket_ts)
    .bind(stats.median)
    .bind(stats.mean)
    .bind(stake_weighted_drift_ms)
    .bind(n_validators)
    .bind(n_samples)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn fetch_validator_summaries(pool: &Pool) -> Result<Vec<ValidatorSummary>> {
    let rows = sqlx::query(
        "SELECT validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at \
         FROM validator_drift_summary",
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(ValidatorSummary {
            validator: r.try_get("validator")?,
            n_samples: r.try_get("n_samples")?,
            mean_drift_ms: r.try_get("mean_drift_ms")?,
            median_drift_ms: r.try_get("median_drift_ms")?,
            stddev_drift_ms: r.try_get("stddev_drift_ms")?,
            p10_drift_ms: r.try_get("p10_drift_ms")?,
            p90_drift_ms: r.try_get("p90_drift_ms")?,
            last_seen_slot: r.try_get("last_seen_slot")?,
            last_stake_lamports: r.try_get("last_stake_lamports")?,
            updated_at: r.try_get("updated_at")?,
        });
    }
    Ok(out)
}

pub async fn fetch_network_history(pool: &Pool, since_ts: i64) -> Result<Vec<NetworkBucket>> {
    let rows = sqlx::query(
        "SELECT bucket_ts, median_drift_ms, mean_drift_ms, stake_weighted_drift_ms, \
                n_validators, n_samples \
         FROM network_drift_history WHERE bucket_ts >= ?1 ORDER BY bucket_ts ASC",
    )
    .bind(since_ts)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(NetworkBucket {
            bucket_ts: r.try_get("bucket_ts")?,
            median_drift_ms: r.try_get("median_drift_ms")?,
            mean_drift_ms: r.try_get("mean_drift_ms")?,
            stake_weighted_drift_ms: r.try_get("stake_weighted_drift_ms")?,
            n_validators: r.try_get("n_validators")?,
            n_samples: r.try_get("n_samples")?,
        });
    }
    Ok(out)
}

pub async fn fetch_validator_history(
    pool: &Pool,
    validator: &str,
    limit: i64,
) -> Result<Vec<(i64, i64, f64)>> {
    let rows = sqlx::query(
        "SELECT v.slot AS slot, s.ts_local_us AS ts_local_us, \
                (CAST(v.ts_chain AS REAL) * 1000.0) - (CAST(s.ts_local_us AS REAL) / 1000.0) AS drift_ms \
         FROM vote_records v \
         JOIN slot_obs s ON s.slot = v.slot \
         WHERE v.validator = ?1 \
         ORDER BY v.slot DESC LIMIT ?2",
    )
    .bind(validator)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push((
            r.try_get::<i64, _>("slot")?,
            r.try_get::<i64, _>("ts_local_us")?,
            r.try_get::<f64, _>("drift_ms")?,
        ));
    }
    Ok(out)
}

pub async fn record_chrony_tracking(pool: &Pool, t: &ChronyTracking) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO chrony_tracking \
         (id, updated_at, reference_id, reference_ip, stratum, ref_time_unix, \
          system_offset_seconds, last_offset_seconds, rms_offset_seconds, \
          frequency_ppm, residual_freq_ppm, skew_ppm, root_delay_seconds, \
          root_dispersion_seconds, update_interval_seconds, leap_status) \
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
    )
    .bind(t.updated_at)
    .bind(&t.reference_id)
    .bind(&t.reference_ip)
    .bind(t.stratum)
    .bind(t.ref_time_unix)
    .bind(t.system_offset_seconds)
    .bind(t.last_offset_seconds)
    .bind(t.rms_offset_seconds)
    .bind(t.frequency_ppm)
    .bind(t.residual_freq_ppm)
    .bind(t.skew_ppm)
    .bind(t.root_delay_seconds)
    .bind(t.root_dispersion_seconds)
    .bind(t.update_interval_seconds)
    .bind(&t.leap_status)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn record_chrony_sources(pool: &Pool, sources: &[ChronySource]) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM chrony_sources").execute(&mut *tx).await?;
    for s in sources {
        sqlx::query(
            "INSERT INTO chrony_sources \
             (ip, hostname, operator, country_code, country_name, mode, state, \
              stratum, poll_log2, reach, last_rx_seconds, last_sample_offset_seconds, \
              last_sample_original_seconds, last_sample_error_seconds, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        )
        .bind(&s.ip)
        .bind(&s.hostname)
        .bind(&s.operator)
        .bind(&s.country_code)
        .bind(&s.country_name)
        .bind(&s.mode)
        .bind(&s.state)
        .bind(s.stratum)
        .bind(s.poll_log2)
        .bind(s.reach)
        .bind(s.last_rx_seconds)
        .bind(s.last_sample_offset_seconds)
        .bind(s.last_sample_original_seconds)
        .bind(s.last_sample_error_seconds)
        .bind(s.updated_at)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn fetch_chrony_tracking(pool: &Pool) -> Result<Option<ChronyTracking>> {
    let row_opt = sqlx::query(
        "SELECT updated_at, reference_id, reference_ip, stratum, ref_time_unix, \
                system_offset_seconds, last_offset_seconds, rms_offset_seconds, \
                frequency_ppm, residual_freq_ppm, skew_ppm, root_delay_seconds, \
                root_dispersion_seconds, update_interval_seconds, leap_status \
         FROM chrony_tracking WHERE id = 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row_opt.map(|r| ChronyTracking {
        updated_at: r.try_get("updated_at").unwrap_or(0),
        reference_id: r.try_get("reference_id").ok(),
        reference_ip: r.try_get("reference_ip").ok(),
        stratum: r.try_get("stratum").ok(),
        ref_time_unix: r.try_get("ref_time_unix").ok(),
        system_offset_seconds: r.try_get("system_offset_seconds").ok(),
        last_offset_seconds: r.try_get("last_offset_seconds").ok(),
        rms_offset_seconds: r.try_get("rms_offset_seconds").ok(),
        frequency_ppm: r.try_get("frequency_ppm").ok(),
        residual_freq_ppm: r.try_get("residual_freq_ppm").ok(),
        skew_ppm: r.try_get("skew_ppm").ok(),
        root_delay_seconds: r.try_get("root_delay_seconds").ok(),
        root_dispersion_seconds: r.try_get("root_dispersion_seconds").ok(),
        update_interval_seconds: r.try_get("update_interval_seconds").ok(),
        leap_status: r.try_get("leap_status").ok(),
    }))
}

pub async fn fetch_chrony_sources(pool: &Pool) -> Result<Vec<ChronySource>> {
    let rows = sqlx::query(
        "SELECT ip, hostname, operator, country_code, country_name, mode, state, \
                stratum, poll_log2, reach, last_rx_seconds, last_sample_offset_seconds, \
                last_sample_original_seconds, last_sample_error_seconds, updated_at \
         FROM chrony_sources ORDER BY operator, hostname",
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(ChronySource {
            ip: r.try_get("ip")?,
            hostname: r.try_get("hostname")?,
            operator: r.try_get("operator")?,
            country_code: r.try_get("country_code").ok(),
            country_name: r.try_get("country_name").ok(),
            mode: r.try_get("mode").ok(),
            state: r.try_get("state").ok(),
            stratum: r.try_get("stratum").ok(),
            poll_log2: r.try_get("poll_log2").ok(),
            reach: r.try_get("reach").ok(),
            last_rx_seconds: r.try_get("last_rx_seconds").ok(),
            last_sample_offset_seconds: r.try_get("last_sample_offset_seconds").ok(),
            last_sample_original_seconds: r.try_get("last_sample_original_seconds").ok(),
            last_sample_error_seconds: r.try_get("last_sample_error_seconds").ok(),
            updated_at: r.try_get("updated_at").unwrap_or(0),
        });
    }
    Ok(out)
}

pub async fn get_best_synced_validators(
    pool: &Pool,
    limit: i64,
    min_samples: i64,
) -> Result<Vec<ValidatorSummary>> {
    let rows = sqlx::query(
        "SELECT validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at \
         FROM validator_drift_summary \
         WHERE n_samples >= ?1 \
         ORDER BY ABS(mean_drift_ms) ASC \
         LIMIT ?2",
    )
    .bind(min_samples)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(ValidatorSummary {
            validator: r.try_get("validator")?,
            n_samples: r.try_get("n_samples")?,
            mean_drift_ms: r.try_get("mean_drift_ms")?,
            median_drift_ms: r.try_get("median_drift_ms")?,
            stddev_drift_ms: r.try_get("stddev_drift_ms")?,
            p10_drift_ms: r.try_get("p10_drift_ms")?,
            p90_drift_ms: r.try_get("p90_drift_ms")?,
            last_seen_slot: r.try_get("last_seen_slot")?,
            last_stake_lamports: r.try_get("last_stake_lamports")?,
            updated_at: r.try_get("updated_at")?,
        });
    }
    Ok(out)
}

/// Backfill `network_drift_history` for every 5-minute bucket whose start
/// is within `lookback_secs` seconds of now. Calls
/// [`recompute_network_history_bucket`] for each, which is idempotent
/// (`INSERT OR REPLACE`) and a no-op when a bucket has no underlying
/// vote/slot_obs data. Cheap enough to run at every daemon start.
pub async fn backfill_history(pool: &Pool, lookback_secs: i64) -> Result<usize> {
    let now = chrono::Utc::now().timestamp();
    let earliest = now - lookback_secs;
    let mut bucket = (earliest / 300) * 300;
    let end = (now / 300) * 300;
    let mut count = 0usize;
    while bucket <= end {
        recompute_network_history_bucket(pool, bucket).await?;
        bucket += 300;
        count += 1;
    }
    Ok(count)
}

pub async fn cleanup_old(pool: &Pool, retention_days: u32, history_retention_days: u32) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let cutoff_us = (now - (retention_days as i64) * 86400) * 1_000_000;
    let cutoff_history = now - (history_retention_days as i64) * 86400;
    let cutoff_errors = now - 7 * 86400;

    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM slot_obs WHERE ts_local_us < ?1")
        .bind(cutoff_us)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM vote_records WHERE slot NOT IN (SELECT slot FROM slot_obs)")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM stake_snap WHERE snapshot_ts < ?1")
        .bind(now - 30 * 86400)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM network_drift_history WHERE bucket_ts < ?1")
        .bind(cutoff_history)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM error_log WHERE ts < ?1")
        .bind(cutoff_errors)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub fn current_5min_bucket(now_secs: i64) -> i64 {
    (now_secs / 300) * 300
}

struct Stats {
    n: usize,
    mean: f64,
    median: f64,
    stddev: f64,
    p10: f64,
    p90: f64,
}

impl Stats {
    fn from_sorted(sorted: &[f64]) -> Self {
        let n = sorted.len();
        if n == 0 {
            return Self {
                n: 0,
                mean: 0.0,
                median: 0.0,
                stddev: 0.0,
                p10: 0.0,
                p90: 0.0,
            };
        }
        let mean = sorted.iter().sum::<f64>() / n as f64;
        let var = if n > 1 {
            sorted.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64
        } else {
            0.0
        };
        let stddev = var.sqrt();
        let median = percentile_sorted(sorted, 0.5);
        let p10 = percentile_sorted(sorted, 0.1);
        let p90 = percentile_sorted(sorted, 0.9);
        Self {
            n,
            mean,
            median,
            stddev,
            p10,
            p90,
        }
    }
}

fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let pos = q * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    let frac = pos - lo as f64;
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_basic() {
        let v: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile_sorted(&v, 0.5) - 3.0).abs() < 1e-9);
        assert!((percentile_sorted(&v, 0.0) - 1.0).abs() < 1e-9);
        assert!((percentile_sorted(&v, 1.0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn stats_basic() {
        let v: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let s = Stats::from_sorted(&v);
        assert_eq!(s.n, 5);
        assert!((s.mean - 3.0).abs() < 1e-9);
        assert!((s.median - 3.0).abs() < 1e-9);
        assert!((s.stddev - 1.5811388300841898).abs() < 1e-9);
    }

    #[test]
    fn bucket_floors_to_5min() {
        assert_eq!(current_5min_bucket(1700000000), 1700000000 / 300 * 300);
        assert_eq!(current_5min_bucket(1700000299), (1700000299 / 300) * 300);
    }

    #[tokio::test]
    async fn end_to_end_inmemory() {
        let pool = init(":memory:").await.unwrap();
        let now_us = chrono::Utc::now().timestamp_micros();
        let now_s = now_us / 1_000_000;
        record_slot_obs(&pool, 100, now_us - 1_000_000).await.unwrap();
        record_slot_obs(&pool, 101, now_us).await.unwrap();
        let votes = vec![
            VoteRecord {
                validator: "VAL1".into(),
                slot_voted: 100,
                ts_chain: now_s - 1,
            },
            VoteRecord {
                validator: "VAL1".into(),
                slot_voted: 101,
                ts_chain: now_s,
            },
        ];
        let n = record_votes(&pool, &votes, 200).await.unwrap();
        assert_eq!(n, 2);

        record_stake_snapshot(&pool, now_s, &[("VAL1".into(), 5_000_000_000)])
            .await
            .unwrap();

        recompute_validator_summaries(&pool).await.unwrap();
        let summaries = fetch_validator_summaries(&pool).await.unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].validator, "VAL1");
        assert_eq!(summaries[0].n_samples, 2);
        assert_eq!(summaries[0].last_stake_lamports, 5_000_000_000);
    }

    /// Insert synthetic validator_drift_summary rows and verify
    /// `get_best_synced_validators` orders by absolute mean drift ascending,
    /// applies the min_samples filter, and respects the limit.
    #[tokio::test]
    async fn get_best_synced_orders_and_filters() {
        let pool = init(":memory:").await.unwrap();
        let now = chrono::Utc::now().timestamp();

        let rows = [
            // (validator, n_samples, mean_drift_ms, stake)
            ("BIG_DRIFT_HIGH_N", 100i64, 5000.0, 1_000_000_000i64),
            ("TINY_DRIFT_LOW_N", 3, 0.5, 100_000_000),
            ("TINY_DRIFT_HIGH_N", 50, 1.2, 500_000_000),
            ("ZERO_DRIFT", 8, 0.0, 200_000_000),
            ("NEGATIVE_TINY", 20, -2.4, 300_000_000),
            ("MEDIUM_DRIFT", 7, 50.0, 400_000_000),
        ];
        for (v, n, mean, stake) in rows.iter() {
            sqlx::query(
                "INSERT INTO validator_drift_summary \
                 (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                  p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at) \
                 VALUES (?1, ?2, ?3, ?3, 1.0, ?3, ?3, 0, ?4, ?5)",
            )
            .bind(*v)
            .bind(*n)
            .bind(*mean)
            .bind(*stake)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();
        }

        // min_samples=5 excludes TINY_DRIFT_LOW_N (n=3); top 3 by |mean|
        // among the remaining is ZERO_DRIFT (0), TINY_DRIFT_HIGH_N (1.2),
        // NEGATIVE_TINY (2.4).
        let best = get_best_synced_validators(&pool, 3, 5).await.unwrap();
        assert_eq!(best.len(), 3);
        assert_eq!(best[0].validator, "ZERO_DRIFT");
        assert_eq!(best[1].validator, "TINY_DRIFT_HIGH_N");
        assert_eq!(best[2].validator, "NEGATIVE_TINY");
        for w in best.windows(2) {
            assert!(
                w[0].mean_drift_ms.abs() <= w[1].mean_drift_ms.abs(),
                "results not sorted by abs(mean) ascending"
            );
        }

        // limit honoured even when more rows would qualify
        let best_limit_1 = get_best_synced_validators(&pool, 1, 5).await.unwrap();
        assert_eq!(best_limit_1.len(), 1);
        assert_eq!(best_limit_1[0].validator, "ZERO_DRIFT");

        // min_samples=200 filters everyone out
        let none = get_best_synced_validators(&pool, 10, 200).await.unwrap();
        assert!(none.is_empty());
    }
}
