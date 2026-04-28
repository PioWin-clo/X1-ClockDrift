use crate::config::Config;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::process::Command;

pub async fn commit_and_push(config: &Config) -> Result<()> {
    let repo = &config.git_repo_path;
    let branch = &config.git_branch;
    let key = &config.git_deploy_key;

    if !Path::new(repo).join(".git").exists() {
        anyhow::bail!("repo at {repo} is not a git repository");
    }

    git(&["add", "data/"], repo, key).await?;

    let status = git_output(&["status", "--porcelain"], repo, key).await?;
    if status.trim().is_empty() {
        tracing::debug!("no data changes to commit");
        return Ok(());
    }

    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let msg = format!("data: {ts}");
    git(&["commit", "-m", &msg], repo, key).await?;
    git(&["push", "origin", branch], repo, key).await?;
    tracing::info!(branch = %branch, msg = %msg, "git push ok");
    Ok(())
}

pub async fn maybe_daily_squash(config: &Config) -> Result<()> {
    let now_utc = chrono::Utc::now();
    use chrono::Timelike;
    if now_utc.hour() != 0 || now_utc.minute() >= 30 {
        return Ok(());
    }

    let today_str = now_utc.format("%Y-%m-%d").to_string();
    let marker_path = Path::new(&config.git_repo_path)
        .join(".git")
        .join("clockdrift_last_squash");

    if let Ok(prev) = tokio::fs::read_to_string(&marker_path).await {
        if prev.trim() == today_str {
            return Ok(());
        }
    }

    let yesterday_str = (now_utc - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    do_daily_squash(config, &yesterday_str).await?;
    if let Some(parent) = marker_path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    tokio::fs::write(&marker_path, today_str.as_bytes())
        .await
        .ok();
    Ok(())
}

async fn do_daily_squash(config: &Config, yesterday_label: &str) -> Result<()> {
    let repo = &config.git_repo_path;
    let key = &config.git_deploy_key;
    let branch = &config.git_branch;

    let yesterday_midnight = (chrono::Utc::now() - chrono::Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|d| d.and_utc())
        .ok_or_else(|| anyhow::anyhow!("date overflow"))?;
    let before_iso = yesterday_midnight.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let boundary = git_output(
        &[
            "log",
            &format!("--before={before_iso}"),
            "-n",
            "1",
            "--pretty=%H",
            branch,
        ],
        repo,
        key,
    )
    .await?;
    let boundary = boundary.trim().to_string();

    if boundary.is_empty() {
        tracing::info!("no boundary commit before yesterday midnight; skipping squash");
        return Ok(());
    }

    let count_str = git_output(
        &["rev-list", "--count", &format!("{boundary}..HEAD")],
        repo,
        key,
    )
    .await?;
    let count: i64 = count_str.trim().parse().unwrap_or(0);
    if count < 2 {
        tracing::info!(count, "not enough commits since boundary; skipping squash");
        return Ok(());
    }

    let msg = format!("data: {yesterday_label} daily squash");
    git(&["reset", "--soft", &boundary], repo, key).await?;
    git(&["commit", "-m", &msg], repo, key).await?;
    git(&["push", "--force-with-lease", "origin", branch], repo, key).await?;
    tracing::info!(yesterday = %yesterday_label, n_squashed = count, "daily squash pushed");
    Ok(())
}

async fn git(args: &[&str], repo: &str, key: &str) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env(
            "GIT_SSH_COMMAND",
            format!("ssh -i {key} -o StrictHostKeyChecking=accept-new -o BatchMode=yes"),
        )
        .output()
        .await
        .with_context(|| format!("running git {args:?}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

async fn git_output(args: &[&str], repo: &str, key: &str) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env(
            "GIT_SSH_COMMAND",
            format!("ssh -i {key} -o StrictHostKeyChecking=accept-new -o BatchMode=yes"),
        )
        .output()
        .await
        .with_context(|| format!("running git {args:?}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
