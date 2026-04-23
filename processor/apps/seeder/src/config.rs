use anyhow::{Context, Result};

const SUPPORTED_KEYS: &[u32] = &[4, 6, 7];

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub dump_path: String,
    pub initial_rate_per_min: u32,
    pub rox_path: String,
    pub keys: Vec<u32>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let keys_raw = std::env::var("SEEDER_KEYS").unwrap_or_else(|_| "4,7".into());
        let keys = parse_keys(&keys_raw)?;

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL missing")?,
            dump_path: std::env::var("SEEDER_DUMP_PATH")
                .unwrap_or_else(|_| "seeds/osu_mania.sql".into()),
            initial_rate_per_min: std::env::var("SEEDER_RATE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
            rox_path: std::env::var("ROX_PATH").unwrap_or_else(|_| "rox_data".into()),
            keys,
        })
    }
}

fn parse_keys(raw: &str) -> Result<Vec<u32>> {
    let mut keys = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let k: u32 = part
            .parse()
            .with_context(|| format!("SEEDER_KEYS: invalid number '{part}'"))?;
        if !SUPPORTED_KEYS.contains(&k) {
            anyhow::bail!(
                "SEEDER_KEYS: unsupported key count {k}, minacalc only supports 4, 6, 7"
            );
        }
        if !keys.contains(&k) {
            keys.push(k);
        }
    }
    if keys.is_empty() {
        anyhow::bail!("SEEDER_KEYS: must contain at least one key count");
    }
    Ok(keys)
}

pub const RATE_MIN: u32 = 10;
// osu! API: 1200 req/min official + 200 burst. Leave some headroom.
pub const RATE_MAX: u32 = 1100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_keys() {
        assert_eq!(parse_keys("4,7").unwrap(), vec![4, 7]);
    }

    #[test]
    fn rejects_unsupported_key() {
        assert!(parse_keys("5").is_err());
    }

    #[test]
    fn dedups_and_trims() {
        assert_eq!(parse_keys(" 4, 4, 7 ").unwrap(), vec![4, 7]);
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_keys("").is_err());
    }
}
