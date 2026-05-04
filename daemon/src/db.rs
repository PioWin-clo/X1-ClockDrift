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
    pub cluster_id: Option<i64>,
    pub cluster_size: i64,
    pub is_multi_node: bool,
    pub is_foundation: bool,
    pub foundation_label: Option<String>,
    pub severity: Option<String>,
}

/// One cluster row produced by the cluster-detection query.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClusterRow {
    pub cluster_id: i64,
    pub cluster_size: i64,
    pub members: Vec<String>,
    pub r_mean: i64,
    pub r_stddev: i64,
    pub r_p10: i64,
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
    updated_at INTEGER NOT NULL,
    cluster_id INTEGER,
    cluster_size INTEGER DEFAULT 1,
    is_multi_node INTEGER DEFAULT 0,
    is_foundation INTEGER DEFAULT 0,
    foundation_label TEXT,
    severity TEXT
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

CREATE TABLE IF NOT EXISTS chrony_tracking_history (
    bucket_ts INTEGER PRIMARY KEY,
    avg_system_offset_us REAL NOT NULL,
    avg_rms_offset_us REAL NOT NULL,
    sample_count INTEGER NOT NULL,
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
    migrate_v3(&pool).await?;
    migrate_v4(&pool).await?;
    Ok(pool)
}

/// One-time idempotent migration to add v0.3.0 columns to
/// `validator_drift_summary` on databases that already exist from earlier
/// releases. SQLite has no `ADD COLUMN IF NOT EXISTS`, so we probe via
/// `PRAGMA table_info` first. New `CREATE TABLE` statements in SCHEMA are
/// already idempotent.
async fn migrate_v3(pool: &Pool) -> Result<()> {
    let rows = sqlx::query("PRAGMA table_info(validator_drift_summary)")
        .fetch_all(pool)
        .await?;
    let cols: Vec<String> = rows
        .iter()
        .map(|r| r.try_get::<String, _>("name").unwrap_or_default())
        .collect();

    if !cols.iter().any(|c| c == "cluster_id") {
        sqlx::query("ALTER TABLE validator_drift_summary ADD COLUMN cluster_id INTEGER")
            .execute(pool)
            .await?;
        tracing::info!("migrate_v3: added cluster_id");
    }
    if !cols.iter().any(|c| c == "cluster_size") {
        sqlx::query("ALTER TABLE validator_drift_summary ADD COLUMN cluster_size INTEGER DEFAULT 1")
            .execute(pool)
            .await?;
        tracing::info!("migrate_v3: added cluster_size");
    }
    if !cols.iter().any(|c| c == "is_multi_node") {
        sqlx::query("ALTER TABLE validator_drift_summary ADD COLUMN is_multi_node INTEGER DEFAULT 0")
            .execute(pool)
            .await?;
        tracing::info!("migrate_v3: added is_multi_node");
    }
    // Index on cluster_id is created here, NOT in SCHEMA, because on an
    // existing v0.2.0 database CREATE TABLE IF NOT EXISTS is a no-op and the
    // column wouldn't exist yet — SQLite would reject the index with
    // "no such column: cluster_id". This ordering is guarded by the
    // migration_v2_to_v3_preserves_data integration test.
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_drift_summary_cluster ON validator_drift_summary(cluster_id)")
        .execute(pool)
        .await?;
    Ok(())
}

/// v0.4.0 migration: add foundation/severity columns to
/// `validator_drift_summary`, then flag known X1 Labs Foundation rows.
/// Idempotent — safe to run on fresh installs (where columns already exist
/// from SCHEMA) and on existing v0.3.x DBs.
async fn migrate_v4(pool: &Pool) -> Result<()> {
    let rows = sqlx::query("PRAGMA table_info(validator_drift_summary)")
        .fetch_all(pool)
        .await?;
    let cols: Vec<String> = rows
        .iter()
        .map(|r| r.try_get::<String, _>("name").unwrap_or_default())
        .collect();

    if !cols.iter().any(|c| c == "is_foundation") {
        sqlx::query("ALTER TABLE validator_drift_summary ADD COLUMN is_foundation INTEGER DEFAULT 0")
            .execute(pool)
            .await?;
        tracing::info!("migrate_v4: added is_foundation");
    }
    if !cols.iter().any(|c| c == "foundation_label") {
        sqlx::query("ALTER TABLE validator_drift_summary ADD COLUMN foundation_label TEXT")
            .execute(pool)
            .await?;
        tracing::info!("migrate_v4: added foundation_label");
    }
    if !cols.iter().any(|c| c == "severity") {
        sqlx::query("ALTER TABLE validator_drift_summary ADD COLUMN severity TEXT")
            .execute(pool)
            .await?;
        tracing::info!("migrate_v4: added severity");
    }

    flag_foundation_validators(pool).await?;
    Ok(())
}

/// One-shot UPDATE that flags known X1 Labs Foundation pubkeys in
/// `validator_drift_summary`. Idempotent — re-running just refreshes
/// the labels. Called both from `migrate_v4` (so existing rows are
/// labelled even before the next recompute cycle) and is no-op for
/// rows that don't yet exist (the recompute_validator_summaries INSERT
/// also sets these fields directly).
pub async fn flag_foundation_validators(pool: &Pool) -> Result<usize> {
    let mut count = 0usize;
    for f in crate::foundation::X1_LABS_FOUNDATION {
        let res = sqlx::query(
            "UPDATE validator_drift_summary \
             SET is_foundation = 1, foundation_label = ?1 \
             WHERE validator = ?2",
        )
        .bind(f.label)
        .bind(f.vote_account)
        .execute(pool)
        .await?;
        count += res.rows_affected() as usize;
    }
    Ok(count)
}

/// Severity bucket for a validator given their drift stats. Returns None
/// for under-sampled validators (n_samples < 5).
///
/// v0.4.1: foundation flag no longer short-circuits to "healthy". A
/// foundation node with +5s drift has a real operational problem and
/// should be classified `critical` like any other validator. The
/// `is_foundation` column is independent and rendered as a separate
/// 🏛️ badge on the frontend (next to the severity icon, not instead).
pub fn classify_severity(n_samples: i64, mean_drift_ms: f64) -> Option<&'static str> {
    if n_samples < 5 {
        return None;
    }
    let abs = mean_drift_ms.abs();
    if abs > 5000.0 {
        Some("critical")
    } else if abs > 1000.0 {
        Some("high")
    } else {
        Some("healthy")
    }
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

        // v0.4.0: foundation lookup + severity classification at INSERT time
        // so the row carries correct flags from the moment it lands. Avoids
        // a second-pass UPDATE and a window where the dashboard could read
        // unflagged rows.
        // v0.4.1: severity is independent of foundation status — a foundation
        // node with critical drift is still critical.
        let foundation_node = crate::foundation::lookup_foundation(&validator);
        let is_foundation: i64 = if foundation_node.is_some() { 1 } else { 0 };
        let foundation_label: Option<&'static str> = foundation_node.map(|f| f.label);
        let severity = classify_severity(stats.n as i64, stats.mean);

        sqlx::query(
            "INSERT INTO validator_drift_summary \
             (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
              p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
              is_foundation, foundation_label, severity) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
        .bind(is_foundation)
        .bind(foundation_label)
        .bind(severity)
        .execute(&mut *tx)
        .await?;
        written += 1;
    }
    tx.commit().await?;

    // Phase 2 (v0.3.0): detect operator clusters from drift signature.
    // Singletons keep cluster_id NULL, cluster_size 1, is_multi_node 0
    // — those are the schema defaults set by the INSERT above. Foundation
    // rows are excluded from cluster detection in v0.4.0 so their shared
    // infrastructure isn't reported as a "farm".
    if let Err(e) = detect_and_assign_clusters(pool).await {
        tracing::warn!(error = %e, "cluster detection failed");
    }

    Ok(written)
}

