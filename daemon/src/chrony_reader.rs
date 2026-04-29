use crate::db::{self, ChronySource, ChronyTracking, Pool};
use anyhow::{Context, Result};
use std::time::Duration;
use tokio::process::Command;

const POLL_INTERVAL_SECS: u64 = 30;

pub struct NtpSource {
    pub ip_match: &'static str,
    pub hostname: &'static str,
    pub operator: &'static str,
    pub country_code: &'static str,
    pub country_name: &'static str,
}

pub const KNOWN_SOURCES: &[NtpSource] = &[
    NtpSource {
        ip_match: "194.146.251.100",
        hostname: "tempus1.gum.gov.pl",
        operator: "GUM",
        country_code: "PL",
        country_name: "Poland",
    },
    NtpSource {
        ip_match: "194.146.251.101",
        hostname: "tempus2.gum.gov.pl",
        operator: "GUM",
        country_code: "PL",
        country_name: "Poland",
    },
    NtpSource {
        ip_match: "194.146.251.102",
        hostname: "tempus3.gum.gov.pl",
        operator: "GUM",
        country_code: "PL",
        country_name: "Poland",
    },
    NtpSource {
        ip_match: "2001:638:610:be01::108",
        hostname: "ptbtime1.ptb.de",
        operator: "PTB",
        country_code: "DE",
        country_name: "Germany",
    },
    NtpSource {
        ip_match: "2001:638:610:be01::104",
        hostname: "ptbtime2.ptb.de",
        operator: "PTB",
        country_code: "DE",
        country_name: "Germany",
    },
    NtpSource {
        ip_match: "2001:638:610:be01::103",
        hostname: "ptbtime3.ptb.de",
        operator: "PTB",
        country_code: "DE",
        country_name: "Germany",
    },
    NtpSource {
        ip_match: "2001:718:1:1::144:201",
        hostname: "tik.cesnet.cz",
        operator: "CESNET",
        country_code: "CZ",
        country_name: "Czechia",
    },
    NtpSource {
        ip_match: "2001:718:1:101::144:238",
        hostname: "tak.cesnet.cz",
        operator: "CESNET",
        country_code: "CZ",
        country_name: "Czechia",
    },
    NtpSource {
        ip_match: "2a01:3f7:2:1::1",
        hostname: "sth1.ntp.netnod.se",
        operator: "Netnod",
        country_code: "SE",
        country_name: "Sweden",
    },
    NtpSource {
        ip_match: "2a01:3f7:2:2::1",
        hostname: "sth2.ntp.netnod.se",
        operator: "Netnod",
        country_code: "SE",
        country_name: "Sweden",
    },
];

/// Returns (hostname, operator, country_code, country_name).
/// For unknown IPs, returns (ip, "Unknown", None, None).
pub fn lookup_source(ip: &str) -> (String, String, Option<String>, Option<String>) {
    for s in KNOWN_SOURCES {
        if s.ip_match == ip {
            return (
                s.hostname.to_string(),
                s.operator.to_string(),
                Some(s.country_code.to_string()),
                Some(s.country_name.to_string()),
            );
        }
    }
    (ip.to_string(), "Unknown".to_string(), None, None)
}

pub fn mode_char_label(c: char) -> &'static str {
    match c {
        '^' => "server",
        '=' => "peer",
        '#' => "local",
        _ => "unknown",
    }
}

pub fn state_char_label(c: char) -> &'static str {
    match c {
        '*' => "current_best",
        '+' => "combined",
        '-' => "excluded",
        'x' => "false_ticker",
        '?' => "unreachable",
        '~' => "too_variable",
        _ => "unknown",
    }
}

/// Localised state label pair (English, Polish), pre-rendered for the JSON
/// export so the frontend just picks one based on the user's language.
pub fn state_labels(state: &str) -> (&'static str, &'static str) {
    match state {
        "current_best" => ("\u{2605} primary", "\u{2605} g\u{0142}\u{00f3}wne"),
        "combined" => ("\u{2713} combined", "\u{2713} \u{0142}\u{0105}czone"),
        "excluded" => ("\u{25cb} excluded", "\u{25cb} wykluczone"),
        "false_ticker" => ("\u{2717} false ticker", "\u{2717} b\u{0142}\u{0119}dne"),
        "unreachable" => ("? unreachable", "? nieosi\u{0105}galne"),
        "too_variable" => ("~ too variable", "~ zbyt zmienne"),
        _ => ("? unknown", "? nieznane"),
    }
}

