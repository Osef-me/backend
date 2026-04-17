mod config;
mod db;
mod dump;
mod limiter;
mod state;
mod tui;
mod worker;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use bridge::{OsuClient, OsuCredentials, RoxStore};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env()?;
    let shared = state::new_shared(cfg.initial_rate_per_min);
    let lim = limiter::Limiter::new(cfg.initial_rate_per_min);

    let pool = db::connect(&cfg.database_url).await?;
    db::run_migrations(&pool).await?;

    let osu_client = OsuClient::new(OsuCredentials {
        client_id: cfg.osu_client_id.clone(),
        client_secret: cfg.osu_client_secret.clone(),
    });
    let rox_store = Arc::new(RoxStore::new(&cfg.rox_path));

    let worker_state = shared.clone();
    let worker_lim = lim.clone();
    let worker_pool = pool.clone();
    let worker_rox = rox_store.clone();
    let worker_handle = tokio::spawn(async move {
        worker::run(worker_pool, osu_client, worker_lim, worker_rox, worker_state.clone()).await;
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
