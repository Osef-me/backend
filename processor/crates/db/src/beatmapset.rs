use anyhow::Result;
use bridge::osu::Beatmapset;
use sqlx::PgPool;

use crate::convert::parse_rfc3339_to_naive;

pub async fn upsert_beatmapset(pool: &PgPool, beatmapset: &Beatmapset) -> Result<(i32, bool)> {
    let tags = collect_tags(&beatmapset.tags);
    let status_changed_at = parse_rfc3339_to_naive(beatmapset.last_updated.as_deref());
    let cover_url = beatmapset.covers.as_ref().and_then(|c| c.cover.as_deref());

    let row: (i32, bool) = sqlx::query_as(
        r#"
        INSERT INTO beatmapset
          (osu_id, artist, artist_unicode, title, title_unicode, creator, source,
           tags, has_video, has_storyboard, is_explicit, is_featured,
           cover_url, preview_url, osu_file_url, osu_status_changed_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
        ON CONFLICT (osu_id) DO UPDATE SET updated_at = NOW()
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
