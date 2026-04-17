mod beatmap;
mod beatmapset;
mod convert;
mod insert;
mod pending;
mod ratings;
pub mod pool;

pub use beatmapset::upsert_beatmapset;
pub use insert::{insert_full_beatmap, ComputedBeatmapData};
pub use pending::enqueue_pending_beatmap;
pub use pool::{connect, run_migrations};
pub use sqlx::PgPool;

pub async fn query_db_size(pool: &PgPool) -> anyhow::Result<u64> {
    let row: (i64,) = sqlx::query_as("SELECT pg_database_size(current_database())")
        .fetch_one(pool)
        .await?;
    Ok(row.0 as u64)
}
