use anyhow::Result;
use bridge::osu::Beatmap;
use bridge::{CalcResult, CalcType, Proportions};
use sqlx::PgPool;

use crate::beatmap::{insert_beatmap_row, record_duplicate_beatmap};
use crate::pending::remove_pending_beatmap;
use crate::ratings::{insert_mania_ratio, insert_unique_ratings};

pub struct ComputedBeatmapData {
    pub proportions: Proportions,
    pub ratings: Vec<(CalcType, CalcResult)>,
    pub normalized_hash: String,
}

/// Inserts a fully processed beatmap in a single transaction.
/// Returns true if newly inserted, false if duplicate (osu_hash conflict).
pub async fn insert_full_beatmap(
    pool: &PgPool,
    beatmapset_pk: i32,
    beatmap: &Beatmap,
    osu_hash: &str,
    computed: &ComputedBeatmapData,
) -> Result<bool> {
    let mut tx = pool.begin().await?;

    let beatmap_pk = insert_beatmap_row(
        &mut tx,
        beatmapset_pk,
        beatmap,
        osu_hash,
        &computed.normalized_hash,
    )
    .await?;

    match beatmap_pk {
        None => {
            record_duplicate_beatmap(&mut tx, beatmap, osu_hash, &computed.normalized_hash).await?;
            tx.commit().await?;
            Ok(false)
        }
        Some(pk) => {
            insert_mania_ratio(&mut tx, pk, &computed.proportions).await?;
            insert_unique_ratings(&mut tx, pk, &computed.ratings).await?;
            remove_pending_beatmap(&mut tx, osu_hash).await?;
            tx.commit().await?;
            Ok(true)
        }
    }
}
