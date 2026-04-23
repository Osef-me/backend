use anyhow::Result;
use bridge::osu::Beatmap;
use sqlx::{Postgres, Transaction};

use crate::convert::{f64_to_decimal, map_osu_status_to_db, parse_rfc3339_to_naive};

/// Returns the new beatmap PK if inserted, None if osu_hash already exists.
pub async fn insert_beatmap_row(
    tx: &mut Transaction<'_, Postgres>,
    beatmapset_pk: i32,
    beatmap: &Beatmap,
    osu_hash: &str,
    notes_hash: &str,
) -> Result<Option<i32>> {
    let bpm = f64_to_decimal(beatmap.bpm.unwrap_or(0.0));
    let cs = f64_to_decimal(beatmap.cs);
    let ar = f64_to_decimal(beatmap.ar);
    let od = f64_to_decimal(beatmap.accuracy);
    let hp = f64_to_decimal(beatmap.drain);
    let max_combo = beatmap.max_combo.unwrap_or(0);
    let status = map_osu_status_to_db(&beatmap.status);
    let main_pattern = serde_json::json!({});

    let play_count = beatmap.playcount.unwrap_or(0);
    let pass_count = beatmap.passcount.unwrap_or(0);
    let difficulty_rating = f64_to_decimal(beatmap.difficulty_rating.unwrap_or(0.0));
    let mapper_user_id = beatmap.user_id.map(|u| u as i32);
    let is_scoreable = beatmap.is_scoreable.unwrap_or(true);
    let is_convert = beatmap.convert.unwrap_or(false);
    let osu_last_updated = parse_rfc3339_to_naive(beatmap.last_updated.as_deref());

    let row: Option<(i32,)> = sqlx::query_as(
        r#"
        INSERT INTO beatmap
          (osu_id, beatmapset_id, difficulty, osu_hash, notes_hash,
           count_circles, count_sliders, count_spinners, max_combo,
           main_pattern, drain_time, total_time, bpm, cs, ar, od, hp,
           mode, status,
           play_count, pass_count, difficulty_rating, mapper_user_id,
           is_scoreable, is_convert, osu_last_updated, osu_ranked_code)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,3,$18,
                $19,$20,$21,$22,$23,$24,$25,$26)
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
    )
    .bind(beatmap.id as i32)
    .bind(beatmapset_pk)
    .bind(&beatmap.version)
    .bind(osu_hash)
    .bind(notes_hash)
    .bind(beatmap.count_circles)
    .bind(beatmap.count_sliders)
    .bind(beatmap.count_spinners)
    .bind(max_combo)
    .bind(main_pattern)
    .bind(beatmap.hit_length)
    .bind(beatmap.total_length)
    .bind(bpm)
    .bind(cs)
    .bind(ar)
    .bind(od)
    .bind(hp)
    .bind(status)
    .bind(play_count)
    .bind(pass_count)
    .bind(difficulty_rating)
    .bind(mapper_user_id)
    .bind(is_scoreable)
    .bind(is_convert)
    .bind(osu_last_updated)
    .bind(beatmap.ranked)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(row.map(|(id,)| id))
}

pub async fn record_duplicate_beatmap(
    tx: &mut Transaction<'_, Postgres>,
    beatmap: &Beatmap,
    osu_hash: &str,
    notes_hash: &str,
) -> Result<()> {
    let canonical_id = find_canonical_beatmap_id(tx, osu_hash, notes_hash).await?;
    if let Some(canonical_id) = canonical_id {
        insert_beatmap_duplicate_row(tx, canonical_id, beatmap, osu_hash, notes_hash).await?;
    }
    Ok(())
}

async fn find_canonical_beatmap_id(
    tx: &mut Transaction<'_, Postgres>,
    osu_hash: &str,
    notes_hash: &str,
) -> Result<Option<i32>> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT id FROM beatmap WHERE notes_hash = $1 OR osu_hash = $2 LIMIT 1",
    )
    .bind(notes_hash)
    .bind(osu_hash)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|(id,)| id))
}

async fn insert_beatmap_duplicate_row(
    tx: &mut Transaction<'_, Postgres>,
    canonical_beatmap_id: i32,
    beatmap: &Beatmap,
    osu_hash: &str,
    notes_hash: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO beatmap_duplicate (canonical_beatmap_id, osu_id, osu_hash, notes_hash)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (osu_hash) DO NOTHING
        "#,
    )
    .bind(canonical_beatmap_id)
    .bind(beatmap.id as i32)
    .bind(osu_hash)
    .bind(notes_hash)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
