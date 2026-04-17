use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

struct DbConnParams {
    user: String,
    password: String,
    host: String,
    port: String,
    database: String,
}

fn parse_db_url(url: &str) -> Result<DbConnParams> {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))
        .ok_or_else(|| anyhow!("unsupported DATABASE_URL scheme"))?;
    let (auth, host_db) = rest
        .split_once('@')
        .ok_or_else(|| anyhow!("missing @ in DATABASE_URL"))?;
    let (user, password) = auth.split_once(':').unwrap_or((auth, ""));
    let (host_port, db_with_query) = host_db
        .split_once('/')
        .ok_or_else(|| anyhow!("missing / db in DATABASE_URL"))?;
    let (host, port) = host_port.split_once(':').unwrap_or((host_port, "5432"));
    let database = db_with_query.split('?').next().unwrap_or(db_with_query);
    Ok(DbConnParams {
        user: user.to_string(),
        password: password.to_string(),
        host: host.to_string(),
        port: port.to_string(),
        database: database.to_string(),
    })
}

async fn ensure_output_dir_exists(path: &Path) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
    }
}

async fn run_pg_dump(params: &DbConnParams, out_path: &Path) -> Result<()> {
    let out_str = out_path.to_str().ok_or_else(|| anyhow!("bad output path"))?;
    let status = Command::new("pg_dump")
        .env("PGPASSWORD", &params.password)
        .args([
            "-h", &params.host,
            "-p", &params.port,
            "-U", &params.user,
            "-d", &params.database,
            "--data-only",
            "--no-owner",
            "--no-privileges",
            "-Fp",
            "-t", "beatmapset",
            "-t", "beatmap",
            "-t", "pending_beatmap",
            "-f", out_str,
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

pub async fn pg_dump_data(database_url: &str, out_path: &Path) -> Result<()> {
    ensure_output_dir_exists(out_path).await;
    let params = parse_db_url(database_url)?;
    run_pg_dump(&params, out_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_postgres_url() {
        let params = parse_db_url("postgres://alice:secret@localhost:5432/mydb").unwrap();
        assert_eq!(params.user, "alice");
        assert_eq!(params.password, "secret");
        assert_eq!(params.host, "localhost");
        assert_eq!(params.port, "5432");
        assert_eq!(params.database, "mydb");
    }

    #[test]
    fn parse_postgresql_scheme_variant() {
        let params = parse_db_url("postgresql://alice:secret@localhost:5432/mydb").unwrap();
        assert_eq!(params.user, "alice");
    }

    #[test]
    fn parse_url_defaults_port_to_5432() {
        let params = parse_db_url("postgres://alice:secret@localhost/mydb").unwrap();
        assert_eq!(params.port, "5432");
    }

    #[test]
    fn parse_url_strips_query_string() {
        let params = parse_db_url("postgres://alice:secret@localhost/mydb?sslmode=disable").unwrap();
        assert_eq!(params.database, "mydb");
    }

    #[test]
    fn parse_url_no_password() {
        let params = parse_db_url("postgres://alice@localhost/mydb").unwrap();
        assert_eq!(params.user, "alice");
        assert_eq!(params.password, "");
    }

    #[test]
    fn unsupported_scheme_returns_error() {
        assert!(parse_db_url("mysql://user:pass@host/db").is_err());
    }

    #[test]
    fn missing_at_sign_returns_error() {
        assert!(parse_db_url("postgres://usernamehost/db").is_err());
    }

    #[test]
    fn missing_slash_after_host_returns_error() {
        assert!(parse_db_url("postgres://user:pass@host").is_err());
    }
}
