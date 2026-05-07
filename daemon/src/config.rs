use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub log_path: String,
    pub rpc_url: String,
    pub db_path: String,
    pub api_listen: String,
    pub git_repo_path: String,
    pub git_remote_url: String,
    pub git_deploy_key: String,
    pub git_branch: String,
    pub export_interval_secs: u64,
    pub rpc_rate_limit_per_sec: u32,
    pub retention_days: u32,
    pub kill_switch_path: String,
    pub watchdog_secs: u64,

    #[serde(default = "default_stake_refresh_secs")]
    pub stake_refresh_secs: u64,

    #[serde(default = "default_history_retention_days")]
    pub history_retention_days: u32,

    #[serde(default)]
    pub frontend_dir: Option<String>,

    /// v1.5.1 — how many days of network drift history to backfill on
    /// cold start. Default: 1 day (was 7 hardcoded in ≤v1.5.0).
    /// Backfill is non-blocking from v1.5.1 onward — exporter publishes
    /// `summary.json` immediately and fills history in the background —
    /// but a tighter default still matters because backfill churns the
    /// SQLite page cache and competes with the steady-state export
    /// cycle for I/O. Set to 0 to skip backfill entirely; raise to 7
    /// when rebuilding a database from scratch and depth is needed.
    #[serde(default = "default_backfill_lookback_days")]
    pub backfill_lookback_days: u32,

    /// v1.5.1 — opt-in cross-validation against the X1 RPC's
    /// `getVoteAccounts` response. When enabled, the daemon compares
    /// its `active_validators` count against the RPC's `current.len()`
    /// once per export cycle and logs a WARN if the delta exceeds 5%.
    /// Default `false` (no extra RPC traffic). Useful for spotting
    /// future Capybara-style cleanup events early.
    #[serde(default)]
    pub validate_against_rpc: bool,
}

fn default_stake_refresh_secs() -> u64 {
    3600
}

fn default_history_retention_days() -> u32 {
    7
}

fn default_backfill_lookback_days() -> u32 {
    1
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config at {}", path.display()))?;
        let cfg: Config = toml::from_str(&raw)
            .with_context(|| format!("failed to parse config at {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        if self.export_interval_secs == 0 {
            anyhow::bail!("export_interval_secs must be > 0");
        }
        if self.rpc_rate_limit_per_sec == 0 {
            anyhow::bail!("rpc_rate_limit_per_sec must be > 0");
        }
        if self.watchdog_secs < 30 {
            anyhow::bail!("watchdog_secs must be >= 30");
        }
        if self.git_branch.trim().is_empty() {
            anyhow::bail!("git_branch must not be empty");
        }
        if !self.api_listen.contains(':') {
            anyhow::bail!("api_listen must be in HOST:PORT form");
        }
        // v1.5.1: backfill window cap. 7 days is the deliberate upper
        // bound — anything beyond that is just churning the page cache
        // for buckets the dashboard never displays (the network chart
        // window selector tops out at 12 days but only 6 raw + 1 day
        // aggregated visible buckets are useful in practice).
        if self.backfill_lookback_days > 7 {
            anyhow::bail!(
                "backfill_lookback_days must be ≤ 7 (got {})",
                self.backfill_lookback_days
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let toml = r#"
log_path = "/var/log/validator.log"
rpc_url = "https://rpc.mainnet.x1.xyz"
db_path = "/data/data.db"
api_listen = "127.0.0.1:8088"
git_repo_path = "/data/repo"
git_remote_url = "git@github.com:PioWin-clo/X1-ClockDrift.git"
git_deploy_key = "/keys/deploy"
git_branch = "data"
export_interval_secs = 300
rpc_rate_limit_per_sec = 5
retention_days = 30
kill_switch_path = "/data/STOP"
watchdog_secs = 120
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.git_branch, "data");
        assert_eq!(cfg.export_interval_secs, 300);
        assert_eq!(cfg.stake_refresh_secs, 3600);
        assert_eq!(cfg.history_retention_days, 7);
        // v1.5.1: defaults for the new operability fields.
        assert_eq!(cfg.backfill_lookback_days, 1);
        assert!(!cfg.validate_against_rpc);
    }

    /// v1.5.1: backfill window must not exceed 7 days. Catches a typo
    /// that would otherwise force a 90-minute cold-start backfill on
    /// every restart (which is exactly the v1.5.0 deploy bug).
    #[test]
    fn rejects_excessive_backfill() {
        let cfg = Config {
            log_path: "/x".into(),
            rpc_url: "http://x".into(),
            db_path: "/x".into(),
            api_listen: "127.0.0.1:1".into(),
            git_repo_path: "/x".into(),
            git_remote_url: "x".into(),
            git_deploy_key: "/x".into(),
            git_branch: "data".into(),
            export_interval_secs: 300,
            rpc_rate_limit_per_sec: 5,
            retention_days: 30,
            kill_switch_path: "/x".into(),
            watchdog_secs: 120,
            stake_refresh_secs: 3600,
            history_retention_days: 7,
            frontend_dir: None,
            backfill_lookback_days: 30,
            validate_against_rpc: false,
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_zero_interval() {
        let cfg = Config {
            log_path: "/x".into(),
            rpc_url: "http://x".into(),
            db_path: "/x".into(),
            api_listen: "127.0.0.1:1".into(),
            git_repo_path: "/x".into(),
            git_remote_url: "x".into(),
            git_deploy_key: "/x".into(),
            git_branch: "data".into(),
            export_interval_secs: 0,
            rpc_rate_limit_per_sec: 5,
            retention_days: 30,
            kill_switch_path: "/x".into(),
            watchdog_secs: 120,
            stake_refresh_secs: 3600,
            history_retention_days: 7,
            frontend_dir: None,
            backfill_lookback_days: 1,
            validate_against_rpc: false,
        };
        assert!(cfg.validate().is_err());
    }
}
