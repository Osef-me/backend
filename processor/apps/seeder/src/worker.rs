use std::sync::Arc;
use std::time::Duration;

use bridge::osu::{Beatmap, OsuClient};
use bridge::{calculate_all, decode_bytes, normalized_hash_100, RoxStore};
use crate::db::PgPool;
use tokio::time::sleep;

use crate::db::{self, ComputedBeatmapData};
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
                let mut s = state.write().await;
                s.errors += 1;
                s.message = Some(format!("search error: {e}"));
                drop(s);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        {
            let mut s = state.write().await;
            s.pages_fetched += 1;
            if s.total_known.is_none() {
                s.total_known = resp.total;
            }
        }

        for set in &resp.beatmapsets {
            if state.read().await.shutdown {
                break;
            }
            match db::upsert_beatmapset(&pool, set).await {
                Ok((set_pk, inserted)) => {
                    if inserted {
                        state.write().await.sets_inserted += 1;
                    }
                    state.write().await.last_title =
                        Some(format!("{} - {}", set.artist, set.title));

                    if let Some(beatmaps) = &set.beatmaps {
                        for bm in beatmaps.iter().filter(|b| b.mode_int == 3) {
                            if state.read().await.shutdown {
                                break;
                            }
                            if state.read().await.paused {
                                sleep(Duration::from_millis(200)).await;
                            }
                            process_beatmap(
                                &pool, &mut osu, &limiter, &rox_store, &state, set_pk, bm,
                            )
                            .await;
                        }
                    }
                }
                Err(e) => {
                    let mut s = state.write().await;
                    s.errors += 1;
                    s.message = Some(format!("upsert set {}: {e}", set.id));
                }
            }
        }

        cursor = resp.cursor_string.clone();
        state.write().await.last_cursor = cursor.clone();
        if cursor.is_none() {
            let mut s = state.write().await;
            s.done = true;
            s.message = Some("finished (no more cursor)".into());
            break;
        }
    }
}

async fn process_beatmap(
    pool: &PgPool,
    osu: &mut OsuClient,
    limiter: &Arc<Limiter>,
    rox_store: &Arc<RoxStore>,
    state: &Shared,
    set_pk: i32,
    bm: &Beatmap,
) {
    let osu_hash = match &bm.checksum {
        Some(h) if !h.is_empty() => h.clone(),
        _ => {
            state.write().await.skipped += 1;
            return;
        }
    };

    // enqueue so it's visible in pending table until processed
    let _ = db::enqueue_pending_beatmap(pool, &osu_hash, bm.id as i32).await;

    limiter.acquire().await;

    let bytes = match osu.download_osu_file(bm.id).await {
        Ok(b) => b,
        Err(e) => {
            let mut s = state.write().await;
            s.errors += 1;
            s.message = Some(format!("dl {}: {e}", bm.id));
            return;
        }
    };

    let chart = match decode_bytes("x.osu", &bytes) {
        Ok(c) => c,
        Err(e) => {
            let mut s = state.write().await;
            s.errors += 1;
            s.message = Some(format!("decode {}: {e}", bm.id));
            return;
        }
    };

    let normalized_hash = normalized_hash_100(&chart);
    let new_rox = !rox_store.contains(&normalized_hash);
    if let Err(e) = rox_store.save_if_absent(&normalized_hash, &chart) {
        let mut s = state.write().await;
        s.errors += 1;
        s.message = Some(format!("rox save {}: {e}", bm.id));
        return;
    }
    if new_rox {
        state.write().await.rox_saved += 1;
    }

    let (proportions, ratings) = match calculate_all(&chart, 100) {
        Ok(v) => v,
        Err(e) => {
            let mut s = state.write().await;
            s.errors += 1;
            s.message = Some(format!("calc {}: {e}", bm.id));
            return;
        }
    };

    let ratings_count = ratings.iter().fold(
        std::collections::HashSet::new(),
        |mut acc, (ct, _)| {
            acc.insert(ct.rating_type());
            acc
        },
    ).len() as u64;

    let computed = ComputedBeatmapData {
        proportions,
        ratings,
        normalized_hash,
    };

    match db::insert_full_beatmap(pool, set_pk, bm, &osu_hash, &computed).await {
        Ok(true) => {
            let mut s = state.write().await;
            s.maps_inserted += 1;
            s.ratings_inserted += ratings_count;
        }
        Ok(false) => {
            state.write().await.skipped += 1;
        }
        Err(e) => {
            let mut s = state.write().await;
            s.errors += 1;
            s.message = Some(format!("insert map {}: {e}", bm.id));
        }
    }
}

