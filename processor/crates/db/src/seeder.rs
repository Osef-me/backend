use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeederProgress {
    pub cursor: Option<String>,
    pub total_known: Option<u64>,
    pub sets_inserted: u64,
    pub maps_inserted: u64,
    pub ratings_inserted: u64,
    pub rox_saved: u64,
    pub rox_bytes: u64,
    pub pages_fetched: u64,
    pub errors: u64,
    pub skipped: u64,
}

pub async fn load_seeder_progress(pool: &PgPool) -> Result<Option<SeederProgress>> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM seeder_state WHERE key = 'progress'",
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some((value,)) if !value.is_empty() => {
            let progress: SeederProgress = serde_json::from_str(&value)?;
            Ok(Some(progress))
        }
        _ => Ok(None),
    }
}

pub async fn save_seeder_progress(pool: &PgPool, progress: &SeederProgress) -> Result<()> {
    let value = serde_json::to_string(progress)?;
    sqlx::query(
        "INSERT INTO seeder_state (key, value, updated_at)
         VALUES ('progress', $1, NOW())
         ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()",
    )
    .bind(&value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn clear_seeder_progress(pool: &PgPool) -> Result<()> {
    sqlx::query("DELETE FROM seeder_state WHERE key = 'progress'")
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_serializes_and_deserializes() {
        let progress = SeederProgress {
            cursor: Some("abc123".into()),
            sets_inserted: 1000,
            maps_inserted: 5000,
            ..Default::default()
        };
        let json = serde_json::to_string(&progress).unwrap();
        let restored: SeederProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.sets_inserted, 1000);
        assert_eq!(restored.cursor, Some("abc123".into()));
    }
}