#[derive(Debug, PartialEq)]
pub struct TrackingParsed {
    pub reference_id: String,
    pub reference_ip: String,
    pub stratum: i64,
    pub ref_time_unix: f64,
    pub system_offset_seconds: f64,
    pub last_offset_seconds: f64,
    pub rms_offset_seconds: f64,
    pub frequency_ppm: f64,
    pub residual_freq_ppm: f64,
    pub skew_ppm: f64,
    pub root_delay_seconds: f64,
    pub root_dispersion_seconds: f64,
    pub update_interval_seconds: f64,
    pub leap_status: String,
}

pub fn parse_tracking_csv(line: &str) -> Option<TrackingParsed> {
    let fields: Vec<&str> = line.trim().split(',').collect();
    if fields.len() < 14 {
        return None;
    }
    Some(TrackingParsed {
        reference_id: fields[0].to_string(),
        reference_ip: fields[1].to_string(),
        stratum: fields[2].parse().ok()?,
        ref_time_unix: fields[3].parse().ok()?,
        system_offset_seconds: fields[4].parse().ok()?,
        last_offset_seconds: fields[5].parse().ok()?,
        rms_offset_seconds: fields[6].parse().ok()?,
        frequency_ppm: fields[7].parse().ok()?,
        residual_freq_ppm: fields[8].parse().ok()?,
        skew_ppm: fields[9].parse().ok()?,
        root_delay_seconds: fields[10].parse().ok()?,
        root_dispersion_seconds: fields[11].parse().ok()?,
        update_interval_seconds: fields[12].parse().ok()?,
        leap_status: fields[13].to_string(),
    })
}

#[derive(Debug, PartialEq)]
pub struct SourceParsed {
    pub mode_char: char,
    pub state_char: char,
    pub address: String,
    pub stratum: i64,
    pub poll_log2: i64,
    pub reach: i64,
    pub last_rx_seconds: i64,
    pub last_sample_offset_seconds: f64,
    pub last_sample_original_seconds: f64,
    pub last_sample_error_seconds: f64,
}

pub fn parse_sources_csv(text: &str) -> Vec<SourceParsed> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let fields: Vec<&str> = trimmed.split(',').collect();
            if fields.len() < 10 {
                return None;
            }
            Some(SourceParsed {
                mode_char: fields[0].chars().next()?,
                state_char: fields[1].chars().next()?,
                address: fields[2].to_string(),
                stratum: fields[3].parse().ok()?,
                poll_log2: fields[4].parse().ok()?,
                reach: fields[5].parse().ok()?,
                last_rx_seconds: fields[6].parse().ok()?,
                last_sample_offset_seconds: fields[7].parse().ok()?,
                last_sample_original_seconds: fields[8].parse().ok()?,
                last_sample_error_seconds: fields[9].parse().ok()?,
            })
        })
        .collect()
}

