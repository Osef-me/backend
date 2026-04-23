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
use db::{connect, run_migrations, PgPool};
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::limiter::Limiter;
use crate::state::Shared;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    init_logfile()?;

    let result = run().await;

    log_run_result(&result);
    result
}

async fn run() -> Result<()> {
    let ctx = setup().await?;
    let worker = spawn_worker(&ctx);

    let tui_res = tui::run(ctx.shared.clone(), ctx.limiter.clone()).await;
    ctx.shared.write().await.shutdown = true;

    join_worker(worker).await;
    report_tui_error(tui_res);

    run_dump_phase(&ctx.cfg).await
}

/// Runtime handles owned by the main task and handed to the worker.
struct RuntimeCtx {
    cfg: Config,
    shared: Shared,
    limiter: Arc<Limiter>,
    pool: PgPool,
    osu_pool: Arc<OsuClientPool>,
    rox_store: Arc<RoxStore>,
}

// ── startup ────────────────────────────────────────────────────────────────

fn init_logfile() -> Result<()> {
    let log_dir = std::env::var("SEEDER_LOG_DIR").unwrap_or_else(|_| "logs".into());
    let lf = logfile::init(Path::new(&log_dir))
        .map_err(|e| anyhow::anyhow!("failed to open log file in {log_dir}: {e}"))?;
    logfile::install_panic_hook();
    lf.write(
        "INFO",
        &format!("seeder starting, log file: {}", lf.path().display()),
    );
    Ok(())
}

fn log_run_result(result: &Result<()>) {
    let Some(lf) = logfile::global() else { return };
    match result {
        Ok(()) => lf.write("INFO", "seeder finished cleanly"),
        Err(e) => lf.write("FATAL", &format!("seeder exited with error: {e:#}")),
    }
}

async fn setup() -> Result<RuntimeCtx> {
    let cfg = load_config()?;
    let shared = state::new_shared(cfg.initial_rate_per_min);

    let (credentials, effective_rate) = prepare_credentials(&cfg)?;
    let limiter = Limiter::new(effective_rate);
    init_shared_rate(&shared, effective_rate).await;

    let pool = connect(&cfg.database_url).await?;
    run_migrations(&pool).await?;

    let osu_pool = Arc::new(OsuClientPool::new(credentials));
    let rox_store = Arc::new(RoxStore::new(&cfg.rox_path));

    Ok(RuntimeCtx {
        cfg,
        shared,
        limiter,
        pool,
        osu_pool,
        rox_store,
    })
}

fn load_config() -> Result<Config> {
    let cfg = Config::from_env()?;
    log_info(format!(
        "config: rate_per_min={} dump_path={} rox_path={} keys={:?}",
        cfg.initial_rate_per_min, cfg.dump_path, cfg.rox_path, cfg.keys
    ));
    Ok(cfg)
}

fn prepare_credentials(cfg: &Config) -> Result<(Vec<bridge::OsuCredentials>, u32)> {
    let credentials = load_credentials_from_env();
    if credentials.is_empty() {
        anyhow::bail!("no osu! credentials found (OSU_CLIENT_ID/OSU_CLIENT_SECRET)");
    }
    let client_count = credentials.len();
    let effective_rate = cfg.initial_rate_per_min * client_count as u32;
    eprintln!(
        "loaded {client_count} osu! client(s), effective rate: {effective_rate} req/min"
    );
    log_info(format!(
        "loaded {client_count} osu! client(s), effective rate: {effective_rate} req/min"
    ));
    Ok((credentials, effective_rate))
}

async fn init_shared_rate(shared: &Shared, effective_rate: u32) {
    let mut s = shared.write().await;
    s.rate_per_min = effective_rate;
    s.rate_ceiling = effective_rate;
}

// ── worker lifecycle ───────────────────────────────────────────────────────

fn spawn_worker(ctx: &RuntimeCtx) -> JoinHandle<()> {
    let shared = ctx.shared.clone();
    let limiter = ctx.limiter.clone();
    let pool = ctx.pool.clone();
    let rox_store = ctx.rox_store.clone();
    let osu_pool = ctx.osu_pool.clone();
    let keys = ctx.cfg.keys.clone();

    tokio::spawn(async move {
        worker::run(pool, osu_pool, limiter, rox_store, shared.clone(), keys).await;
        shared.write().await.done = true;
    })
}

async fn join_worker(handle: JoinHandle<()>) {
    let Err(e) = handle.await else { return };
    let msg = if e.is_panic() {
        format!("worker task panicked: {e}")
    } else if e.is_cancelled() {
        "worker task cancelled".into()
    } else {
        format!("worker task join error: {e}")
    };
    eprintln!("{msg}");
    log_fatal(msg);
}

fn report_tui_error(tui_res: Result<()>) {
    let Err(e) = tui_res else { return };
    eprintln!("tui error: {e}");
    log_fatal(format!("tui error: {e}"));
}

// ── dump phase ─────────────────────────────────────────────────────────────

async fn run_dump_phase(cfg: &Config) -> Result<()> {
    let out = PathBuf::from(&cfg.dump_path);
    eprintln!("dumping to {}…", out.display());
    log_info(format!("dumping to {}", out.display()));

    if let Err(e) = dump::pg_dump_data(&cfg.database_url, &out).await {
        log_fatal(format!("pg_dump failed: {e:#}"));
        return Err(e);
    }

    eprintln!("dump done: {}", out.display());
    log_info(format!("dump done: {}", out.display()));
    Ok(())
}

// ── log helpers ────────────────────────────────────────────────────────────

fn log_info(msg: String) {
    if let Some(lf) = logfile::global() {
        lf.write("INFO", &msg);
    }
}

fn log_fatal(msg: String) {
    if let Some(lf) = logfile::global() {
        lf.write("FATAL", &msg);
    }
}
