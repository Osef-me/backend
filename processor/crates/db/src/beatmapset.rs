use anyhow::Result;
use bridge::osu::Beatmapset;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::convert::{f64_to_decimal, parse_rfc3339_to_naive};

pub async fn upsert_beatmapset(pool: &PgPool, beatmapset: &Beatmapset) -> Result<(i32, bool)> {
    let tags = collect_tags(&beatmapset.tags);
    let status_changed_at = parse_rfc3339_to_naive(beatmapset.last_updated.as_deref());
    let cover_url = beatmapset.covers.as_ref().and_then(|c| c.cover.as_deref());
    let submitted_at = parse_rfc3339_to_naive(beatmapset.submitted_date.as_deref());
    let ranked_at = parse_rfc3339_to_naive(beatmapset.ranked_date.as_deref());
    let set_bpm: Option<Decimal> = beatmapset.bpm.map(f64_to_decimal);
    let mapper_user_id = beatmapset.user_id.map(|u| u as i32);
    let play_count = beatmapset.play_count.unwrap_or(0);
    let favourite_count = beatmapset.favourite_count.unwrap_or(0);

    let row: (i32, bool) = sqlx::query_as(
        r#"
        INSERT INTO beatmapset
          (osu_id, artist, artist_unicode, title, title_unicode, creator, source,
           tags, has_video, has_storyboard, is_explicit, is_featured,
           cover_url, preview_url, osu_file_url, osu_status_changed_at,
           mapper_user_id, play_count, favourite_count, submitted_at, ranked_at,
           language, genre, set_bpm)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,
                $17,$18,$19,$20,$21,$22,$23,$24)
        ON CONFLICT (osu_id) DO UPDATE SET
            updated_at      = NOW(),
            mapper_user_id  = EXCLUDED.mapper_user_id,
            play_count      = EXCLUDED.play_count,
            favourite_count = EXCLUDED.favourite_count,
            submitted_at    = EXCLUDED.submitted_at,
            ranked_at       = EXCLUDED.ranked_at,
            language        = EXCLUDED.language,
            genre           = EXCLUDED.genre,
            set_bpm         = EXCLUDED.set_bpm
        RETURNING id, (xmax = 0) AS inserted
        "#,
    )
    .bind(beatmapset.id as i32)
    .bind(&beatmapset.artist)
    .bind(&beatmapset.artist_unicode)
    .bind(&beatmapset.title)
    .bind(&beatmapset.title_unicode)
    .bind(&beatmapset.creator)
    .bind(&beatmapset.source)
    .bind(&tags)
    .bind(beatmapset.video)
    .bind(beatmapset.storyboard)
    .bind(beatmapset.nsfw)
    .bind(false)
    .bind(&cover_url)
    .bind(&beatmapset.preview_url)
    .bind::<Option<String>>(None)
    .bind(status_changed_at)
    .bind(mapper_user_id)
    .bind(play_count)
    .bind(favourite_count)
    .bind(submitted_at)
    .bind(ranked_at)
    .bind(&beatmapset.language)
    .bind(&beatmapset.genre)
    .bind(set_bpm)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

fn collect_tags(tags_string: &str) -> Vec<String> {
    tags_string
        .split_whitespace()
        .map(|tag| tag.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_tags_normal() {
        let tags = collect_tags("stream jumpstream technical");
        assert_eq!(tags, vec!["stream", "jumpstream", "technical"]);
    }

    #[test]
    fn test_collect_tags_empty_string() {
        let tags = collect_tags("");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_collect_tags_extra_whitespace() {
        let tags = collect_tags("  stream   jumpstream  ");
        assert_eq!(tags, vec!["stream", "jumpstream"]);
    }
}
