use std::sync::Arc;
use std::time::Duration;

use bridge::osu::{Beatmap, Beatmapset};
use bridge::{calculate_all, decode_bytes, normalized_hash_100, CalcResult, CalcType, OsuClientPool, RoxChart, RoxStore};
use db::{enqueue_pending_beatmap, insert_full_beatmap, load_seeder_progress, save_seeder_progress, upsert_beatmapset, ComputedBeatmapData, KeyProgress, PgPool, SeederProgress};
use tokio::sync::Semaphore;
use tokio::time::sleep;

use crate::config::RATE_MIN;
use crate::limiter::Limiter;
use crate::retry::{compute_backoff, is_rate_limited, is_transient};
use crate::state::{LogType, Shared};

/// Drop the effective rate by half (floored at RATE_MIN) when we hit a 429.
async fn on_rate_limited(state: &Shared, limiter: &Limiter) {
    let mut s = state.write().await;
    s.rate_limit_hits += 1;
    s.rate_success_streak = 0;
    let current = s.rate_per_min;
    let new_rate = (current / 2).max(RATE_MIN);
    if new_rate < current {
        s.rate_per_min = new_rate;
        drop(s);
        limiter.set_rate(new_rate);
        state.write().await.log(
            LogType::Retry,
            format!("rate-limit: {current} -> {new_rate} req/min"),
        );
    }
}

/// After a run of successful ops, nudge the rate back up towards `rate_ceiling`.
async fn on_success(state: &Shared, limiter: &Limiter) {
    const RECOVERY_THRESHOLD: u64 = 200;
    const RECOVERY_STEP: u32 = 20;

    let mut s = state.write().await;
    s.rate_success_streak += 1;
    let ceiling = s.rate_ceiling;
    let current = s.rate_per_min;
    if current >= ceiling || s.rate_success_streak < RECOVERY_THRESHOLD {
        return;
    }
    let new_rate = (current + RECOVERY_STEP).min(ceiling);
    s.rate_per_min = new_rate;
    s.rate_success_streak = 0;
    drop(s);
    limiter.set_rate(new_rate);
    state.write().await.log(
        LogType::Info,
        format!("rate recovery: {current} -> {new_rate} req/min"),
    );
}

const DEFAULT_MAX_CONCURRENT_DOWNLOADS: usize = 32;

