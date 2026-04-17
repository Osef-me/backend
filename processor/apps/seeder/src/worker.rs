use std::sync::Arc;
use std::time::Duration;

use bridge::osu::{Beatmap, Beatmapset, OsuClient};
use bridge::{calculate_all, decode_bytes, normalized_hash_100, CalcResult, CalcType, RoxChart, RoxStore};
use db::{enqueue_pending_beatmap, insert_full_beatmap, upsert_beatmapset, ComputedBeatmapData, PgPool};
use tokio::time::sleep;

use crate::limiter::Limiter;
use crate::state::Shared;

pub async fn run(
    pool: PgPool,
    mut osu: OsuClient,
    limiter: Arc<Limiter>,
    rox_store: Arc<RoxStore>,
    state: Shared,
) {
    let mut cursor: Option<String> = None;
    loop {
        if state.read().await.shutdown {
            break;
        }
        if state.read().await.paused {
            sleep(Duration::from_millis(200)).await;
            continue;
        }

        limiter.acquire().await;

        let resp = match osu.search_mania(cursor.as_deref()).await {
            Ok(r) => r,
            Err(e) => {
                record_error(&state, format!("search error: {e}")).await;
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        update_pagination_state(&state, &resp).await;

        for set in &resp.beatmapsets {
            if state.read().await.shutdown {
                break;
            }
            process_beatmapset(&pool, &mut osu, &limiter, &rox_store, &state, set).await;
        }

        cursor = resp.cursor_string;
        state.write().await.last_cursor = cursor.clone();
        if cursor.is_none() {
            let mut stats_guard = state.write().await;
            stats_guard.done = true;
            stats_guard.message = Some("finished (no more cursor)".into());
            break;
        }
    }
}

async fn update_pagination_state(state: &Shared, resp: &bridge::osu::SearchResp) {
    let mut stats_guard = state.write().await;
    stats_guard.pages_fetched += 1;
    if stats_guard.total_known.is_none() {
        stats_guard.total_known = resp.total;
    }
}

async fn process_beatmapset(
    pool: &PgPool,
    osu: &mut OsuClient,
    limiter: &Arc<Limiter>,
    rox_store: &Arc<RoxStore>,
    state: &Shared,
    set: &Beatmapset,
) {
    match upsert_beatmapset(pool, set).await {
        Ok((set_pk, inserted)) => {
            if inserted {
                state.write().await.sets_inserted += 1;
            }
            state.write().await.last_title = Some(format!("{} - {}", set.artist, set.title));
            if let Some(beatmaps) = &set.beatmaps {
                for beatmap in beatmaps.iter().filter(|b| b.mode_int == 3) {
                    process_mania_beatmap(pool, osu, limiter, rox_store, state, set_pk, beatmap).await;
                }
            }
        }
        Err(e) => {
            record_error(state, format!("upsert set {}: {e}", set.id)).await;
        }
    }
}

async fn process_mania_beatmap(
    pool: &PgPool,
    osu: &mut OsuClient,
    limiter: &Arc<Limiter>,
    rox_store: &Arc<RoxStore>,
    state: &Shared,
    set_pk: i32,
    beatmap: &Beatmap,
) {
    if state.read().await.shutdown {
        return;
    }
    if state.read().await.paused {
        sleep(Duration::from_millis(200)).await;
    }
    process_beatmap(pool, osu, limiter, rox_store, state, set_pk, beatmap).await;
}

async fn process_beatmap(
    pool: &PgPool,
    osu: &mut OsuClient,
    limiter: &Arc<Limiter>,
    rox_store: &Arc<RoxStore>,
    state: &Shared,
    set_pk: i32,
    beatmap: &Beatmap,
) {
    let Some(osu_hash) = extract_osu_hash(beatmap) else {
        state.write().await.skipped += 1;
        return;
    };

    let _ = enqueue_pending_beatmap(pool, &osu_hash, beatmap.id as i32).await;
    limiter.acquire().await;

    let chart = match download_and_decode_chart(osu, beatmap).await {
        Ok(chart) => chart,
        Err(message) => {
            record_error(state, message).await;
            return;
        }
    };

    let normalized_hash = normalized_hash_100(&chart);

    let rox_is_new = match save_chart_to_rox_store(rox_store, &normalized_hash, &chart) {
        Ok(is_new) => is_new,
        Err(message) => {
            record_error(state, message).await;
            return;
        }
    };
    if rox_is_new {
        state.write().await.rox_saved += 1;
    }

    let (proportions, ratings) = match calculate_all(&chart, 100) {
        Ok(result) => result,
        Err(message) => {
            record_error(state, format!("calc {}: {message}", beatmap.id)).await;
            return;
        }
    };

    let ratings_count = count_unique_rating_types(&ratings);
    let computed = ComputedBeatmapData { proportions, ratings, normalized_hash };

    match insert_full_beatmap(pool, set_pk, beatmap, &osu_hash, &computed).await {
        Ok(true) => {
            let mut stats_guard = state.write().await;
            stats_guard.maps_inserted += 1;
            stats_guard.ratings_inserted += ratings_count;
        }
        Ok(false) => {
            state.write().await.skipped += 1;
        }
        Err(e) => {
            record_error(state, format!("insert map {}: {e}", beatmap.id)).await;
        }
    }
}

fn extract_osu_hash(beatmap: &Beatmap) -> Option<String> {
    beatmap.checksum.as_ref().filter(|hash| !hash.is_empty()).cloned()
}

async fn download_and_decode_chart(osu: &mut OsuClient, beatmap: &Beatmap) -> Result<RoxChart, String> {
    let bytes = osu
        .download_osu_file(beatmap.id)
        .await
        .map_err(|e| format!("dl {}: {e}", beatmap.id))?;
    decode_bytes("x.osu", &bytes).map_err(|e| format!("decode {}: {e}", beatmap.id))
}

fn save_chart_to_rox_store(
    rox_store: &RoxStore,
    normalized_hash: &str,
    chart: &RoxChart,
) -> Result<bool, String> {
    let is_new = !rox_store.contains(normalized_hash);
    rox_store
        .save_if_absent(normalized_hash, chart)
        .map_err(|e| format!("rox save: {e}"))?;
    Ok(is_new)
}

fn count_unique_rating_types(ratings: &[(CalcType, CalcResult)]) -> u64 {
    ratings
        .iter()
        .map(|(calc_type, _)| calc_type.rating_type())
        .collect::<std::collections::HashSet<_>>()
        .len() as u64
}

async fn record_error(state: &Shared, message: String) {
    let mut stats_guard = state.write().await;
    stats_guard.errors += 1;
    stats_guard.message = Some(message);
}
