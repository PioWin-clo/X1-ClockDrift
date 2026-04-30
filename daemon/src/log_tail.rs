use crate::db::{self, Pool};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

/// Pattern matches:
///   [2026-04-28T07:31:24.442703241Z INFO  solana_runtime::bank] bank frozen: 46131510 hash: ...
const PATTERN: &str = r"^\[(?P<ts>[^\s]+Z)\s+INFO\s+solana_runtime::bank\] bank frozen: (?P<slot>\d+) hash:";

/// How often we re-emit the "circular truncate" info line. The validator log
/// rolls thousands of times per minute; without throttling this would flood
/// the journal.
const TRUNCATE_LOG_THROTTLE_SECS: u64 = 60;
const POLL_INTERVAL_MS: u64 = 200;

pub async fn run(log_path: String, pool: Pool, shutdown: CancellationToken) -> Result<()> {
    let re = Regex::new(PATTERN).context("invalid log pattern regex")?;
    tracing::info!(path = %log_path, "log_tail starting");

    loop {
        if shutdown.is_cancelled() {
            return Ok(());
        }
        match tail_once(&log_path, &re, &pool, &shutdown).await {
            Ok(_) => {
                // Reachable when the file's inode changed under us
                // (real rotation / replacement) OR shutdown was requested.
                if shutdown.is_cancelled() {
                    tracing::info!("log_tail shutting down");
                    return Ok(());
                }
                tracing::info!("log_tail reopening after rotation in 5s");
            }
            Err(e) => {
                tracing::error!(error = %e, "log_tail error, restarting in 5s");
                let _ = db::record_error(&pool, "log_tail", &e.to_string()).await;
            }
        }
        tokio::select! {
            _ = shutdown.cancelled() => return Ok(()),
            _ = sleep(Duration::from_secs(5)) => {}
        }
    }
}

async fn tail_once(
    path: &str,
    re: &Regex,
    pool: &Pool,
    shutdown: &CancellationToken,
) -> Result<()> {
    let p = Path::new(path);
    if !p.exists() {
        anyhow::bail!("log file does not exist: {path}");
    }

    let initial_meta = tokio::fs::metadata(p)
        .await
        .with_context(|| format!("metadata of {path}"))?;
    let initial_inode = initial_meta.ino();

    let mut file = File::open(p)
        .await
        .with_context(|| format!("opening {path}"))?;
    file.seek(SeekFrom::End(0)).await?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    let mut last_truncate_log: Option<Instant> = None;

    loop {
        buf.clear();
        let n = tokio::select! {
            biased;
            _ = shutdown.cancelled() => return Ok(()),
            res = reader.read_line(&mut buf) => res?,
        };
        if n == 0 {
            tokio::select! {
                biased;
                _ = shutdown.cancelled() => return Ok(()),
                _ = sleep(Duration::from_millis(POLL_INTERVAL_MS)) => {}
            }

            let meta = match tokio::fs::metadata(path).await {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(error = %e, "metadata read failed; retrying");
                    continue;
                }
            };

            // Real rotation: file was unlinked and replaced (different inode).
            // Bail so the outer loop re-opens by path.
            if meta.ino() != initial_inode {
                tracing::info!(
                    old_inode = initial_inode,
                    new_inode = meta.ino(),
                    "log file inode changed, will reopen"
                );
                return Ok(());
            }

            // Same inode but file shorter than our read position: the validator
            // freed bytes from the start of its circular buffer. Re-seek to the
            // current EOF and keep going. We skip whatever was rotated out — by
            // construction those bytes were already past our read head.
            let pos = reader.get_mut().stream_position().await?;
            if meta.len() < pos {
                let should_log = match last_truncate_log {
                    None => true,
                    Some(t) => t.elapsed() >= Duration::from_secs(TRUNCATE_LOG_THROTTLE_SECS),
                };
                if should_log {
                    tracing::info!(
                        last_read_pos = pos,
                        current_size = meta.len(),
                        "circular truncate detected, re-seeking to EOF (throttled)"
                    );
                    last_truncate_log = Some(Instant::now());
                }
                reader.get_mut().seek(SeekFrom::End(0)).await?;
            }
            continue;
        }

        let line = buf.trim_end_matches(['\n', '\r']);
        if let Some(caps) = re.captures(line) {
            let ts_str = caps.name("ts").map(|m| m.as_str()).unwrap_or("");
            let slot_str = caps.name("slot").map(|m| m.as_str()).unwrap_or("");
            let slot: u64 = match slot_str.parse() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let ts_us = match parse_ts_to_us(ts_str) {
                Some(u) => u,
                None => continue,
            };
            if let Err(e) = db::record_slot_obs(pool, slot, ts_us).await {
                tracing::warn!(error = %e, slot, "record_slot_obs failed");
                let _ = db::record_error(pool, "log_tail", &format!("slot_obs: {e}")).await;
            }
        }
    }
}

