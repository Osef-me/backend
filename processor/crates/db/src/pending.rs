use anyhow::Result;
use sqlx::{PgPool, Postgres, Transaction};

pub async fn enqueue_pending_beatmap(
    pool: &PgPool,
    osu_hash: &str,
    osu_id: i32,
) -> Result<()> {
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

pub async fn remove_pending_beatmap(
    tx: &mut Transaction<'_, Postgres>,
    osu_hash: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM pending_beatmap WHERE osu_hash = $1")
        .bind(osu_hash)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
