use std::sync::Arc;
use std::time::Duration;

use bridge::osu::{Beatmap, Beatmapset};
use bridge::{calculate_all, decode_bytes, normalized_hash_100, CalcResult, CalcType, OsuClientPool, RoxChart, RoxStore};
use db::{enqueue_pending_beatmap, insert_full_beatmap, load_seeder_progress, save_seeder_progress, upsert_beatmapset, ComputedBeatmapData, PgPool, SeederProgress};
use tokio::sync::Semaphore;
use tokio::time::sleep;

use crate::limiter::Limiter;
use crate::retry::{compute_backoff, is_transient};
use crate::state::{LogType, Shared};

const MAX_CONCURRENT_DOWNLOADS: usize = 8;

pub async fn run(
    pool: PgPool,
    osu: Arc<OsuClientPool>,
    limiter: Arc<Limiter>,
    rox_store: Arc<RoxStore>,
    state: Shared,
) {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS));
    let mut cursor: Option<String> = load_saved_progress(&pool, &state).await;

    loop {
        if state.read().await.shutdown {
            break;
        }
        if state.read().await.paused {
            sleep(Duration::from_millis(200)).await;
            continue;
        }

        limiter.acquire().await;

        let resp = loop {
            match osu.search_mania(cursor.as_deref()).await {
                Ok(r) => break r,
                Err(e) => {
                    let msg = format!("search error: {e}");
                    if is_transient(&msg) {
                        let mut s = state.write().await;
                        s.retry_attempt += 1;
                        let attempt = s.retry_attempt;
                        s.log(LogType::Retry, format!("[attempt {attempt}] {msg}"));
                        drop(s);
                        sleep(compute_backoff(attempt)).await;
                    } else {
                        record_error(&state, LogType::Network, msg).await;
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        };
        state.write().await.retry_attempt = 0;

        update_pagination_state(&state, &resp).await;

        let mut handles = Vec::new();
        for set in resp.beatmapsets {
            if state.read().await.shutdown {
                break;
            }
            let handle = tokio::spawn(process_beatmapset_parallel(
                pool.clone(),
                osu.clone(),
                limiter.clone(),
                rox_store.clone(),
                state.clone(),
                semaphore.clone(),
                set,
            ));
            handles.push(handle);
        }
        for handle in handles {
            let _ = handle.await;
        }

        cursor = resp.cursor_string;
        state.write().await.last_cursor = cursor.clone();
        persist_progress(&pool, &state, cursor.as_deref()).await;
        if cursor.is_none() {
            let mut stats_guard = state.write().await;
            stats_guard.done = true;
            stats_guard.log(LogType::Info, "finished (no more cursor)".into());
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

async fn process_beatmapset_parallel(
    pool: PgPool,
    osu: Arc<OsuClientPool>,
    limiter: Arc<Limiter>,
    rox_store: Arc<RoxStore>,
    state: Shared,
    semaphore: Arc<Semaphore>,
    set: Beatmapset,
) {
    let set_id = set.id;
    match upsert_beatmapset(&pool, &set).await {
        Ok((set_pk, inserted)) => {
            if inserted {
                state.write().await.sets_inserted += 1;
            }
            state.write().await.last_title = Some(format!("{} - {}", set.artist, set.title));

            if let Some(beatmaps) = set.beatmaps {
                let mania_maps: Vec<_> = beatmaps.into_iter().filter(|b| b.mode_int == 3).collect();
                let mut handles = Vec::with_capacity(mania_maps.len());

                for beatmap in mania_maps {
                    let permit = semaphore.clone().acquire_owned().await;
                    if permit.is_err() {
                        break;
                    }
                    let permit = permit.unwrap();

                    let handle = tokio::spawn(process_beatmap_parallel(
                        pool.clone(),
                        osu.clone(),
                        limiter.clone(),
                        rox_store.clone(),
                        state.clone(),
                        set_pk,
                        beatmap,
                        permit,
                    ));
                    handles.push(handle);
                }

                for handle in handles {
                    let _ = handle.await;
                }
            }
        }
        Err(e) => {
            record_error(&state, LogType::Db, format!("upsert set {}: {e}", set_id)).await;
        }
    }
}

async fn process_beatmap_parallel(
    pool: PgPool,
    osu: Arc<OsuClientPool>,
    limiter: Arc<Limiter>,
    rox_store: Arc<RoxStore>,
    state: Shared,
    set_pk: i32,
    beatmap: Beatmap,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    if state.read().await.shutdown {
        return;
    }
    while state.read().await.paused {
        sleep(Duration::from_millis(200)).await;
    }
    process_beatmap_inner(&pool, &osu, &limiter, &rox_store, &state, set_pk, &beatmap).await;
}

async fn process_beatmap_inner(
    pool: &PgPool,
    osu: &OsuClientPool,
    limiter: &Limiter,
    rox_store: &RoxStore,
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
            record_error(state, LogType::Network, message).await;
            return;
        }
    };

    let normalized_hash = normalized_hash_100(&chart);

    let rox_is_new = match save_chart_to_rox_store(rox_store, &normalized_hash, &chart) {
        Ok(is_new) => is_new,
        Err(message) => {
            record_error(state, LogType::Db, message).await;
            return;
        }
    };
    if let Some(bytes) = rox_is_new {
        let mut s = state.write().await;
        s.rox_saved += 1;
        s.rox_bytes += bytes;
    }

    let (proportions, ratings) = match calculate_all(&chart, 100) {
        Ok(result) => result,
        Err(message) => {
            record_error(state, LogType::Calc, format!("calc {}: {message}", beatmap.id)).await;
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
            if stats_guard.maps_inserted % 100 == 0 {
                if let Ok(size) = query_db_size(pool).await {
                    stats_guard.db_bytes = size;
                }
            }
        }
        Ok(false) => {
            state.write().await.skipped += 1;
        }
        Err(e) => {
            record_error(state, LogType::Db, format!("insert map {}: {e}", beatmap.id)).await;
        }
    }
}

fn extract_osu_hash(beatmap: &Beatmap) -> Option<String> {
    beatmap.checksum.as_ref().filter(|hash| !hash.is_empty()).cloned()
}

async fn download_and_decode_chart(osu: &OsuClientPool, beatmap: &Beatmap) -> Result<RoxChart, String> {
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
) -> Result<Option<u64>, String> {
    rox_store
        .save_if_absent_with_size(normalized_hash, chart)
        .map_err(|e| format!("rox save: {e}"))
}

fn count_unique_rating_types(ratings: &[(CalcType, CalcResult)]) -> u64 {
    ratings
        .iter()
        .map(|(calc_type, _)| calc_type.rating_type())
        .collect::<std::collections::HashSet<_>>()
        .len() as u64
}

async fn record_error(state: &Shared, log_type: LogType, message: String) {
    let mut stats_guard = state.write().await;
    stats_guard.errors += 1;
    stats_guard.log(log_type, message);
}

async fn query_db_size(pool: &PgPool) -> Result<u64, String> {
    db::query_db_size(pool).await.map_err(|e| e.to_string())
}

async fn load_saved_progress(pool: &PgPool, state: &Shared) -> Option<String> {
    match load_seeder_progress(pool).await {
        Ok(Some(progress)) => {
            let mut s = state.write().await;
            s.total_known = progress.total_known;
            s.sets_inserted = progress.sets_inserted;
            s.maps_inserted = progress.maps_inserted;
            s.ratings_inserted = progress.ratings_inserted;
            s.rox_saved = progress.rox_saved;
            s.rox_bytes = progress.rox_bytes;
            s.pages_fetched = progress.pages_fetched;
            s.errors = progress.errors;
            s.skipped = progress.skipped;
            let cursor_preview = progress.cursor.as_ref().map(|c| &c[..c.len().min(20)]).unwrap_or("none");
            s.log(LogType::Info, format!("resuming: {} sets, {} maps, cursor: {}", progress.sets_inserted, progress.maps_inserted, cursor_preview));
            progress.cursor
        }
        Ok(None) => {
            state.write().await.log(LogType::Info, "starting fresh (no saved progress)".into());
            None
        }
        Err(e) => {
            state.write().await.log(LogType::Db, format!("failed to load progress: {e}"));
            None
        }
    }
}

async fn persist_progress(pool: &PgPool, state: &Shared, cursor: Option<&str>) {
    let s = state.read().await;
    let progress = SeederProgress {
        cursor: cursor.map(String::from),
        total_known: s.total_known,
        sets_inserted: s.sets_inserted,
        maps_inserted: s.maps_inserted,
        ratings_inserted: s.ratings_inserted,
        rox_saved: s.rox_saved,
        rox_bytes: s.rox_bytes,
        pages_fetched: s.pages_fetched,
        errors: s.errors,
        skipped: s.skipped,
    };
    drop(s);
    if let Err(e) = save_seeder_progress(pool, &progress).await {
        eprintln!("failed to save progress: {e}");
    }
}

