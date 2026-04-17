mod cache;
mod error;
mod service;
mod types;

use axum::{routing::post, Router};
use bridge::RoxStore;
use service::AppState;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let rox_path = std::env::var("ROX_PATH").unwrap_or_else(|_| "/data/rox".into());
    let state = AppState {
        store: Arc::new(RoxStore::new(&rox_path)),
        cache: cache::new_cache(),
    };

    let app = Router::new()
        .route("/processor.Processor/Calculate", post(service::calculate))
        .route("/health", axum::routing::get(|| async { axum::Json(serde_json::json!({ "ok": true })) }))
        .with_state(state);

    let addr = std::env::var("PROCESSOR_ADDR").unwrap_or_else(|_| "0.0.0.0:4000".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("processor listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