/// Detect operator clusters by drift signature: validators that share
/// `(round(mean), round(stddev), round(p10))` with at least 3 members are
/// flagged as `is_multi_node = 1`, given the same `cluster_size` and a
/// 1-indexed `cluster_id` ordered by descending size.
///
/// Idempotent — safe to call repeatedly. Resets all cluster columns to
/// defaults first so removed members get re-classified as singletons.
pub async fn detect_and_assign_clusters(pool: &Pool) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE validator_drift_summary \
         SET cluster_id = NULL, cluster_size = 1, is_multi_node = 0",
    )
    .execute(&mut *tx)
    .await?;

    // Note: alias `group_size` (not `cluster_size`) — the table itself has
    // a `cluster_size` column, and SQLite would resolve `HAVING cluster_size`
    // against that column (always 1 from the UPDATE above) instead of the
    // COUNT aggregate, silently producing zero clusters.
    //
    // `is_foundation = 0` filter: X1 Labs Foundation nodes share infra by
    // design and would otherwise show up as the largest "farm". They get
    // a dedicated showcase section instead.
    let rows = sqlx::query(
        "SELECT \
            CAST(round(mean_drift_ms) AS INTEGER) AS r_mean, \
            CAST(round(stddev_drift_ms) AS INTEGER) AS r_stddev, \
            CAST(round(p10_drift_ms) AS INTEGER) AS r_p10, \
            COUNT(*) AS group_size, \
            GROUP_CONCAT(validator) AS members \
         FROM validator_drift_summary \
         WHERE n_samples >= 5 AND is_foundation = 0 \
         GROUP BY r_mean, r_stddev, r_p10 \
         HAVING group_size >= 3 \
         ORDER BY group_size DESC, r_mean ASC",
    )
    .fetch_all(&mut *tx)
    .await?;

    for (idx, r) in rows.iter().enumerate() {
        let cluster_id = (idx as i64) + 1;
        let cluster_size: i64 = r.try_get("group_size")?;
        let members: String = r.try_get("members")?;
        for member in members.split(',') {
            let m = member.trim();
            if m.is_empty() {
                continue;
            }
            sqlx::query(
                "UPDATE validator_drift_summary \
                 SET cluster_id = ?1, cluster_size = ?2, is_multi_node = 1 \
                 WHERE validator = ?3",
            )
            .bind(cluster_id)
            .bind(cluster_size)
            .bind(m)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(())
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
                p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                cluster_id, cluster_size, is_multi_node, \
                is_foundation, foundation_label, severity \
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
            cluster_id: r.try_get::<Option<i64>, _>("cluster_id").unwrap_or(None),
            cluster_size: r.try_get("cluster_size").unwrap_or(1),
            is_multi_node: r.try_get::<i64, _>("is_multi_node").unwrap_or(0) != 0,
            is_foundation: r.try_get::<i64, _>("is_foundation").unwrap_or(0) != 0,
            foundation_label: r.try_get::<Option<String>, _>("foundation_label").unwrap_or(None),
            severity: r.try_get::<Option<String>, _>("severity").unwrap_or(None),
        });
    }
    Ok(out)
}

/// Foundation validators only, sorted by absolute mean drift descending
/// (worst drifters first — that's what the operator wants to see).
pub async fn fetch_foundation_validators(pool: &Pool) -> Result<Vec<ValidatorSummary>> {
    let rows = sqlx::query(
        "SELECT validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                cluster_id, cluster_size, is_multi_node, \
                is_foundation, foundation_label, severity \
         FROM validator_drift_summary \
         WHERE is_foundation = 1 \
         ORDER BY ABS(mean_drift_ms) DESC",
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
            cluster_id: r.try_get::<Option<i64>, _>("cluster_id").unwrap_or(None),
            cluster_size: r.try_get("cluster_size").unwrap_or(1),
            is_multi_node: r.try_get::<i64, _>("is_multi_node").unwrap_or(0) != 0,
            is_foundation: r.try_get::<i64, _>("is_foundation").unwrap_or(0) != 0,
            foundation_label: r.try_get::<Option<String>, _>("foundation_label").unwrap_or(None),
            severity: r.try_get::<Option<String>, _>("severity").unwrap_or(None),
        });
    }
    Ok(out)
}

/// Aggregate cluster summary stats for the dashboard.
/// Returns: (n_clusters, n_in_clusters, n_singletons, largest_size, largest_total_stake_lamports).
pub async fn fetch_cluster_summary(pool: &Pool) -> Result<(i64, i64, i64, i64, i64)> {
    let row = sqlx::query(
        "SELECT \
            (SELECT COUNT(DISTINCT cluster_id) FROM validator_drift_summary WHERE cluster_id IS NOT NULL) AS n_clusters, \
            (SELECT COUNT(*) FROM validator_drift_summary WHERE is_multi_node = 1) AS n_in, \
            (SELECT COUNT(*) FROM validator_drift_summary WHERE is_multi_node = 0) AS n_singletons",
    )
    .fetch_one(pool)
    .await?;
    let n_clusters: i64 = row.try_get("n_clusters").unwrap_or(0);
    let n_in: i64 = row.try_get("n_in").unwrap_or(0);
    let n_singletons: i64 = row.try_get("n_singletons").unwrap_or(0);

    let largest = sqlx::query(
        "SELECT cluster_id, cluster_size, SUM(last_stake_lamports) AS total_stake \
         FROM validator_drift_summary \
         WHERE cluster_id IS NOT NULL \
         GROUP BY cluster_id \
         ORDER BY cluster_size DESC, total_stake DESC \
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    let (largest_size, largest_stake) = match largest {
        Some(r) => (
            r.try_get::<i64, _>("cluster_size").unwrap_or(0),
            r.try_get::<i64, _>("total_stake").unwrap_or(0),
        ),
        None => (0, 0),
    };
    Ok((n_clusters, n_in, n_singletons, largest_size, largest_stake))
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

/// Best-synced ranking — v0.6.0 dropped the stake-based business filter.
///
/// Filters kept:
///   * `min_samples` — statistical sufficiency (default 100 in exporter)
///   * `is_foundation = 0` — Foundation has its own showcase section
///   * `ABS(mean_drift_ms) < 5000` — defensive: a validator with 100+
///     samples but |drift| ≥ 5 s is pathological, not "best synced"
///
/// Filter removed:
///   * `last_stake_lamports >= 1000 XNT` — stake doesn't determine clock
///     quality. A 2-XNT validator with NTP-discipline can have a tighter
///     clock than a 100k-XNT one. Capybara delegation gating is a
///     Foundation business decision, out of scope for this dashboard.
pub async fn get_best_synced_validators(
    pool: &Pool,
    limit: i64,
    min_samples: i64,
) -> Result<Vec<ValidatorSummary>> {
    let rows = sqlx::query(
        "SELECT validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                cluster_id, cluster_size, is_multi_node, \
                is_foundation, foundation_label, severity \
         FROM validator_drift_summary \
         WHERE n_samples >= ?1 \
           AND is_foundation = 0 \
           AND ABS(mean_drift_ms) < 5000 \
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
            cluster_id: r.try_get::<Option<i64>, _>("cluster_id").unwrap_or(None),
            cluster_size: r.try_get("cluster_size").unwrap_or(1),
            is_multi_node: r.try_get::<i64, _>("is_multi_node").unwrap_or(0) != 0,
            is_foundation: r.try_get::<i64, _>("is_foundation").unwrap_or(0) != 0,
            foundation_label: r.try_get::<Option<String>, _>("foundation_label").unwrap_or(None),
            severity: r.try_get::<Option<String>, _>("severity").unwrap_or(None),
        });
    }
    Ok(out)
}

/// Running-mean accumulator for `chrony_tracking_history`.
/// Each successful chrony poll feeds (system_offset_us, rms_offset_us)
/// into the current 5-minute bucket.
pub async fn accumulate_chrony_history(
    pool: &Pool,
    bucket_ts: i64,
    system_offset_us: f64,
    rms_offset_us: f64,
) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO chrony_tracking_history \
         (bucket_ts, avg_system_offset_us, avg_rms_offset_us, sample_count, updated_at) \
         VALUES (?1, ?2, ?3, 1, ?4) \
         ON CONFLICT(bucket_ts) DO UPDATE SET \
             avg_system_offset_us = (avg_system_offset_us * sample_count + ?2) / (sample_count + 1), \
             avg_rms_offset_us    = (avg_rms_offset_us    * sample_count + ?3) / (sample_count + 1), \
             sample_count = sample_count + 1, \
             updated_at = ?4",
    )
    .bind(bucket_ts)
    .bind(system_offset_us)
    .bind(rms_offset_us)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Per-bucket map for the history chart's third dataset.
pub async fn fetch_chrony_history_map(
    pool: &Pool,
    since_ts: i64,
) -> Result<std::collections::HashMap<i64, f64>> {
    let rows = sqlx::query(
        "SELECT bucket_ts, avg_system_offset_us \
         FROM chrony_tracking_history \
         WHERE bucket_ts >= ?1",
    )
    .bind(since_ts)
    .fetch_all(pool)
    .await?;
    let mut map = std::collections::HashMap::with_capacity(rows.len());
    for r in rows {
        let ts: i64 = r.try_get("bucket_ts")?;
        let v: f64 = r.try_get("avg_system_offset_us")?;
        map.insert(ts, v);
    }
    Ok(map)
}

