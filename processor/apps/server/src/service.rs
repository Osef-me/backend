use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use base64::Engine;
use bridge::{calculate_one, CalcType, RoxStore};
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
    let calc_type = CalcType::from_str(&req.calc_type)
        .ok_or_else(|| ServiceError::InvalidArgument(format!("unknown calc_type: {}", req.calc_type)))?;

    if req.centirates.is_empty() {
        return Err(ServiceError::InvalidArgument("centirates must not be empty".into()));
    }

    let (chart, normalized_hash) = match req.input {
        InputKind::Hash(hash) => {
            let chart = state
                .store
                .load(&hash)
                .map_err(ServiceError::Internal)?
                .ok_or_else(|| ServiceError::NotFound(
                    format!("rox file not found for hash {hash}, resubmit with file"),
                ))?;
            (chart, hash)
        }
        InputKind::File(file) => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&file.content)
                .map_err(|e| ServiceError::InvalidArgument(format!("invalid base64: {e}")))?;
            let hint = format!("file.{}", file.extension);
            let chart = auto_decode_bytes(&hint, &bytes)
                .map_err(|e| ServiceError::InvalidArgument(format!("decode error: {e}")))?;
            let hash = normalized_notes_hash(&chart, 100.0);
            state
                .store
                .save_if_absent(&hash, &chart)
                .map_err(ServiceError::Internal)?;
            (chart, hash)
        }
    };

    let mut results = Vec::with_capacity(req.centirates.len());
    for centirate in &req.centirates {
        let key = CacheKey {
            hash: normalized_hash.clone(),
            calc_type: calc_type.as_str().to_string(),
            centirate: *centirate,
        };

        let entry = if let Some(hit) = state.cache.get(&key).await {
            hit
        } else {
            let result = calculate_one(&chart, &calc_type, *centirate)
                .map_err(ServiceError::Internal)?;
            let entry = CacheEntry {
                rating: result.rating,
                mania_skill: result.mania_skill,
            };
            state.cache.insert(key, entry.clone()).await;
            entry
        };

        results.push(RateResult {
            centirate: *centirate,
            rating: entry.rating,
            mania_skill: entry.mania_skill,
        });
    }

    Ok((
        StatusCode::OK,
        Json(CalcResponse {
            normalized_hash,
            results,
        }),
    ))
}
