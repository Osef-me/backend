use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

// minimal URL parsing without extra dep
fn parse_db_url(url: &str) -> Result<(String, String, String, String, String)> {
    // postgres://user:pass@host:port/db
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))
        .ok_or_else(|| anyhow!("unsupported DATABASE_URL scheme"))?;
    let (auth, host_db) = rest
        .split_once('@')
        .ok_or_else(|| anyhow!("missing @ in DATABASE_URL"))?;
    let (user, pass) = auth.split_once(':').unwrap_or((auth, ""));
    let (host_port, db) = host_db
        .split_once('/')
        .ok_or_else(|| anyhow!("missing / db in DATABASE_URL"))?;
    let (host, port) = host_port.split_once(':').unwrap_or((host_port, "5432"));
    Ok((
        user.to_string(),
        pass.to_string(),
        host.to_string(),
        port.to_string(),
        db.split('?').next().unwrap_or(db).to_string(),
    ))
}

pub async fn pg_dump_data(database_url: &str, out_path: &Path) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
    }
    let (user, pass, host, port, db) = parse_db_url(database_url)?;

    let status = Command::new("pg_dump")
        .env("PGPASSWORD", pass)
        .args([
            "-h", &host,
            "-p", &port,
            "-U", &user,
            "-d", &db,
            "--data-only",
            "--no-owner",
            "--no-privileges",
            "-Fp",
            "-t", "beatmapset",
            "-t", "beatmap",
            "-t", "pending_beatmap",
            "-f", out_path.to_str().ok_or_else(|| anyhow!("bad path"))?,
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow!("pg_dump failed: {status}"));
    }
    Ok(())
}

