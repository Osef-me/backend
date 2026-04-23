mod config;
mod dump;
mod limiter;
mod logfile;
mod retry;
mod state;
mod tui;
mod worker;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use bridge::{load_credentials_from_env, OsuClientPool, RoxStore};
use db::{connect, run_migrations};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let log_dir = std::env::var("SEEDER_LOG_DIR").unwrap_or_else(|_| "logs".into());
    let lf = logfile::init(Path::new(&log_dir))
        .map_err(|e| anyhow::anyhow!("failed to open log file in {log_dir}: {e}"))?;
    logfile::install_panic_hook();
    lf.write("INFO", &format!("seeder starting, log file: {}", lf.path().display()));

    let result = run().await;

    match &result {
        Ok(()) => lf.write("INFO", "seeder finished cleanly"),
        Err(e) => lf.write("FATAL", &format!("seeder exited with error: {e:#}")),
    }
    result
}

async fn run() -> Result<()> {
    let cfg = config::Config::from_env()?;
    logfile::global().map(|lf| {
        lf.write(
            "INFO",
            &format!(
                "config: rate_per_min={} dump_path={} rox_path={} keys={:?}",
                cfg.initial_rate_per_min, cfg.dump_path, cfg.rox_path, cfg.keys
            ),
        )
    });

    let shared = state::new_shared(cfg.initial_rate_per_min);

    let credentials = load_credentials_from_env();
    if credentials.is_empty() {
        anyhow::bail!("no osu! credentials found (OSU_CLIENT_ID/OSU_CLIENT_SECRET)");
    }
    let client_count = credentials.len();
    let effective_rate = cfg.initial_rate_per_min * client_count as u32;
    eprintln!(
        "loaded {} osu! client(s), effective rate: {} req/min",
        client_count, effective_rate
    );
    logfile::global().map(|lf| {
        lf.write(
            "INFO",
            &format!(
                "loaded {client_count} osu! client(s), effective rate: {effective_rate} req/min"
            ),
        )
    });

    let lim = limiter::Limiter::new(effective_rate);
    shared.write().await.rate_per_min = effective_rate;

    let pool = connect(&cfg.database_url).await?;
    run_migrations(&pool).await?;

    let osu_pool = Arc::new(OsuClientPool::new(credentials));
    let rox_store = Arc::new(RoxStore::new(&cfg.rox_path));

    let worker_state = shared.clone();
    let worker_lim = lim.clone();
    let worker_pool = pool.clone();
    let worker_rox = rox_store.clone();
    let worker_osu = osu_pool.clone();
    let worker_keys = cfg.keys.clone();
    let worker_handle = tokio::spawn(async move {
        worker::run(
            worker_pool,
            worker_osu,
            worker_lim,
            worker_rox,
            worker_state.clone(),
            worker_keys,
        )
        .await;
        let mut s = worker_state.write().await;
        s.done = true;
    });

    let tui_res = tui::run(shared.clone(), lim.clone()).await;

    shared.write().await.shutdown = true;
    let worker_join = worker_handle.await;
    if let Err(e) = &worker_join {
        let msg = if e.is_panic() {
            format!("worker task panicked: {e}")
        } else if e.is_cancelled() {
            "worker task cancelled".into()
        } else {
            format!("worker task join error: {e}")
        };
        eprintln!("{msg}");
        logfile::global().map(|lf| lf.write("FATAL", &msg));
    }

    if let Err(e) = tui_res {
        eprintln!("tui error: {e}");
        logfile::global().map(|lf| lf.write("FATAL", &format!("tui error: {e}")));
    }

    let out = PathBuf::from(&cfg.dump_path);
    eprintln!("dumping to {}…", out.display());
    logfile::global().map(|lf| lf.write("INFO", &format!("dumping to {}", out.display())));
    if let Err(e) = dump::pg_dump_data(&cfg.database_url, &out).await {
        logfile::global().map(|lf| lf.write("FATAL", &format!("pg_dump failed: {e:#}")));
        return Err(e);
    }
    eprintln!("dump done: {}", out.display());
    logfile::global().map(|lf| lf.write("INFO", &format!("dump done: {}", out.display())));

    Ok(())
}
