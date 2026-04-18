use anyhow::Result;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

#[derive(Debug, FromRow)]
pub struct PendingBeatmap {
    pub id: i32,
    pub osu_hash: String,
    pub osu_id: Option<i32>,
}

pub async fn enqueue_pending_beatmap(pool: &PgPool, osu_hash: &str, osu_id: i32) -> Result<()> {
    sqlx::query(
        "INSERT INTO pending_beatmap (osu_hash, osu_id)
         VALUES ($1, $2)
         ON CONFLICT (osu_hash) DO NOTHING",
    )
    .bind(osu_hash)
    .bind(osu_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_pending_beatmap(tx: &mut Transaction<'_, Postgres>, osu_hash: &str) -> Result<()> {
    sqlx::query("DELETE FROM pending_beatmap WHERE osu_hash = $1")
        .bind(osu_hash)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn fetch_pending_beatmaps(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<PendingBeatmap>> {
    let rows = sqlx::query_as::<_, PendingBeatmap>(
        "SELECT id, osu_hash, osu_id FROM pending_beatmap ORDER BY created_at ASC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn remove_pending_by_hash(pool: &PgPool, osu_hash: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM pending_beatmap WHERE osu_hash = $1")
        .bind(osu_hash)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