/// All raw `(slot_obs.ts_local_us, drift_ms)` samples for a single validator
/// in the last `lookback_secs` seconds. Caller buckets / aggregates as needed.
pub async fn get_validator_history(
    pool: &Pool,
    validator: &str,
    lookback_secs: u64,
) -> Result<Vec<(i64, f64)>> {
    let cutoff_us =
        (chrono::Utc::now().timestamp() - lookback_secs as i64) * 1_000_000;
    let rows = sqlx::query(
        "SELECT s.ts_local_us AS ts_local_us, \
                (CAST(v.ts_chain AS REAL) * 1000.0) - (CAST(s.ts_local_us AS REAL) / 1000.0) AS drift_ms \
         FROM vote_records v \
         JOIN slot_obs s ON s.slot = v.slot \
         WHERE v.validator = ?1 AND s.ts_local_us >= ?2 \
         ORDER BY s.ts_local_us ASC",
    )
    .bind(validator)
    .bind(cutoff_us)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push((
            r.try_get::<i64, _>("ts_local_us")?,
            r.try_get::<f64, _>("drift_ms")?,
        ));
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

/// v1.0.0 Layer 1/Layer 2 framework — see methodology.html for context.
///
/// Tier 1: pipeline anomalies — slow vote pipeline, NOT clock drift.
///   * `500 <= |mean_drift_ms| < 5000` — elevated latency but bounded;
///     causes are network position, CPU saturation, geographic distance
///     from leaders, suboptimal Tachyon config. Operator should
///     investigate infra; not a chain-time threat.
///   * `n_samples >= 20` — same statistical floor as the legacy worst
///     ranking; we want early signal on infra issues.
pub async fn get_pipeline_anomalies(
    pool: &Pool,
    limit: i64,
) -> Result<Vec<ValidatorSummary>> {
    let rows = sqlx::query(
        "SELECT validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                cluster_id, cluster_size, is_multi_node, \
                is_foundation, foundation_label, severity \
         FROM validator_drift_summary \
         WHERE n_samples >= 20 \
           AND ABS(mean_drift_ms) >= 500 \
           AND ABS(mean_drift_ms) < 5000 \
         ORDER BY ABS(mean_drift_ms) DESC \
         LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows_to_summaries(rows))
}

/// v1.0.0 Layer 1/Layer 2 framework — see methodology.html for context.
///
/// Tier 2: Layer 2 clock drift — genuine NTP/chrony misconfiguration.
///   * `|mean_drift_ms| >= 5000` — pipeline contribution is bounded
///     ~400-850 ms; anything past 5 s reflects validator system clock
///     deviating from real UTC, not protocol latency.
///   * `n_samples >= 20` — same floor as Tier 1.
///
/// This is the regime Strontium oracle corrects for chain consumers.
pub async fn get_clock_drift_validators(
    pool: &Pool,
    limit: i64,
) -> Result<Vec<ValidatorSummary>> {
    let rows = sqlx::query(
        "SELECT validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                cluster_id, cluster_size, is_multi_node, \
                is_foundation, foundation_label, severity \
         FROM validator_drift_summary \
         WHERE n_samples >= 20 \
           AND ABS(mean_drift_ms) >= 5000 \
         ORDER BY ABS(mean_drift_ms) DESC \
         LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows_to_summaries(rows))
}

/// Shared row->ValidatorSummary mapping for the worst-tier queries.
/// Identical column list across get_pipeline_anomalies and
/// get_clock_drift_validators (and the legacy get_worst_validators); kept
/// as a free function rather than copy-pasted three times.
fn rows_to_summaries(rows: Vec<sqlx::sqlite::SqliteRow>) -> Vec<ValidatorSummary> {
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(ValidatorSummary {
            validator: r.try_get("validator").unwrap_or_default(),
            n_samples: r.try_get("n_samples").unwrap_or(0),
            mean_drift_ms: r.try_get("mean_drift_ms").unwrap_or(0.0),
            median_drift_ms: r.try_get("median_drift_ms").unwrap_or(0.0),
            stddev_drift_ms: r.try_get("stddev_drift_ms").unwrap_or(0.0),
            p10_drift_ms: r.try_get("p10_drift_ms").unwrap_or(0.0),
            p90_drift_ms: r.try_get("p90_drift_ms").unwrap_or(0.0),
            last_seen_slot: r.try_get("last_seen_slot").unwrap_or(0),
            last_stake_lamports: r.try_get("last_stake_lamports").unwrap_or(0),
            updated_at: r.try_get("updated_at").unwrap_or(0),
            cluster_id: r.try_get::<Option<i64>, _>("cluster_id").unwrap_or(None),
            cluster_size: r.try_get("cluster_size").unwrap_or(1),
            is_multi_node: r.try_get::<i64, _>("is_multi_node").unwrap_or(0) != 0,
            is_foundation: r.try_get::<i64, _>("is_foundation").unwrap_or(0) != 0,
            foundation_label: r.try_get::<Option<String>, _>("foundation_label").unwrap_or(None),
            severity: r.try_get::<Option<String>, _>("severity").unwrap_or(None),
        });
    }
    out
}

/// Legacy combined "top worst" ranking — kept for one release so external
/// consumers fetching `worst_validators.json` don't 404. Use
/// [`get_pipeline_anomalies`] (Tier 1, slow pipeline) and
/// [`get_clock_drift_validators`] (Tier 2, real Layer 2 drift) instead.
///
/// Scheduled for removal in v1.1.0 (release notes track this).
#[deprecated(
    since = "1.0.0",
    note = "Split into get_pipeline_anomalies (Tier 1) + get_clock_drift_validators (Tier 2). \
            Scheduled for removal in v1.1.0."
)]
pub async fn get_worst_validators(
    pool: &Pool,
    limit: i64,
    min_samples: i64,
    min_abs_drift_ms: f64,
) -> Result<Vec<ValidatorSummary>> {
    let rows = sqlx::query(
        "SELECT validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                cluster_id, cluster_size, is_multi_node, \
                is_foundation, foundation_label, severity \
         FROM validator_drift_summary \
         WHERE n_samples >= ?1 \
           AND ABS(mean_drift_ms) >= ?2 \
         ORDER BY ABS(mean_drift_ms) DESC \
         LIMIT ?3",
    )
    .bind(min_samples)
    .bind(min_abs_drift_ms)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows_to_summaries(rows))
}

/// One time-bucketed slice of foundation cluster drift over time.
/// `bucket_ts_secs` is unix-second time of bucket start (frontend
/// converts to ms for `Date()`). Stddev computed in Rust over raw
/// samples — SQLite has no native stddev, and the alternative subquery
/// shape is "gnarly".
#[derive(Debug, Clone)]
pub struct FoundationDriftBucket {
    pub bucket_ts_secs: i64,
    pub avg_drift_ms: f64,
    pub min_drift_ms: f64,
    pub max_drift_ms: f64,
    pub stddev_drift_ms: f64,
    pub nodes_active: i64,
    pub n_samples: i64,
}

/// Time-bucketed drift trend across X1 Labs Foundation validators.
/// Returns one bucket per `bucket_minutes` interval going back `days`
/// days. Bucket aggregates: avg/min/max/stddev of drift, count of
/// distinct foundation nodes contributing, total sample count.
///
/// SQL fetches raw `(ts_local_us, validator, drift_ms)` rows for the
/// foundation set and time window; the Rust side groups them into
/// buckets and computes statistics. For 12 nodes × 14 days × ~600 RPC
/// samples/day this is at most ~100k rows — trivially fast in Rust.
pub async fn get_foundation_drift_history(
    pool: &Pool,
    days: u32,
    bucket_minutes: u32,
) -> Result<Vec<FoundationDriftBucket>> {
    use std::collections::BTreeMap;

    let cutoff_us = (chrono::Utc::now().timestamp() - (days as i64) * 86400) * 1_000_000;
    let bucket_secs = (bucket_minutes as i64).max(1) * 60;

    let rows = sqlx::query(
        "SELECT s.ts_local_us AS ts_local_us, \
                v.validator AS validator, \
                (CAST(v.ts_chain AS REAL) * 1000.0) - (CAST(s.ts_local_us AS REAL) / 1000.0) AS drift_ms \
         FROM vote_records v \
         JOIN slot_obs s ON s.slot = v.slot \
         WHERE v.validator IN (SELECT validator FROM validator_drift_summary WHERE is_foundation = 1) \
           AND s.ts_local_us >= ?1 \
         ORDER BY s.ts_local_us ASC",
    )
    .bind(cutoff_us)
    .fetch_all(pool)
    .await?;

    let mut buckets: BTreeMap<i64, BucketAccum> = BTreeMap::new();
    for r in rows {
        let ts_us: i64 = r.try_get("ts_local_us")?;
        let validator: String = r.try_get("validator")?;
        let drift: f64 = r.try_get("drift_ms")?;
        let bucket_ts = (ts_us / 1_000_000 / bucket_secs) * bucket_secs;
        buckets.entry(bucket_ts).or_default().add(validator, drift);
    }

    let mut out: Vec<FoundationDriftBucket> = buckets
        .into_iter()
        .map(|(bucket_ts_secs, acc)| FoundationDriftBucket {
            bucket_ts_secs,
            avg_drift_ms: acc.mean(),
            min_drift_ms: acc.min(),
            max_drift_ms: acc.max(),
            stddev_drift_ms: acc.stddev(),
            nodes_active: acc.nodes.len() as i64,
            n_samples: acc.values.len() as i64,
        })
        .collect();
    out.sort_by_key(|b| b.bucket_ts_secs);
    Ok(out)
}