fn max_concurrent_downloads() -> usize {
    std::env::var("SEEDER_MAX_DOWNLOADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n: &usize| n > 0)
        .unwrap_or(DEFAULT_MAX_CONCURRENT_DOWNLOADS)
}

pub async fn run(
    pool: PgPool,
    osu: Arc<OsuClientPool>,
    limiter: Arc<Limiter>,
    rox_store: Arc<RoxStore>,
    state: Shared,
    keys: Vec<u32>,
) {
    let concurrent = max_concurrent_downloads();
    state
        .write()
        .await
        .log(LogType::Info, format!("concurrent downloads = {concurrent}"));
    let semaphore = Arc::new(Semaphore::new(concurrent));
    let mut key_states = load_key_states(&pool, &state, &keys).await;

    'outer: for idx in 0..key_states.len() {
        if key_states[idx].done {
            continue;
        }
        let key = key_states[idx].key;

        {
            let mut s = state.write().await;
            s.current_key = Some(key);
            s.log(LogType::Info, format!("starting key={key}"));
        }

        let mut cursor = key_states[idx].cursor.clone();

        loop {
            if state.read().await.shutdown {
                break 'outer;
            }
            if state.read().await.paused {
                sleep(Duration::from_millis(200)).await;
                continue;
            }

            limiter.acquire().await;

            let resp = loop {
                match osu.search_mania(cursor.as_deref(), key).await {
                    Ok(r) => break r,
                    Err(e) => {
                        let msg = format!("search error (key={key}): {e}");
                        if is_rate_limited(&msg) {
                            on_rate_limited(&state, &limiter).await;
                        }
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
                    key,
                ));
                handles.push(handle);
            }
            for handle in handles {
                let _ = handle.await;
            }

            cursor = resp.cursor_string;
            key_states[idx].cursor = cursor.clone();
            state.write().await.last_cursor = cursor.clone();

            if cursor.is_none() {
                key_states[idx].done = true;
                state.write().await.log(
                    LogType::Info,
                    format!("key={key} finished (no more cursor)"),
                );
                persist_progress(&pool, &state, &key_states).await;
                break;
            }

            persist_progress(&pool, &state, &key_states).await;
        }
    }

    if key_states.iter().all(|k| k.done) {
        let mut s = state.write().await;
        s.done = true;
        s.log(LogType::Info, "all keys finished".into());
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
    key: u32,
) {
    let set_id = set.id;
    match upsert_beatmapset(&pool, &set).await {
        Ok((set_pk, inserted)) => {
            if inserted {
                state.write().await.sets_inserted += 1;
            }
            state.write().await.last_title = Some(format!("{} - {}", set.artist, set.title));

            if let Some(beatmaps) = set.beatmaps {
                let before = beatmaps.len();
                let mania_maps: Vec<_> = beatmaps
                    .into_iter()
                    .filter(|b| b.mode_int == 3 && (b.cs.round() as u32) == key)
                    .collect();
                let skipped_wrong_key = before.saturating_sub(mania_maps.len());
                if skipped_wrong_key > 0 {
                    let mut s = state.write().await;
                    s.skipped += skipped_wrong_key as u64;
                }
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
            if is_rate_limited(&message) {
                on_rate_limited(state, limiter).await;
            }
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

    let chart_for_calc = chart;
    let calc_join = tokio::task::spawn_blocking(move || calculate_all(&chart_for_calc, 100)).await;
    let (proportions, ratings) = match calc_join {
        Ok(Ok(result)) => result,
        Ok(Err(message)) => {
            record_error(state, LogType::Calc, format!("calc {}: {message}", beatmap.id)).await;
            return;
        }
        Err(e) => {
            record_error(
                state,
                LogType::Calc,
                format!("calc join {}: {e}", beatmap.id),
            )
            .await;
            return;
        }
    };

    let ratings_count = count_unique_rating_types(&ratings);
    let computed = ComputedBeatmapData { proportions, ratings, normalized_hash };

    match insert_full_beatmap(pool, set_pk, beatmap, &osu_hash, &computed).await {
        Ok(true) => {
            {
                let mut stats_guard = state.write().await;
                stats_guard.maps_inserted += 1;
                stats_guard.ratings_inserted += ratings_count;
                if stats_guard.maps_inserted % 100 == 0 {
                    if let Ok(size) = query_db_size(pool).await {
                        stats_guard.db_bytes = size;
                    }
                }
            }
            on_success(state, limiter).await;
        }
        Ok(false) => {
            state.write().await.skipped += 1;
            on_success(state, limiter).await;
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

async fn load_key_states(pool: &PgPool, state: &Shared, configured_keys: &[u32]) -> Vec<KeyProgress> {
    let saved = match load_seeder_progress(pool).await {
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
            s.log(
                LogType::Info,
                format!(
                    "resuming: {} sets, {} maps",
                    progress.sets_inserted, progress.maps_inserted
                ),
            );
            Some(progress)
        }
        Ok(None) => {
            state
                .write()
                .await
                .log(LogType::Info, "starting fresh (no saved progress)".into());
            None
        }
        Err(e) => {
            state
                .write()
                .await
                .log(LogType::Db, format!("failed to load progress: {e}"));
            None
        }
    };

    let mut out: Vec<KeyProgress> = configured_keys
        .iter()
        .map(|&k| KeyProgress { key: k, cursor: None, done: false })
        .collect();

    if let Some(progress) = saved {
        for kp in &progress.keys {
            if let Some(target) = out.iter_mut().find(|k| k.key == kp.key) {
                target.cursor = kp.cursor.clone();
                target.done = kp.done;
            }
        }

        // Legacy single-cursor row: treat it as progress for the first configured
        // key if we have no per-key state yet.
        if progress.keys.is_empty() {
            if let Some(first) = out.first_mut() {
                first.cursor = progress.cursor.clone();
                state.write().await.log(
                    LogType::Info,
                    format!(
                        "legacy cursor adopted for key={}",
                        first.key
                    ),
                );
            }
        }
    }

    for kp in &out {
        let preview = kp
            .cursor
            .as_ref()
            .map(|c| c[..c.len().min(20)].to_string())
            .unwrap_or_else(|| "none".into());
        state.write().await.log(
            LogType::Info,
            format!(
                "key={} cursor={} done={}",
                kp.key, preview, kp.done
            ),
        );
    }

    out
}

async fn persist_progress(pool: &PgPool, state: &Shared, keys: &[KeyProgress]) {
    let s = state.read().await;
    let progress = SeederProgress {
        cursor: None,
        keys: keys.to_vec(),
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
