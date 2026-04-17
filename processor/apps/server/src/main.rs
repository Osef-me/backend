use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tracing_subscriber::EnvFilter;

mod cache;
mod calc;
mod error;
mod rox_store;
mod service;
mod types;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = Router::new().route("/health", get(health));

    let addr = std::env::var("PROCESSOR_ADDR").unwrap_or("0.0.0.0:4000".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("processor listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}
