use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

/// Pattern matches:
///   [2026-04-28T07:31:24.442703241Z INFO  solana_runtime::bank] bank frozen: 46131510 hash: ...
const PATTERN: &str = r"^\[(?P<ts>[^\s]+Z)\s+INFO\s+solana_runtime::bank\] bank frozen: (?P<slot>\d+) hash:";

pub async fn run(log_path: String, tx: Sender<(u64, i64)>) -> Result<()> {
    let re = Regex::new(PATTERN).context("invalid log pattern regex")?;
    let path = log_path.clone();
    tracing::info!(path = %path, "log_tail starting");

    loop {
        match tail_once(&path, &re, &tx).await {
            Ok(_) => {
                tracing::warn!("log_tail exited cleanly, restarting in 5s");
            }
            Err(e) => {
                tracing::error!(error = %e, "log_tail error, restarting in 5s");
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}

async fn tail_once(path: &str, re: &Regex, tx: &Sender<(u64, i64)>) -> Result<()> {
    let p = Path::new(path);
    if !p.exists() {
        anyhow::bail!("log file does not exist: {path}");
    }

    let mut file = File::open(p)
        .await
        .with_context(|| format!("opening {path}"))?;
    let initial_len = file.metadata().await?.len();
    file.seek(SeekFrom::Start(initial_len)).await?;
    let mut reader = BufReader::new(file);
    let mut last_size = initial_len;
    let mut buf = String::new();

    loop {
        buf.clear();
        let n = reader.read_line(&mut buf).await?;
        if n == 0 {
            sleep(Duration::from_millis(200)).await;

            let meta = match tokio::fs::metadata(path).await {
                Ok(m) => m,
                Err(e) => {
                    anyhow::bail!("metadata failed: {e}");
                }
            };
            let size = meta.len();
            if size < last_size {
                tracing::info!(
                    old = last_size,
                    new = size,
                    "log file truncated/rotated, reopening"
                );
                return Ok(());
            }
            last_size = size;
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
            if tx.send((slot, ts_us)).await.is_err() {
                tracing::warn!("log_tail downstream closed");
                return Ok(());
            }
        }
        last_size = last_size.saturating_add(n as u64);
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
}