/// Per-bucket accumulator for foundation drift trend computation.
#[derive(Default)]
struct BucketAccum {
    values: Vec<f64>,
    nodes: std::collections::HashSet<String>,
}

impl BucketAccum {
    fn add(&mut self, validator: String, drift: f64) {
        self.values.push(drift);
        self.nodes.insert(validator);
    }
    fn mean(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }
    }
    fn min(&self) -> f64 {
        self.values.iter().copied().fold(f64::INFINITY, f64::min)
    }
    fn max(&self) -> f64 {
        self.values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
    }
    fn stddev(&self) -> f64 {
        let n = self.values.len();
        if n < 2 {
            return 0.0;
        }
        let m = self.mean();
        let var = self
            .values
            .iter()
            .map(|v| (v - m).powi(2))
            .sum::<f64>()
            / (n as f64 - 1.0);
        var.sqrt()
    }
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
    sqlx::query("DELETE FROM chrony_tracking_history WHERE bucket_ts < ?1")
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

    /// Cluster detection: 3 validators sharing rounded (mean, stddev, p10)
    /// signature get one cluster_id; isolated validators stay singletons.
    #[tokio::test]
    async fn detect_clusters_groups_three_or_more() {
        let pool = init(":memory:").await.unwrap();
        let now = chrono::Utc::now().timestamp();
        let rows = [
            // Cluster A — 3 members at (-883, 298, -1151).
            // Avoid p10 == ±0.5 increments (SQLite ROUND uses banker's
            // rounding so e.g. -1150.5 -> -1150, splitting the group).
            ("A1", 50i64, -883.4, 298.2, -1150.8),
            ("A2", 50, -883.1, 297.9, -1151.2),
            ("A3", 50, -882.7, 298.4, -1150.7),
            // Cluster B — 4 members at (10, 5, 5)
            ("B1", 50, 10.0, 5.0, 5.0),
            ("B2", 50, 10.1, 5.1, 5.0),
            ("B3", 50, 9.9, 4.9, 5.0),
            ("B4", 50, 10.2, 5.0, 5.0),
            // Singletons
            ("S1", 50, 100.0, 50.0, 50.0),
            ("S2", 50, -42.0, 10.0, -52.0),
            // Pair (size=2) — should NOT be a cluster
            ("P1", 50, 200.0, 30.0, 170.0),
            ("P2", 50, 200.1, 30.0, 170.0),
        ];
        for (v, n, m, sd, p10) in rows.iter() {
            sqlx::query(
                "INSERT INTO validator_drift_summary \
                 (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                  p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at) \
                 VALUES (?1, ?2, ?3, ?3, ?4, ?5, ?5, 0, 0, ?6)",
            )
            .bind(*v)
            .bind(*n)
            .bind(*m)
            .bind(*sd)
            .bind(*p10)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();
        }

        detect_and_assign_clusters(&pool).await.unwrap();
        let summaries = fetch_validator_summaries(&pool).await.unwrap();
        let by_v: std::collections::HashMap<&str, &ValidatorSummary> =
            summaries.iter().map(|s| (s.validator.as_str(), s)).collect();

        // Cluster B (size 4) should be cluster_id=1 (largest); cluster A is 2.
        let b1 = by_v["B1"];
        assert_eq!(b1.cluster_size, 4);
        assert!(b1.is_multi_node);
        assert_eq!(b1.cluster_id, Some(1));
        for v in &["B1", "B2", "B3", "B4"] {
            assert_eq!(by_v[v].cluster_id, Some(1));
        }

        let a1 = by_v["A1"];
        assert_eq!(a1.cluster_size, 3);
        assert!(a1.is_multi_node);
        assert_eq!(a1.cluster_id, Some(2));
        for v in &["A1", "A2", "A3"] {
            assert_eq!(by_v[v].cluster_id, Some(2));
        }

        // Singletons + size-2 pair are NOT in clusters.
        for v in &["S1", "S2", "P1", "P2"] {
            let row = by_v[v];
            assert_eq!(row.cluster_id, None, "{v} should be singleton");
            assert!(!row.is_multi_node);
            assert_eq!(row.cluster_size, 1);
        }
    }

    /// `accumulate_chrony_history` running mean: two samples in the same
    /// bucket should average correctly.
    #[tokio::test]
    async fn chrony_running_mean() {
        let pool = init(":memory:").await.unwrap();
        let bucket = 1_777_400_000i64;
        accumulate_chrony_history(&pool, bucket, 100.0, 10.0).await.unwrap();
        accumulate_chrony_history(&pool, bucket, 200.0, 30.0).await.unwrap();
        accumulate_chrony_history(&pool, bucket, 300.0, 50.0).await.unwrap();

        let map = fetch_chrony_history_map(&pool, 0).await.unwrap();
        let avg = *map.get(&bucket).unwrap();
        assert!((avg - 200.0).abs() < 1e-9, "expected 200, got {avg}");

        let row = sqlx::query("SELECT sample_count, avg_rms_offset_us FROM chrony_tracking_history WHERE bucket_ts = ?1")
            .bind(bucket)
            .fetch_one(&pool).await.unwrap();
        let count: i64 = row.try_get("sample_count").unwrap();
        let avg_rms: f64 = row.try_get("avg_rms_offset_us").unwrap();
        assert_eq!(count, 3);
        assert!((avg_rms - 30.0).abs() < 1e-9, "expected 30, got {avg_rms}");
    }

    /// `get_validator_history` returns drift samples joined with slot_obs.
    #[tokio::test]
    async fn validator_history_join_works() {
        let pool = init(":memory:").await.unwrap();
        let now_us = chrono::Utc::now().timestamp_micros();
        let now_s = now_us / 1_000_000;

        record_slot_obs(&pool, 100, now_us - 2_000_000).await.unwrap();
        record_slot_obs(&pool, 101, now_us - 1_000_000).await.unwrap();
        record_slot_obs(&pool, 102, now_us).await.unwrap();
        let votes = vec![
            VoteRecord { validator: "TARGET".into(), slot_voted: 100, ts_chain: now_s - 2 },
            VoteRecord { validator: "TARGET".into(), slot_voted: 101, ts_chain: now_s - 1 },
            VoteRecord { validator: "OTHER".into(),  slot_voted: 102, ts_chain: now_s },
        ];
        record_votes(&pool, &votes, 200).await.unwrap();

        let target_history = get_validator_history(&pool, "TARGET", 24 * 3600).await.unwrap();
        assert_eq!(target_history.len(), 2, "TARGET should have 2 samples");
        assert!(target_history[0].0 < target_history[1].0, "should be ASC by ts");

        let other_history = get_validator_history(&pool, "OTHER", 24 * 3600).await.unwrap();
        assert_eq!(other_history.len(), 1);

        let ghost = get_validator_history(&pool, "GHOST", 24 * 3600).await.unwrap();
        assert!(ghost.is_empty());
    }

    /// Regression guard for the v0.2.0 → v0.3.0 production migration.
    /// Builds a SQLite database with the exact v0.2.0 schema (no cluster_id
    /// columns, no chrony_tracking_history table), inserts representative
    /// data, then calls `init()` and asserts:
    ///   1. `init()` succeeds (no "no such column: cluster_id" error).
    ///   2. All pre-existing rows are preserved.
    ///   3. New columns and tables are present after migration.
    ///   4. Existing rows have correct default values for new columns.
    ///
    /// This test would have caught the v0.3.0 deployment failure on
    /// Sentinel where `CREATE INDEX ... ON ... (cluster_id)` in SCHEMA
    /// ran before `migrate_v3` had ALTER-added the column.
    #[tokio::test]
    async fn migration_v2_to_v3_preserves_data() {
        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

        let path = std::env::temp_dir().join(format!(
            "x1cd_migration_test_{}_{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path_str = path.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&path);

        // === Phase 1: build a real v0.2.0 schema (verbatim from before
        // v0.3.0 added cluster_* columns and chrony_tracking_history).
        const V2_SCHEMA: &str = r#"
            CREATE TABLE slot_obs (slot INTEGER PRIMARY KEY, ts_local_us INTEGER NOT NULL);
            CREATE INDEX idx_slot_obs_ts ON slot_obs(ts_local_us);

            CREATE TABLE vote_records (
                slot INTEGER NOT NULL,
                block_slot INTEGER NOT NULL,
                validator TEXT NOT NULL,
                ts_chain INTEGER NOT NULL,
                PRIMARY KEY (slot, validator, block_slot)
            );
            CREATE INDEX idx_vote_records_validator ON vote_records(validator);
            CREATE INDEX idx_vote_records_slot ON vote_records(slot);

            CREATE TABLE stake_snap (
                snapshot_ts INTEGER NOT NULL,
                validator TEXT NOT NULL,
                stake_lamports INTEGER NOT NULL,
                PRIMARY KEY (snapshot_ts, validator)
            );

            CREATE TABLE validator_drift_summary (
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

            CREATE TABLE network_drift_history (
                bucket_ts INTEGER PRIMARY KEY,
                median_drift_ms REAL NOT NULL,
                mean_drift_ms REAL NOT NULL,
                stake_weighted_drift_ms REAL NOT NULL,
                n_validators INTEGER NOT NULL,
                n_samples INTEGER NOT NULL
            );

            CREATE TABLE error_log (
                ts INTEGER NOT NULL,
                source TEXT NOT NULL,
                message TEXT NOT NULL
            );
            CREATE INDEX idx_error_log_ts ON error_log(ts);

            CREATE TABLE chrony_tracking (
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

            CREATE TABLE chrony_sources (
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

        // Build the v0.2.0 DB on the target path, then close it before
        // calling init() — mimics a daemon restart against an existing DB.
        {
            let opts = SqliteConnectOptions::from_str(&format!("sqlite://{path_str}"))
                .unwrap()
                .create_if_missing(true);
            let setup_pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(opts)
                .await
                .unwrap();

            for stmt in V2_SCHEMA.split(';') {
                let s = stmt.trim();
                if !s.is_empty() {
                    sqlx::query(s).execute(&setup_pool).await.unwrap();
                }
            }

            // Representative data simulating ~24h of v0.2.0 production.
            for slot in 100i64..110 {
                sqlx::query("INSERT INTO slot_obs (slot, ts_local_us) VALUES (?1, ?2)")
                    .bind(slot)
                    .bind(1_700_000_000_000_000i64 + slot * 400_000)
                    .execute(&setup_pool)
                    .await
                    .unwrap();
            }
            for i in 0..5i64 {
                sqlx::query(
                    "INSERT INTO vote_records (slot, block_slot, validator, ts_chain) \
                     VALUES (?1, ?2, ?3, ?4)",
                )
                .bind(100 + i)
                .bind(200 + i)
                .bind(format!("VAL{i}"))
                .bind(1_700_000_000i64 + i)
                .execute(&setup_pool)
                .await
                .unwrap();
            }
            for i in 0..3i64 {
                sqlx::query(
                    "INSERT INTO validator_drift_summary \
                     (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                      p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at) \
                     VALUES (?1, 10, 100.0, 100.0, 5.0, 95.0, 105.0, 100, 1000000000, 1700000000)",
                )
                .bind(format!("VAL{i}"))
                .execute(&setup_pool)
                .await
                .unwrap();
            }
            for i in 0..2i64 {
                sqlx::query(
                    "INSERT INTO stake_snap (snapshot_ts, validator, stake_lamports) \
                     VALUES (1700000000, ?1, ?2)",
                )
                .bind(format!("VAL{i}"))
                .bind(1_000_000_000i64 * (i + 1))
                .execute(&setup_pool)
                .await
                .unwrap();
            }
            sqlx::query(
                "INSERT INTO chrony_tracking (id, updated_at, leap_status) \
                 VALUES (1, 1700000000, 'Normal')",
            )
            .execute(&setup_pool)
            .await
            .unwrap();

            setup_pool.close().await;
        }

        // === Phase 2: call init() — must NOT fail. The bug we are
        // guarding against here was: SCHEMA contained a CREATE INDEX on
        // a v0.3.0-only column, executed before migrate_v3 had a chance
        // to ALTER TABLE ADD it; SQLite rejected the index with
        // "no such column: cluster_id".
        let pool = init(&path_str).await.expect("init() must succeed on v0.2.0 schema");

        // === Phase 3: original rows preserved.
        let n_slots: i64 = sqlx::query("SELECT COUNT(*) AS n FROM slot_obs")
            .fetch_one(&pool).await.unwrap().try_get("n").unwrap();
        assert_eq!(n_slots, 10);
        let n_votes: i64 = sqlx::query("SELECT COUNT(*) AS n FROM vote_records")
            .fetch_one(&pool).await.unwrap().try_get("n").unwrap();
        assert_eq!(n_votes, 5);
        let n_summary: i64 = sqlx::query("SELECT COUNT(*) AS n FROM validator_drift_summary")
            .fetch_one(&pool).await.unwrap().try_get("n").unwrap();
        assert_eq!(n_summary, 3);
        let n_stake: i64 = sqlx::query("SELECT COUNT(*) AS n FROM stake_snap")
            .fetch_one(&pool).await.unwrap().try_get("n").unwrap();
        assert_eq!(n_stake, 2);
        let leap: String = sqlx::query("SELECT leap_status FROM chrony_tracking WHERE id = 1")
            .fetch_one(&pool).await.unwrap().try_get("leap_status").unwrap();
        assert_eq!(leap, "Normal");

        // === Phase 4: new columns exist on validator_drift_summary.
        let cols = sqlx::query("PRAGMA table_info(validator_drift_summary)")
            .fetch_all(&pool).await.unwrap();
        let names: Vec<String> = cols
            .iter()
            .map(|r| r.try_get::<String, _>("name").unwrap_or_default())
            .collect();
        for new_col in &["cluster_id", "cluster_size", "is_multi_node"] {
            assert!(
                names.iter().any(|c| c == *new_col),
                "missing column after migration: {new_col}"
            );
        }

        // === Phase 5: defaults applied to existing rows.
        let row = sqlx::query(
            "SELECT cluster_id, cluster_size, is_multi_node \
             FROM validator_drift_summary WHERE validator = 'VAL0'",
        )
        .fetch_one(&pool).await.unwrap();
        let cid: Option<i64> = row.try_get("cluster_id").unwrap_or(None);
        let csize: i64 = row.try_get("cluster_size").unwrap_or(-1);
        let imn: i64 = row.try_get("is_multi_node").unwrap_or(-1);
        assert_eq!(cid, None, "cluster_id should be NULL on existing rows");
        assert_eq!(csize, 1, "cluster_size should default to 1");
        assert_eq!(imn, 0, "is_multi_node should default to 0");

        // === Phase 6: new chrony_tracking_history table created and empty.
        let n_chrony_history: i64 =
            sqlx::query("SELECT COUNT(*) AS n FROM chrony_tracking_history")
                .fetch_one(&pool).await.unwrap().try_get("n").unwrap();
        assert_eq!(n_chrony_history, 0);

        // Cleanup.
        pool.close().await;
        let _ = std::fs::remove_file(&path);
    }

    /// Insert synthetic validator_drift_summary rows and verify
    /// `get_best_synced_validators` orders by absolute mean drift ascending,
    /// applies the min_samples filter, excludes foundation, applies the
    /// `ABS(drift) < 5000` defensive filter, and respects the limit.
    /// Updated for v0.6.0 signature (no min_stake_lamports).
    #[tokio::test]
    async fn get_best_synced_orders_and_filters() {
        let pool = init(":memory:").await.unwrap();
        let now = chrono::Utc::now().timestamp();

        // Rows: (validator, n_samples, mean_drift, stake_lamports, is_foundation)
        let rows = [
            ("BIG_DRIFT_HIGH_N",  100i64, 5000.0, 1_000_000_000i64, 0i64),
            ("TINY_DRIFT_LOW_N",  3,      0.5,    100_000_000,      0),
            ("TINY_DRIFT_HIGH_N", 50,     1.2,    500_000_000,      0),
            ("ZERO_DRIFT",        8,      0.0,    200_000_000,      0),
            ("NEGATIVE_TINY",     20,     -2.4,   300_000_000,      0),
            ("MEDIUM_DRIFT",      7,      50.0,   400_000_000,      0),
            // Foundation node — should be excluded even though tiny drift
            ("FOUNDATION_NODE",   100,    0.1,    50_000_000_000,   1),
        ];
        for (v, n, mean, stake, is_foundation) in rows.iter() {
            sqlx::query(
                "INSERT INTO validator_drift_summary \
                 (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                  p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                  is_foundation) \
                 VALUES (?1, ?2, ?3, ?3, 1.0, ?3, ?3, 0, ?4, ?5, ?6)",
            )
            .bind(*v)
            .bind(*n)
            .bind(*mean)
            .bind(*stake)
            .bind(now)
            .bind(*is_foundation)
            .execute(&pool)
            .await
            .unwrap();
        }

        // min_samples=5 — TINY_DRIFT_LOW_N (n=3) excluded,
        // FOUNDATION_NODE excluded by is_foundation filter,
        // BIG_DRIFT_HIGH_N excluded by |drift| < 5000 (mean is exactly 5000).
        // Top 3 by |mean|: ZERO_DRIFT (0), TINY_DRIFT_HIGH_N (1.2), NEGATIVE_TINY (2.4).
        let best = get_best_synced_validators(&pool, 3, 5).await.unwrap();
        assert_eq!(best.len(), 3);
        assert_eq!(best[0].validator, "ZERO_DRIFT");
        assert_eq!(best[1].validator, "TINY_DRIFT_HIGH_N");
        assert_eq!(best[2].validator, "NEGATIVE_TINY");
        // Foundation node should not appear regardless of how good its drift is.
        for v in &best {
            assert_ne!(v.validator, "FOUNDATION_NODE");
        }
        // BIG_DRIFT_HIGH_N must be filtered by abs<5000 even though it has 100 samples.
        let all_qualifying = get_best_synced_validators(&pool, 100, 5).await.unwrap();
        let pubkeys: Vec<&str> = all_qualifying.iter().map(|v| v.validator.as_str()).collect();
        assert!(
            !pubkeys.contains(&"BIG_DRIFT_HIGH_N"),
            "BIG_DRIFT_HIGH_N (|drift|=5000) should be filtered by abs<5000 check"
        );

        // limit=1 honoured
        let best_limit_1 = get_best_synced_validators(&pool, 1, 5).await.unwrap();
        assert_eq!(best_limit_1.len(), 1);
        assert_eq!(best_limit_1[0].validator, "ZERO_DRIFT");

        // min_samples=200 filters everyone out (none qualify)
        let none = get_best_synced_validators(&pool, 10, 200).await.unwrap();
        assert!(none.is_empty());
    }

    /// Regression guard for the v0.3.x → v0.4.0 production migration.
    /// Builds a SQLite database with the v0.3.x schema (cluster_* columns
    /// from migrate_v3, but no is_foundation/foundation_label/severity),
    /// inserts representative data including a known foundation pubkey,
    /// then calls `init()` and asserts:
    ///   1. `init()` succeeds.
    ///   2. New columns `is_foundation`, `foundation_label`, `severity` exist.
    ///   3. Foundation row gets `is_foundation = 1` + correct label.
    ///   4. Non-foundation row gets `is_foundation = 0`, NULL label.
    ///   5. `severity` is NULL initially (set by next recompute, not by migrate).
    ///   6. All pre-existing data (counts, values) preserved.
    #[tokio::test]
    async fn migration_v3_to_v4_preserves_data() {
        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

        let path = std::env::temp_dir().join(format!(
            "x1cd_migration_v4_test_{}_{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path_str = path.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&path);

        // v0.3.x schema: same as v0.2 + cluster_* columns + chrony_tracking_history.
        // No is_foundation, no foundation_label, no severity.
        const V3_SCHEMA: &str = r#"
            CREATE TABLE slot_obs (slot INTEGER PRIMARY KEY, ts_local_us INTEGER NOT NULL);
            CREATE INDEX idx_slot_obs_ts ON slot_obs(ts_local_us);
            CREATE TABLE vote_records (
                slot INTEGER NOT NULL,
                block_slot INTEGER NOT NULL,
                validator TEXT NOT NULL,
                ts_chain INTEGER NOT NULL,
                PRIMARY KEY (slot, validator, block_slot)
            );
            CREATE INDEX idx_vote_records_validator ON vote_records(validator);
            CREATE INDEX idx_vote_records_slot ON vote_records(slot);
            CREATE TABLE stake_snap (
                snapshot_ts INTEGER NOT NULL,
                validator TEXT NOT NULL,
                stake_lamports INTEGER NOT NULL,
                PRIMARY KEY (snapshot_ts, validator)
            );
            CREATE TABLE validator_drift_summary (
                validator TEXT PRIMARY KEY,
                n_samples INTEGER NOT NULL,
                mean_drift_ms REAL NOT NULL,
                median_drift_ms REAL NOT NULL,
                stddev_drift_ms REAL NOT NULL,
                p10_drift_ms REAL NOT NULL,
                p90_drift_ms REAL NOT NULL,
                last_seen_slot INTEGER NOT NULL,
                last_stake_lamports INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                cluster_id INTEGER,
                cluster_size INTEGER DEFAULT 1,
                is_multi_node INTEGER DEFAULT 0
            );
            CREATE INDEX idx_drift_summary_cluster ON validator_drift_summary(cluster_id);
            CREATE TABLE network_drift_history (
                bucket_ts INTEGER PRIMARY KEY,
                median_drift_ms REAL NOT NULL,
                mean_drift_ms REAL NOT NULL,
                stake_weighted_drift_ms REAL NOT NULL,
                n_validators INTEGER NOT NULL,
                n_samples INTEGER NOT NULL
            );
            CREATE TABLE error_log (
                ts INTEGER NOT NULL,
                source TEXT NOT NULL,
                message TEXT NOT NULL
            );
            CREATE INDEX idx_error_log_ts ON error_log(ts);
            CREATE TABLE chrony_tracking (
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
            CREATE TABLE chrony_sources (
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
            CREATE TABLE chrony_tracking_history (
                bucket_ts INTEGER PRIMARY KEY,
                avg_system_offset_us REAL NOT NULL,
                avg_rms_offset_us REAL NOT NULL,
                sample_count INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
        "#;

        // One known foundation pubkey + one regular validator.
        const FOUNDATION_PUBKEY: &str = "6Wf81YuCHu3j7xJupCq5mxDWz8seuNkybyT9riVm5FeA";

        {
            let opts = SqliteConnectOptions::from_str(&format!("sqlite://{path_str}"))
                .unwrap()
                .create_if_missing(true);
            let setup_pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(opts)
                .await
                .unwrap();

            for stmt in V3_SCHEMA.split(';') {
                let s = stmt.trim();
                if !s.is_empty() {
                    sqlx::query(s).execute(&setup_pool).await.unwrap();
                }
            }

            // Insert a foundation row + a regular row in v0.3.x format.
            sqlx::query(
                "INSERT INTO validator_drift_summary \
                 (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                  p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at) \
                 VALUES (?1, 156, -883.1, -990.0, 297.0, -1151.0, -555.0, 100, 58000000000000000, 1700000000)",
            )
            .bind(FOUNDATION_PUBKEY)
            .execute(&setup_pool).await.unwrap();
            sqlx::query(
                "INSERT INTO validator_drift_summary \
                 (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                  p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at) \
                 VALUES ('REGULAR_VALIDATOR_X', 50, 25.0, 25.0, 5.0, 18.0, 32.0, 99, 1000000000, 1700000000)",
            )
            .execute(&setup_pool).await.unwrap();

            // Some other v0.3.x data.
            sqlx::query("INSERT INTO slot_obs (slot, ts_local_us) VALUES (1, 1700000000000000)")
                .execute(&setup_pool).await.unwrap();
            sqlx::query("INSERT INTO chrony_tracking_history (bucket_ts, avg_system_offset_us, avg_rms_offset_us, sample_count, updated_at) VALUES (1700000000, -7.0, 14.0, 5, 1700000000)")
                .execute(&setup_pool).await.unwrap();

            setup_pool.close().await;
        }

        let pool = init(&path_str).await.expect("init() must succeed on v0.3.x schema");

        // Verify columns added.
        let cols = sqlx::query("PRAGMA table_info(validator_drift_summary)")
            .fetch_all(&pool).await.unwrap();
        let names: Vec<String> = cols
            .iter()
            .map(|r| r.try_get::<String, _>("name").unwrap_or_default())
            .collect();
        for c in &["is_foundation", "foundation_label", "severity"] {
            assert!(
                names.iter().any(|n| n == *c),
                "missing column after migration: {c}"
            );
        }

        // Verify foundation flag applied.
        let row = sqlx::query("SELECT is_foundation, foundation_label, severity FROM validator_drift_summary WHERE validator = ?1")
            .bind(FOUNDATION_PUBKEY)
            .fetch_one(&pool).await.unwrap();
        assert_eq!(row.try_get::<i64, _>("is_foundation").unwrap(), 1);
        assert_eq!(
            row.try_get::<String, _>("foundation_label").unwrap(),
            "X1 Labs (node8)",
        );
        // severity is set later (by recompute), not by migrate.
        let sev: Option<String> = row.try_get("severity").unwrap_or(None);
        assert!(sev.is_none(), "severity should be NULL until recompute runs");

        // Non-foundation row stays non-foundation, NULL label.
        let row2 = sqlx::query("SELECT is_foundation, foundation_label FROM validator_drift_summary WHERE validator = 'REGULAR_VALIDATOR_X'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(row2.try_get::<i64, _>("is_foundation").unwrap(), 0);
        let label2: Option<String> = row2.try_get("foundation_label").unwrap_or(None);
        assert!(label2.is_none());

        // Pre-existing data preserved.
        let n_summary: i64 = sqlx::query("SELECT COUNT(*) AS n FROM validator_drift_summary")
            .fetch_one(&pool).await.unwrap().try_get("n").unwrap();
        assert_eq!(n_summary, 2);
        let n_chrony_history: i64 = sqlx::query("SELECT COUNT(*) AS n FROM chrony_tracking_history")
            .fetch_one(&pool).await.unwrap().try_get("n").unwrap();
        assert_eq!(n_chrony_history, 1);

        // classify_severity helper sanity (v0.4.1 — no foundation arg).
        assert_eq!(classify_severity(2, 0.0), None);
        assert_eq!(classify_severity(10, 6000.0), Some("critical"));
        assert_eq!(classify_severity(10, 2000.0), Some("high"));
        assert_eq!(classify_severity(10, 100.0), Some("healthy"));

        pool.close().await;
        let _ = std::fs::remove_file(&path);
    }

    /// v0.6.0: stake-based filter is REMOVED from best-synced. A 2-XNT
    /// validator with NTP-discipline (low |drift|, ≥100 samples, not
    /// foundation) MUST appear in best-synced. Filters retained are
    /// purely statistical/structural: n_samples ≥ 100, is_foundation = 0,
    /// |drift| < 5000 ms.
    #[tokio::test]
    async fn test_best_synced_includes_low_stake_with_good_clock() {
        let pool = init(":memory:").await.unwrap();
        let now = chrono::Utc::now().timestamp();

        // (validator, n_samples, mean_drift_ms, stake_lamports, is_foundation)
        let rows = [
            // Accepted: 2 XNT but excellent clock + 200 samples
            ("Tiny_stake_great_clock", 200i64, 0.3, 2_000_000_000i64, 0i64),
            // Accepted: meets all thresholds (large stake — also fine)
            ("Real_high_stake", 200, 50.0, 1_500_000_000_000, 0),
            // Rejected: foundation, even with great drift
            ("Foundation_node", 500, 1.0, 55_000_000_000_000, 1),
            // Rejected: |drift|=8000 >= 5000 (pathological)
            ("Pathological_drift", 200, 8000.0, 5_000_000_000_000, 0),
            // Rejected: n=43 < 100 (insufficient statistical power)
            ("Low_n_samples", 43, 0.1, 100_000_000_000_000, 0),
        ];
        for (v, n, mean, stake, is_foundation) in rows.iter() {
            sqlx::query(
                "INSERT INTO validator_drift_summary \
                 (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                  p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                  is_foundation) \
                 VALUES (?1, ?2, ?3, ?3, 1.0, ?3, ?3, 0, ?4, ?5, ?6)",
            )
            .bind(*v)
            .bind(*n)
            .bind(*mean)
            .bind(*stake)
            .bind(now)
            .bind(*is_foundation)
            .execute(&pool)
            .await
            .unwrap();
        }

        let best = get_best_synced_validators(&pool, 10, 100).await.unwrap();
        let pubkeys: Vec<&str> = best.iter().map(|v| v.validator.as_str()).collect();

        assert_eq!(best.len(), 2, "expected 2 — both stake levels pass when clock + samples qualify");
        // Tiny_stake_great_clock has |drift|=0.3 — beats Real_high_stake (|drift|=50)
        assert_eq!(best[0].validator, "Tiny_stake_great_clock");
        assert_eq!(best[1].validator, "Real_high_stake");
        assert!(!pubkeys.contains(&"Foundation_node"));
        assert!(!pubkeys.contains(&"Pathological_drift"));
        assert!(!pubkeys.contains(&"Low_n_samples"));
    }

    /// v0.6.0: stake-based filter is REMOVED from worst-validators.
    /// A 2-XNT validator with -23s drift is operationally newsworthy
    /// regardless of stake — catastrophic operator misconfig is signal.
    /// Filters retained: n_samples ≥ 20, |drift| ≥ 500 ms.
    ///
    /// v1.0.0: this exercises the legacy combined query (deprecated;
    /// removal scheduled for v1.1.0). The two new tier-specific tests
    /// below cover the active code paths.
    #[allow(deprecated)]
    #[tokio::test]
    async fn test_worst_validators_includes_low_stake_with_bad_clock() {
        let pool = init(":memory:").await.unwrap();
        let now = chrono::Utc::now().timestamp();

        // (validator, n_samples, mean_drift_ms, stake_lamports)
        let rows = [
            // Rejected: n=5 < 20 (statistical noise)
            ("tiny_n_spam", 5i64, 50_000.0, 1_000_000_000_000i64),
            // Accepted: -23s on a 2-XNT validator (still newsworthy)
            ("tiny_stake_disaster", 100, -23_000.0, 2_000_000_000),
            // Accepted: -23s on a real validator
            ("real_outlier_a", 100, -23_000.0, 100_000_000_000_000),
            // Accepted: +11s on a real validator
            ("real_outlier_b", 100, 11_000.0, 110_000_000_000_000),
            // Rejected: |drift|=100 < 500 (healthy)
            ("healthy", 100, 100.0, 50_000_000_000_000),
            // Accepted: |drift|=800 ≥ 500
            ("moderate", 50, 800.0, 50_000_000_000_000),
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

        let results = get_worst_validators(&pool, 30, 20, 500.0).await.unwrap();
        let pubkeys: Vec<&str> = results.iter().map(|v| v.validator.as_str()).collect();

        assert_eq!(
            results.len(),
            4,
            "expected 4 (tiny_stake_disaster + real_outlier_a + real_outlier_b + moderate); \
             tiny_n_spam (n<20) and healthy (|drift|<500) filtered"
        );
        // ABS(drift) DESC: 23000 (real_a or tiny_stake_disaster), 23000, 11000, 800
        assert!(pubkeys.contains(&"tiny_stake_disaster"));
        assert!(pubkeys.contains(&"real_outlier_a"));
        assert!(pubkeys.contains(&"real_outlier_b"));
        assert!(pubkeys.contains(&"moderate"));
        // Last by ABS(drift) must be moderate (800)
        assert_eq!(results[3].validator, "moderate");
        // First two are the |23000| pair
        assert_eq!(results[0].mean_drift_ms.abs() as i64, 23_000);
        assert_eq!(results[1].mean_drift_ms.abs() as i64, 23_000);
        // Third is real_outlier_b (11000)
        assert_eq!(results[2].validator, "real_outlier_b");
    }

    /// v1.0.0 Tier 1: pipeline anomalies — slow vote pipeline, NOT clock
    /// drift. Must include 500 ≤ |lag| < 5000 ms entries; must EXCLUDE
    /// |drift| ≥ 5000 ms (those go to Tier 2).
    #[tokio::test]
    async fn test_pipeline_anomalies_excludes_clock_drift_outliers() {
        let pool = init(":memory:").await.unwrap();
        let now = chrono::Utc::now().timestamp();

        // (validator, n_samples, mean_drift_ms, stake_lamports)
        let rows = [
            // Accepted Tier 1: -2s lag (slow pipeline / network / CPU)
            ("slow_network", 100i64, -2000.0, 50_000_000_000i64),
            // Rejected from Tier 1: -23s — Layer 2, belongs to Tier 2
            ("clock_drift_op", 100, -23_000.0, 100_000_000_000_000),
            // Rejected: |drift|=300 < 500 (healthy / normal pipeline)
            ("healthy", 100, -300.0, 50_000_000_000_000),
            // Rejected: n=10 < 20 (statistical noise)
            ("noisy_low_n", 10, -2000.0, 1_000_000_000),
            // Accepted Tier 1: +1.2s
            ("moderate_slow", 50, 1_200.0, 30_000_000_000_000),
            // Boundary: |drift|=4999.9 just under 5000 → Tier 1
            ("boundary_high", 50, 4_999.9, 1_000_000_000),
            // Boundary: |drift|=5000.0 → Tier 2 (excluded from Tier 1)
            ("boundary_layer2", 50, 5_000.0, 1_000_000_000),
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

        let results = get_pipeline_anomalies(&pool, 30).await.unwrap();
        let pubkeys: Vec<&str> = results.iter().map(|v| v.validator.as_str()).collect();

        assert_eq!(
            results.len(),
            3,
            "expected 3 (slow_network + moderate_slow + boundary_high); \
             clock_drift_op (Tier 2) + healthy + noisy_low_n + boundary_layer2 filtered"
        );
        assert!(pubkeys.contains(&"slow_network"));
        assert!(pubkeys.contains(&"moderate_slow"));
        assert!(pubkeys.contains(&"boundary_high"));
        assert!(!pubkeys.contains(&"clock_drift_op"));
        assert!(!pubkeys.contains(&"boundary_layer2"));
        // Sorted by ABS(drift) DESC: 4999.9, 2000, 1200.
        assert_eq!(results[0].validator, "boundary_high");
        assert_eq!(results[1].validator, "slow_network");
        assert_eq!(results[2].validator, "moderate_slow");
    }

    /// v1.0.0 Tier 2: Layer 2 clock drift — only |drift| ≥ 5000 ms.
    /// Pipeline-anomaly entries (500 ≤ |lag| < 5000) must NOT appear.
    #[tokio::test]
    async fn test_clock_drift_validators_only_layer2() {
        let pool = init(":memory:").await.unwrap();
        let now = chrono::Utc::now().timestamp();

        // (validator, n_samples, mean_drift_ms, stake_lamports)
        let rows = [
            // Rejected: -2s is Tier 1, not Tier 2
            ("slow_pipeline", 100i64, -2000.0, 50_000_000_000i64),
            // Accepted: -23s real Layer 2 drift
            ("real_drift", 100, -23_000.0, 100_000_000_000_000),
            // Accepted: +11s real Layer 2 drift
            ("real_drift_pos", 100, 11_000.0, 110_000_000_000_000),
            // Rejected: |drift|=300 < 5000 — normal/healthy
            ("healthy", 100, -300.0, 50_000_000_000_000),
            // Rejected: n=10 < 20 (insufficient samples)
            ("low_n_huge_drift", 10, -50_000.0, 1_000_000_000),
            // Accepted: 2-XNT validator with catastrophic clock drift
            ("tiny_stake_disaster", 100, -42_000.0, 2_000_000_000),
            // Boundary: 5000.0 included in Tier 2.
            ("boundary_layer2", 50, 5_000.0, 1_000_000_000),
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

        let results = get_clock_drift_validators(&pool, 30).await.unwrap();
        let pubkeys: Vec<&str> = results.iter().map(|v| v.validator.as_str()).collect();

        assert_eq!(
            results.len(),
            4,
            "expected 4 (real_drift + real_drift_pos + tiny_stake_disaster + boundary_layer2); \
             slow_pipeline + healthy + low_n_huge_drift filtered"
        );
        assert!(pubkeys.contains(&"real_drift"));
        assert!(pubkeys.contains(&"real_drift_pos"));
        assert!(pubkeys.contains(&"tiny_stake_disaster"));
        assert!(pubkeys.contains(&"boundary_layer2"));
        // Sorted by ABS(drift) DESC.
        assert_eq!(results[0].validator, "tiny_stake_disaster"); // 42000
        assert_eq!(results[1].validator, "real_drift");          // 23000
        assert_eq!(results[2].validator, "real_drift_pos");      // 11000
        assert_eq!(results[3].validator, "boundary_layer2");     // 5000
    }

    /// v0.5.0: foundation drift trend bucketing — verifies the JOIN
    /// filters by is_foundation, buckets by `bucket_minutes`, and
    /// computes avg/min/max/nodes_active correctly.
    #[tokio::test]
    async fn test_foundation_drift_trend_buckets_correctly() {
        let pool = init(":memory:").await.unwrap();
        let now_us = chrono::Utc::now().timestamp_micros();
        let now_s = now_us / 1_000_000;

        // Two foundation validators + one normal validator. Normal must
        // not appear in any bucket.
        let f1 = "FOUND_NODE_A_111111111111111111111111111111";
        let f2 = "FOUND_NODE_B_222222222222222222222222222222";
        let normal = "NORMAL_VALIDATOR_3333333333333333333333333333";
        for (pk, is_foundation) in [(f1, 1i64), (f2, 1), (normal, 0)] {
            sqlx::query(
                "INSERT INTO validator_drift_summary \
                 (validator, n_samples, mean_drift_ms, median_drift_ms, stddev_drift_ms, \
                  p10_drift_ms, p90_drift_ms, last_seen_slot, last_stake_lamports, updated_at, \
                  is_foundation) \
                 VALUES (?1, 10, 0.0, 0.0, 1.0, 0.0, 0.0, 0, 0, ?2, ?3)",
            )
            .bind(pk)
            .bind(now_s)
            .bind(is_foundation)
            .execute(&pool)
            .await
            .unwrap();
        }

        // Choose two buckets safely separated: 6h ago and now.
        // A bucket key is `(ts_local_us / 1_000_000 / 3600) * 3600` so
        // distinct ts_local values 6h apart produce distinct buckets.
        // For drift = ts_chain*1000 - ts_local_us/1000 to equal target,
        // pick ts_local_us = ts_chain*1_000_000 - drift*1000.
        let bucket_a_secs = (now_us - 6 * 3600 * 1_000_000) / 1_000_000;
        let bucket_b_secs = now_us / 1_000_000;

        let inserts: [(i64, &str, f64, i64); 4] = [
            (bucket_a_secs, f1, -890.0, 100),
            (bucket_a_secs, f2, -880.0, 101),
            (bucket_b_secs, f1, -875.0, 200),
            (bucket_b_secs, f2, -885.0, 201),
        ];
        for (bucket_secs, validator, drift_ms, slot) in inserts.iter() {
            let ts_chain = *bucket_secs;
            let ts_local_us = bucket_secs * 1_000_000 - (*drift_ms * 1000.0) as i64;
            sqlx::query("INSERT INTO slot_obs (slot, ts_local_us) VALUES (?1, ?2)")
                .bind(*slot)
                .bind(ts_local_us)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query(
                "INSERT INTO vote_records (slot, block_slot, validator, ts_chain) \
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(*slot)
            .bind(*slot + 1)
            .bind(*validator)
            .bind(ts_chain)
            .execute(&pool)
            .await
            .unwrap();
        }
        // A normal-validator vote in the latest bucket — must NOT appear
        // in foundation trend.
        let normal_slot = 300i64;
        let normal_ts_local = bucket_b_secs * 1_000_000 - (-100.0_f64 * 1000.0) as i64;
        sqlx::query("INSERT INTO slot_obs (slot, ts_local_us) VALUES (?1, ?2)")
            .bind(normal_slot)
            .bind(normal_ts_local)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO vote_records (slot, block_slot, validator, ts_chain) \
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(normal_slot)
        .bind(normal_slot + 1)
        .bind(normal)
        .bind(bucket_b_secs)
        .execute(&pool)
        .await
        .unwrap();

        let history = get_foundation_drift_history(&pool, 1, 60).await.unwrap();
        assert!(
            history.len() >= 2,
            "expected ≥2 buckets (6h ago + now), got {}",
            history.len()
        );

        // Both buckets contain exactly the 2 foundation nodes — normal
        // validator must not contribute.
        for bucket in &history {
            assert_eq!(bucket.nodes_active, 2, "every bucket should have 2 distinct foundation nodes");
            assert_eq!(bucket.n_samples, 2);
        }

        // Most recent bucket: drifts -875 and -885 → avg -880, min -885,
        // max -875. (Allow ±2ms slack for integer rounding in ts_local_us.)
        let last = history.last().unwrap();
        assert!(
            (last.avg_drift_ms - (-880.0)).abs() < 2.0,
            "expected avg ~-880, got {}",
            last.avg_drift_ms
        );
        assert!((last.min_drift_ms - (-885.0)).abs() < 2.0);
        assert!((last.max_drift_ms - (-875.0)).abs() < 2.0);
        // 2 samples → stddev with (n-1) denom = sqrt(((10/2)*2)/1) = sqrt(50) ~ 7.07
        assert!(last.stddev_drift_ms > 0.0, "stddev should be non-zero with 2 distinct values");
    }

    /// v0.4.1: foundation status no longer hides drift severity.
    /// A foundation node with critical drift must still be classified
    /// `critical` so it shows up red on the dashboard.
    #[test]
    fn foundation_node_with_high_drift_classified_correctly() {
        // -883 ms is the production X1 Labs baseline → "healthy".
        assert_eq!(classify_severity(100, -883.0), Some("healthy"));
        // -5500 ms — even on a foundation node — must be "critical".
        assert_eq!(classify_severity(100, -5500.0), Some("critical"));
        assert_eq!(classify_severity(100, 5500.0), Some("critical"));
        // -1500 ms → "high" regardless of foundation status.
        assert_eq!(classify_severity(100, -1500.0), Some("high"));
        // Boundary: exactly 1s and exactly 5s.
        assert_eq!(classify_severity(100, 1000.0), Some("healthy"));  // not >1000
        assert_eq!(classify_severity(100, 1000.1), Some("high"));
        assert_eq!(classify_severity(100, 5000.0), Some("high"));     // not >5000
        assert_eq!(classify_severity(100, 5000.1), Some("critical"));
    }
}
