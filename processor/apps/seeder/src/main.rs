mod config;
mod dump;
mod limiter;
mod retry;
mod state;
mod tui;
mod worker;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use bridge::{load_credentials_from_env, OsuClientPool, RoxStore};
use db::{connect, run_migrations};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env()?;
    let shared = state::new_shared(cfg.initial_rate_per_min);

    let credentials = load_credentials_from_env();
    if credentials.is_empty() {
        anyhow::bail!("no osu! credentials found (OSU_CLIENT_ID/OSU_CLIENT_SECRET)");
    }
    let client_count = credentials.len();
    let effective_rate = cfg.initial_rate_per_min * client_count as u32;
    eprintln!("loaded {} osu! client(s), effective rate: {} req/min", client_count, effective_rate);

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
    let worker_handle = tokio::spawn(async move {
        worker::run(worker_pool, worker_osu, worker_lim, worker_rox, worker_state.clone()).await;
        let mut s = worker_state.write().await;
        s.done = true;
    });

    let tui_res = tui::run(shared.clone(), lim.clone()).await;

    shared.write().await.shutdown = true;
    let _ = worker_handle.await;

    if let Err(e) = tui_res {
        eprintln!("tui error: {e}");
    }

    let out = PathBuf::from(&cfg.dump_path);
    eprintln!("dumping to {}…", out.display());
    dump::pg_dump_data(&cfg.database_url, &out).await?;
    eprintln!("dump done: {}", out.display());

    Ok(())
}
