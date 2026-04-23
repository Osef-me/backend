use std::sync::Arc;
use std::time::Duration;

use bridge::osu::{Beatmap, Beatmapset};
use bridge::{calculate_all, decode_bytes, normalized_hash_100, CalcResult, CalcType, OsuClientPool, RoxChart, RoxStore};
use db::{enqueue_pending_beatmap, insert_full_beatmap, load_seeder_progress, save_seeder_progress, upsert_beatmapset, ComputedBeatmapData, KeyProgress, PgPool, SeederProgress};
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::config::RATE_MIN;
use crate::limiter::Limiter;
use crate::retry::{compute_backoff, is_rate_limited, is_transient};
use crate::state::{LogType, Shared};

const DEFAULT_WORKERS: usize = 32;
const QUEUE_CAPACITY: usize = 512;

fn worker_count() -> usize {
    std::env::var("SEEDER_MAX_DOWNLOADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n: &usize| n > 0)
        .unwrap_or(DEFAULT_WORKERS)
}

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

/// Work item produced by discovery, consumed by processor workers.
struct WorkItem {
    set_pk: i32,
    beatmap: Beatmap,
    key: u32,
}

pub async fn run(
    pool: PgPool,
    osu: Arc<OsuClientPool>,
    limiter: Arc<Limiter>,
    rox_store: Arc<RoxStore>,
    state: Shared,
    keys: Vec<u32>,
) {
    let workers = worker_count();
    state.write().await.log(
        LogType::Info,
        format!("workers={workers} queue_capacity={QUEUE_CAPACITY}"),
    );

    let (tx, rx) = mpsc::channel::<WorkItem>(QUEUE_CAPACITY);
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // Spawn N processor workers.
    let mut worker_handles = Vec::with_capacity(workers);
    for _ in 0..workers {
        let pool = pool.clone();
        let osu = osu.clone();
        let limiter = limiter.clone();
        let rox_store = rox_store.clone();
        let state = state.clone();
        let rx = rx.clone();
        worker_handles.push(tokio::spawn(async move {
            processor_worker(pool, osu, limiter, rox_store, state, rx).await;
        }));
    }

    // Run discovery inline — it owns the sender; workers exit once channel drains.
    discovery(
        pool.clone(),
        osu.clone(),
        limiter.clone(),
        state.clone(),
        keys,
        tx,
    )
    .await;

    // Sender dropped. Workers drain and exit.
    for h in worker_handles {
        let _ = h.await;
    }

    if !state.read().await.shutdown {
        let mut s = state.write().await;
        s.done = true;
        s.log(LogType::Info, "all keys finished".into());
    }
}

// ── discovery ──────────────────────────────────────────────────────────────

async fn discovery(
    pool: PgPool,
    osu: Arc<OsuClientPool>,
    limiter: Arc<Limiter>,
    state: Shared,
    keys: Vec<u32>,
    tx: mpsc::Sender<WorkItem>,
) {
    let mut key_states = load_key_states(&pool, &state, &keys).await;

    for idx in 0..key_states.len() {
        if key_states[idx].done {
            continue;
        }
        let key = key_states[idx].key;
        {
            let mut s = state.write().await;
            s.current_key = Some(key);
            s.log(LogType::Info, format!("discovery: starting key={key}"));
        }

        let mut cursor = key_states[idx].cursor.clone();

        loop {
            if state.read().await.shutdown {
                return;
            }
            if state.read().await.paused {
                sleep(Duration::from_millis(200)).await;
                continue;
            }

            limiter.acquire().await;

            let resp = match fetch_page(&osu, &state, &limiter, cursor.as_deref(), key).await {
                Some(r) => r,
                None => return, // shutdown mid-retry
            };
            state.write().await.retry_attempt = 0;

            update_pagination_state(&state, &resp).await;

            // Enqueue each matching map; this may block on channel full (good — it
            // naturally throttles discovery when processors can't keep up).
            for set in resp.beatmapsets {
                if state.read().await.shutdown {
                    return;
                }
                if let Err(e) = enqueue_set(&pool, &state, &tx, set, key).await {
                    state.write().await.log(LogType::Db, format!("enqueue: {e}"));
                }
            }

            cursor = resp.cursor_string;
            key_states[idx].cursor = cursor.clone();
            state.write().await.last_cursor = cursor.clone();

            if cursor.is_none() {
                key_states[idx].done = true;
                state
                    .write()
                    .await
                    .log(LogType::Info, format!("discovery: key={key} finished"));
                persist_progress(&pool, &state, &key_states).await;
                break;
            }

            persist_progress(&pool, &state, &key_states).await;
        }
    }
}

async fn fetch_page(
    osu: &OsuClientPool,
    state: &Shared,
    limiter: &Limiter,
    cursor: Option<&str>,
    key: u32,
) -> Option<bridge::osu::SearchResp> {
    loop {
        match osu.search_mania(cursor, key).await {
            Ok(r) => return Some(r),
            Err(e) => {
                let msg = format!("search error (key={key}): {e}");
                if is_rate_limited(&msg) {
                    on_rate_limited(state, limiter).await;
                }
                if is_transient(&msg) {
                    let mut s = state.write().await;
                    s.retry_attempt += 1;
                    let attempt = s.retry_attempt;
                    s.log(LogType::Retry, format!("[attempt {attempt}] {msg}"));
                    drop(s);
                    sleep(compute_backoff(attempt)).await;
                } else {
                    record_error(state, LogType::Network, msg).await;
                    sleep(Duration::from_secs(2)).await;
                }
                if state.read().await.shutdown {
                    return None;
                }
            }
        }
    }
}

async fn enqueue_set(
    pool: &PgPool,
    state: &Shared,
    tx: &mpsc::Sender<WorkItem>,
    set: Beatmapset,
    key: u32,
) -> anyhow::Result<()> {
    let set_id = set.id;
    let (set_pk, inserted) = match upsert_beatmapset(pool, &set).await {
        Ok(pair) => pair,
        Err(e) => {
            record_error(state, LogType::Db, format!("upsert set {set_id}: {e}")).await;
            return Ok(());
        }
    };
    if inserted {
        state.write().await.sets_inserted += 1;
    }
    state.write().await.last_title = Some(format!("{} - {}", set.artist, set.title));

    let Some(beatmaps) = set.beatmaps else {
        return Ok(());
    };

    let before = beatmaps.len();
    let mania_maps: Vec<Beatmap> = beatmaps
        .into_iter()
        .filter(|b| b.mode_int == 3 && (b.cs.round() as u32) == key)
        .collect();
    let skipped = before.saturating_sub(mania_maps.len());
    if skipped > 0 {
        state.write().await.skipped += skipped as u64;
    }

    for beatmap in mania_maps {
        // Best-effort pending row so we have a durable hint on crash.
        if let Some(hash) = beatmap.checksum.as_deref().filter(|s| !s.is_empty()) {
            let _ = enqueue_pending_beatmap(pool, hash, beatmap.id as i32).await;
        }
        if tx
            .send(WorkItem { set_pk, beatmap, key })
            .await
            .is_err()
        {
            // Receiver closed (shutdown); stop enqueuing.
            return Ok(());
        }
    }
    Ok(())
}

// ── processor worker ───────────────────────────────────────────────────────

async fn processor_worker(
    pool: PgPool,
    osu: Arc<OsuClientPool>,
    limiter: Arc<Limiter>,
    rox_store: Arc<RoxStore>,
    state: Shared,
    rx: Arc<tokio::sync::Mutex<mpsc::Receiver<WorkItem>>>,
) {
    loop {
        let item = {
            let mut guard = rx.lock().await;
            guard.recv().await
        };
        let Some(item) = item else {
            return; // channel closed
        };

        if state.read().await.shutdown {
            return;
        }
        while state.read().await.paused {
            sleep(Duration::from_millis(200)).await;
        }

        process_beatmap_inner(
            &pool,
            &osu,
            &limiter,
            &rox_store,
            &state,
            item.set_pk,
            &item.beatmap,
        )
        .await;
    }
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
    let computed = ComputedBeatmapData {
        proportions,
        ratings,
        normalized_hash,
    };

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

// ── helpers ────────────────────────────────────────────────────────────────

async fn update_pagination_state(state: &Shared, resp: &bridge::osu::SearchResp) {
    let mut stats_guard = state.write().await;
    stats_guard.pages_fetched += 1;
    // osu! search `total` is flaky: sometimes capped at ~10000 (their internal
    // pagination ceiling), sometimes the full corpus size. Accumulate the max
    // so a lucky later page locks in the real number without ever shrinking.
    if let Some(new_total) = resp.total {
        let current = stats_guard.total_known.unwrap_or(0);
        stats_guard.total_known = Some(current.max(new_total));
    }
}

fn extract_osu_hash(beatmap: &Beatmap) -> Option<String> {
    beatmap
        .checksum
        .as_ref()
        .filter(|hash| !hash.is_empty())
        .cloned()
}

async fn download_and_decode_chart(
    osu: &OsuClientPool,
    beatmap: &Beatmap,
) -> Result<RoxChart, String> {
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

async fn load_key_states(
    pool: &PgPool,
    state: &Shared,
    configured_keys: &[u32],
) -> Vec<KeyProgress> {
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
        .map(|&k| KeyProgress {
            key: k,
            cursor: None,
            done: false,
        })
        .collect();

    if let Some(progress) = saved {
        for kp in &progress.keys {
            if let Some(target) = out.iter_mut().find(|k| k.key == kp.key) {
                target.cursor = kp.cursor.clone();
                target.done = kp.done;
            }
        }

        if progress.keys.is_empty() {
            if let Some(first) = out.first_mut() {
                first.cursor = progress.cursor.clone();
                state.write().await.log(
                    LogType::Info,
                    format!("legacy cursor adopted for key={}", first.key),
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
            format!("key={} cursor={} done={}", kp.key, preview, kp.done),
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