pub async fn run(pool: Pool) {
    tracing::info!(
        poll_secs = POLL_INTERVAL_SECS,
        "chrony_reader starting"
    );
    loop {
        match poll_once(&pool).await {
            Ok(()) => {}
            Err(e) => {
                tracing::warn!(error = %e, "chrony poll failed");
                let _ = db::record_error(&pool, "chrony_reader", &e.to_string()).await;
            }
        }
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn poll_once(pool: &Pool) -> Result<()> {
    // chronyc -c tracking — single CSV line
    let tracking_out = Command::new("chronyc")
        .args(["-c", "tracking"])
        .output()
        .await
        .context("spawning chronyc -c tracking (is chrony installed?)")?;
    if !tracking_out.status.success() {
        anyhow::bail!(
            "chronyc tracking exit {}: {}",
            tracking_out.status,
            String::from_utf8_lossy(&tracking_out.stderr).trim()
        );
    }
    let tracking_str = String::from_utf8_lossy(&tracking_out.stdout);
    let tracking_line = tracking_str.lines().next().unwrap_or("").trim();
    let now = chrono::Utc::now().timestamp();
    if let Some(parsed) = parse_tracking_csv(tracking_line) {
        // Accumulate into the current 5-minute history bucket so the
        // history chart can show Sentinel offset alongside chain drift.
        let bucket_ts = (now / 300) * 300;
        let sys_us = parsed.system_offset_seconds * 1_000_000.0;
        let rms_us = parsed.rms_offset_seconds * 1_000_000.0;
        if let Err(e) = db::accumulate_chrony_history(pool, bucket_ts, sys_us, rms_us).await {
            tracing::warn!(error = %e, bucket_ts, "accumulate_chrony_history failed");
        }

        let tracking = ChronyTracking {
            updated_at: now,
            reference_id: Some(parsed.reference_id),
            reference_ip: Some(parsed.reference_ip),
            stratum: Some(parsed.stratum),
            ref_time_unix: Some(parsed.ref_time_unix),
            system_offset_seconds: Some(parsed.system_offset_seconds),
            last_offset_seconds: Some(parsed.last_offset_seconds),
            rms_offset_seconds: Some(parsed.rms_offset_seconds),
            frequency_ppm: Some(parsed.frequency_ppm),
            residual_freq_ppm: Some(parsed.residual_freq_ppm),
            skew_ppm: Some(parsed.skew_ppm),
            root_delay_seconds: Some(parsed.root_delay_seconds),
            root_dispersion_seconds: Some(parsed.root_dispersion_seconds),
            update_interval_seconds: Some(parsed.update_interval_seconds),
            leap_status: Some(parsed.leap_status),
        };
        db::record_chrony_tracking(pool, &tracking).await?;
    } else {
        tracing::warn!(line = tracking_line, "failed to parse chronyc tracking output");
    }

    // chronyc -c sources — multi-line CSV
    let sources_out = Command::new("chronyc")
        .args(["-c", "sources"])
        .output()
        .await
        .context("spawning chronyc -c sources")?;
    if !sources_out.status.success() {
        anyhow::bail!(
            "chronyc sources exit {}: {}",
            sources_out.status,
            String::from_utf8_lossy(&sources_out.stderr).trim()
        );
    }
    let sources_str = String::from_utf8_lossy(&sources_out.stdout);
    let parsed_sources = parse_sources_csv(&sources_str);
    let mapped: Vec<ChronySource> = parsed_sources
        .into_iter()
        .map(|s| {
            let (hostname, operator, country_code, country_name) = lookup_source(&s.address);
            ChronySource {
                ip: s.address,
                hostname,
                operator,
                country_code,
                country_name,
                mode: Some(mode_char_label(s.mode_char).to_string()),
                state: Some(state_char_label(s.state_char).to_string()),
                stratum: Some(s.stratum),
                poll_log2: Some(s.poll_log2),
                reach: Some(s.reach),
                last_rx_seconds: Some(s.last_rx_seconds),
                last_sample_offset_seconds: Some(s.last_sample_offset_seconds),
                last_sample_original_seconds: Some(s.last_sample_original_seconds),
                last_sample_error_seconds: Some(s.last_sample_error_seconds),
                updated_at: now,
            }
        })
        .collect();
    db::record_chrony_sources(pool, &mapped).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tracking_line() {
        let line = "C292FB64,194.146.251.100,2,1777467251.756574615,0.000025088,-0.000009321,0.000020525,6.363,-0.001,0.026,0.002185057,0.001144548,257.8,Normal";
        let parsed = parse_tracking_csv(line).unwrap();
        assert_eq!(parsed.reference_id, "C292FB64");
        assert_eq!(parsed.reference_ip, "194.146.251.100");
        assert_eq!(parsed.stratum, 2);
        assert!((parsed.system_offset_seconds - 0.000025088).abs() < 1e-12);
        assert!((parsed.last_offset_seconds - -0.000009321).abs() < 1e-12);
        assert!((parsed.rms_offset_seconds - 0.000020525).abs() < 1e-12);
        assert!((parsed.frequency_ppm - 6.363).abs() < 1e-9);
        assert!((parsed.skew_ppm - 0.026).abs() < 1e-9);
        assert!((parsed.root_delay_seconds - 0.002185057).abs() < 1e-12);
        assert!((parsed.update_interval_seconds - 257.8).abs() < 1e-9);
        assert_eq!(parsed.leap_status, "Normal");
    }

    #[test]
    fn rejects_short_tracking_line() {
        assert!(parse_tracking_csv("a,b,c").is_none());
        assert!(parse_tracking_csv("").is_none());
    }

    #[test]
    fn parses_sources_lines() {
        let text = "^,*,194.146.251.100,1,8,377,92,0.000026588,0.000017267,0.002118692\n\
                    ^,+,194.146.251.101,1,8,377,67,-0.000341459,-0.000341459,0.002749540\n\
                    ^,-,2001:638:610:be01::108,1,7,377,608,0.001802702,0.001792657,0.011980331\n";
        let sources = parse_sources_csv(text);
        assert_eq!(sources.len(), 3);
        assert_eq!(sources[0].mode_char, '^');
        assert_eq!(sources[0].state_char, '*');
        assert_eq!(sources[0].address, "194.146.251.100");
        assert_eq!(sources[0].reach, 377);
        assert_eq!(sources[1].state_char, '+');
        assert_eq!(sources[2].address, "2001:638:610:be01::108");
        assert_eq!(sources[2].state_char, '-');
    }

    #[test]
    fn skips_malformed_source_lines() {
        let text = "^,*,1.2.3.4,1,8,377,92,0.0,0.0,0.0\n\
                    bogus\n\
                    ^,*,5.6.7.8,1,8,377,93,0.0,0.0,0.0\n\
                    \n";
        let sources = parse_sources_csv(text);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].address, "1.2.3.4");
        assert_eq!(sources[1].address, "5.6.7.8");
    }

    #[test]
    fn lookup_known_source() {
        let (hostname, operator, cc, cn) = lookup_source("194.146.251.100");
        assert_eq!(hostname, "tempus1.gum.gov.pl");
        assert_eq!(operator, "GUM");
        assert_eq!(cc.as_deref(), Some("PL"));
        assert_eq!(cn.as_deref(), Some("Poland"));

        let (hostname, operator, _, _) = lookup_source("2001:638:610:be01::108");
        assert_eq!(hostname, "ptbtime1.ptb.de");
        assert_eq!(operator, "PTB");
    }

    #[test]
    fn lookup_unknown_source_returns_ip_as_hostname() {
        let (hostname, operator, cc, cn) = lookup_source("8.8.8.8");
        assert_eq!(hostname, "8.8.8.8");
        assert_eq!(operator, "Unknown");
        assert!(cc.is_none());
        assert!(cn.is_none());
    }

    #[test]
    fn state_label_mapping_complete() {
        assert_eq!(state_char_label('*'), "current_best");
        assert_eq!(state_char_label('+'), "combined");
        assert_eq!(state_char_label('-'), "excluded");
        assert_eq!(state_char_label('x'), "false_ticker");
        assert_eq!(state_char_label('?'), "unreachable");
        assert_eq!(state_char_label('~'), "too_variable");
        assert_eq!(state_char_label('Z'), "unknown");
    }

    #[test]
    fn mode_label_mapping() {
        assert_eq!(mode_char_label('^'), "server");
        assert_eq!(mode_char_label('='), "peer");
        assert_eq!(mode_char_label('#'), "local");
    }

    #[test]
    fn state_labels_localised() {
        let (en, pl) = state_labels("current_best");
        assert!(en.contains("primary"));
        assert!(pl.contains("g\u{0142}\u{00f3}wne"));
        let (en, pl) = state_labels("excluded");
        assert!(en.contains("excluded"));
        assert!(pl.contains("wykluczone"));
    }
}
