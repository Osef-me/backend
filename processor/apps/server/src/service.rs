use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use base64::Engine;
use bridge::{calculate_one, CalcType, RoxChart, RoxStore};
use rox_analysis::hash::normalized_notes_hash;
use rox_formats::auto::auto_decode_bytes;

use crate::cache::{AppCache, CacheEntry, CacheKey};
use crate::error::ServiceError;
use crate::types::{CalcRequest, CalcResponse, InputKind, RateResult};

#[derive(Clone)]
pub struct AppState {
    pub store: std::sync::Arc<RoxStore>,
    pub cache: AppCache,
}

pub async fn calculate(
    State(state): State<AppState>,
    Json(req): Json<CalcRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let calc_type = parse_calc_type(&req.calc_type)?;
    validate_centirates(&req.centirates)?;
    let (chart, normalized_hash) = resolve_chart(req.input, &state.store).await?;
    let results = collect_rate_results(&state, &normalized_hash, calc_type, &chart, &req.centirates).await?;
    Ok((StatusCode::OK, Json(CalcResponse { normalized_hash, results })))
}

fn parse_calc_type(name: &str) -> Result<CalcType, ServiceError> {
    name.parse().map_err(ServiceError::InvalidArgument)
}

fn validate_centirates(centirates: &[u32]) -> Result<(), ServiceError> {
    if centirates.is_empty() {
        return Err(ServiceError::InvalidArgument("centirates must not be empty".into()));
    }
    Ok(())
}

async fn resolve_chart(input: InputKind, store: &RoxStore) -> Result<(RoxChart, String), ServiceError> {
    match input {
        InputKind::Hash(hash) => {
            let chart = store
                .load(&hash)
                .map_err(ServiceError::Internal)?
                .ok_or_else(|| ServiceError::NotFound(
                    format!("rox file not found for hash {hash}, resubmit with file"),
                ))?;
            Ok((chart, hash))
        }
        InputKind::File(file) => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&file.content)
                .map_err(|e| ServiceError::InvalidArgument(format!("invalid base64: {e}")))?;
            let hint = format!("file.{}", file.extension);
            let chart = auto_decode_bytes(&hint, &bytes)
                .map_err(|e| ServiceError::InvalidArgument(format!("decode error: {e}")))?;
            let hash = normalized_notes_hash(&chart, 100.0);
            store.save_if_absent(&hash, &chart).map_err(ServiceError::Internal)?;
            Ok((chart, hash))
        }
    }
}

async fn get_or_compute_entry(
    cache: &AppCache,
    key: CacheKey,
    chart: &RoxChart,
    calc_type: CalcType,
    centirate: u32,
) -> Result<CacheEntry, ServiceError> {
    if let Some(hit) = cache.get(&key).await {
        return Ok(hit);
    }
    let result = calculate_one(chart, calc_type, centirate).map_err(ServiceError::Internal)?;
    let entry = CacheEntry { rating: result.rating, mania_skill: result.mania_skill };
    cache.insert(key, entry.clone()).await;
    Ok(entry)
}

async fn collect_rate_results(
    state: &AppState,
    normalized_hash: &str,
    calc_type: CalcType,
    chart: &RoxChart,
    centirates: &[u32],
) -> Result<Vec<RateResult>, ServiceError> {
    let mut results = Vec::with_capacity(centirates.len());
    for &centirate in centirates {
        let key = CacheKey {
            hash: normalized_hash.to_string(),
            calc_type: calc_type.to_string(),
            centirate,
        };
        let entry = get_or_compute_entry(&state.cache, key, chart, calc_type, centirate).await?;
        results.push(RateResult { centirate, rating: entry.rating, mania_skill: entry.mania_skill });
    }
    Ok(results)
}
