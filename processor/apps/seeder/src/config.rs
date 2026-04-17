use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub osu_client_id: String,
    pub osu_client_secret: String,
    pub dump_path: String,
    pub initial_rate_per_min: u32,
    pub rox_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL missing")?,
            osu_client_id: std::env::var("OSU_CLIENT_ID")
                .context("OSU_CLIENT_ID missing")?,
            osu_client_secret: std::env::var("OSU_CLIENT_SECRET")
                .context("OSU_CLIENT_SECRET missing")?,
            dump_path: std::env::var("SEEDER_DUMP_PATH")
                .unwrap_or_else(|_| "seeds/osu_mania.sql".into()),
            initial_rate_per_min: std::env::var("SEEDER_RATE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            rox_path: std::env::var("ROX_PATH").unwrap_or_else(|_| "rox_data".into()),
        })
    }
}

pub const RATE_MIN: u32 = 10;
pub const RATE_MAX: u32 = 200;