/// Parse RFC3339 with up to 9 fractional digits → microseconds since epoch.
/// chrono only handles up to nanosecond precision; we truncate to micro.
pub fn parse_ts_to_us(ts: &str) -> Option<i64> {
    let dt: DateTime<Utc> = match DateTime::parse_from_rfc3339(ts) {
        Ok(d) => d.with_timezone(&Utc),
        Err(_) => return None,
    };
    let secs = dt.timestamp();
    let nanos = dt.timestamp_subsec_nanos() as i64;
    Some(secs * 1_000_000 + nanos / 1_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_nanosecond_timestamp() {
        let ts = "2026-04-28T07:31:24.442703241Z";
        let us = parse_ts_to_us(ts).unwrap();
        let expected_dt = DateTime::parse_from_rfc3339(ts).unwrap();
        let expected_us = expected_dt.timestamp() * 1_000_000
            + expected_dt.timestamp_subsec_nanos() as i64 / 1_000;
        assert_eq!(us, expected_us);
    }

    #[test]
    fn parses_log_line() {
        let re = Regex::new(PATTERN).unwrap();
        let line = "[2026-04-28T07:31:24.442703241Z INFO  solana_runtime::bank] bank frozen: 46131510 hash: 9nuW...";
        let caps = re.captures(line).unwrap();
        assert_eq!(&caps["slot"], "46131510");
        assert_eq!(&caps["ts"], "2026-04-28T07:31:24.442703241Z");
    }

    #[test]
    fn rejects_unrelated_lines() {
        let re = Regex::new(PATTERN).unwrap();
        assert!(re
            .captures("[2026-04-28T07:31:24Z INFO solana_metrics] some other event")
            .is_none());
    }

    /// Reproduces the v0.1.0 production failure mode: validator log is a
    /// circular buffer that loses ~2-3 KB from the start every ~5 seconds
    /// while new lines are appended. Old code treated each shrink as full
    /// rotation and exited; this test asserts we keep reading in place.
    #[tokio::test]
    async fn handles_circular_buffer_truncation() {
        let path = std::env::temp_dir().join(format!(
            "x1cd_log_tail_test_{}_{}.log",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path_str = path.to_string_lossy().to_string();

        // Pre-fill with noise so the file has size >0 when log_tail seeks to end.
        {
            let mut f = std::fs::File::create(&path).unwrap();
            for _ in 0..100 {
                writeln!(
                    f,
                    "[2026-04-28T07:31:24.000000000Z INFO  solana_metrics] noise"
                )
                .unwrap();
            }
        }
        let inode_before = std::fs::metadata(&path).unwrap().ino();

        let pool = crate::db::init(":memory:").await.unwrap();
        let pool_clone = pool.clone();
        let path_clone = path_str.clone();
        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();
        let handle = tokio::spawn(async move {
            let _ = run(path_clone, pool_clone, shutdown_clone).await;
        });

        // Let log_tail open + seek to EOF.
        tokio::time::sleep(Duration::from_millis(400)).await;

        // 1) Append a bank-frozen line. log_tail should pick it up.
        {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .unwrap();
            writeln!(
                f,
                "[2026-04-28T07:31:25.000000000Z INFO  solana_runtime::bank] bank frozen: 200001 hash: aaa"
            )
            .unwrap();
        }
        tokio::time::sleep(Duration::from_millis(600)).await;

        let count_before_trunc = crate::db::slot_obs_count(&pool).await.unwrap();
        assert!(
            count_before_trunc >= 1,
            "expected slot 200001 captured before truncate, got {count_before_trunc}"
        );

        // 2) Circular truncate: keep only last 200 bytes (drops most of file
        //    including the first bank-frozen line bytes), but NEW bytes will
        //    follow. File::create on an existing path issues O_TRUNC, which
        //    keeps the inode (no unlink+create).
        let content = std::fs::read(&path).unwrap();
        let keep_from = content.len().saturating_sub(200);
        let kept: Vec<u8> = content[keep_from..].to_vec();
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(&kept).unwrap();
        }
        let inode_after = std::fs::metadata(&path).unwrap().ino();
        assert_eq!(
            inode_before, inode_after,
            "test precondition: O_TRUNC should preserve inode"
        );

        // Give log_tail a moment to detect the smaller size and re-seek.
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 3) Append another bank-frozen line AFTER the truncate. With the bug
        //    fixed, log_tail re-seeks to EOF in place and catches it. With the
        //    bug present, log_tail returned Ok(()) and is sleeping in the
        //    outer loop's 5s restart window — would still need to reopen and
        //    might miss within our test timeout.
        {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .unwrap();
            writeln!(
                f,
                "[2026-04-28T07:31:26.000000000Z INFO  solana_runtime::bank] bank frozen: 200002 hash: bbb"
            )
            .unwrap();
        }
        tokio::time::sleep(Duration::from_millis(1500)).await;

        let count_after_trunc = crate::db::slot_obs_count(&pool).await.unwrap();

        shutdown.cancel();
        let _ = handle.await;
        let _ = std::fs::remove_file(&path);

        assert!(
            count_after_trunc >= 2,
            "expected log_tail to keep reading post-truncate (count_before={count_before_trunc} count_after={count_after_trunc})"
        );
    }
}
