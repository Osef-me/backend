use anyhow::Result;
use bridge::{CalcResult, CalcType, ManiaSkill, Proportions};
use sqlx::{Postgres, Transaction};

use crate::convert::f64_to_decimal;

pub async fn insert_mania_ratio(
    tx: &mut Transaction<'_, Postgres>,
    beatmap_pk: i32,
    proportions: &Proportions,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO beatmap_mania_ratio
          (beatmap_id, stream, jumpstream, handstream, stamina, jackspeed, chordjack, technical)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ON CONFLICT (beatmap_id) DO NOTHING
        "#,
    )
    .bind(beatmap_pk)
    .bind(f64_to_decimal(proportions.stream))
    .bind(f64_to_decimal(proportions.jumpstream))
    .bind(f64_to_decimal(proportions.handstream))
    .bind(f64_to_decimal(proportions.stamina))
    .bind(f64_to_decimal(proportions.jackspeed))
    .bind(f64_to_decimal(proportions.chordjack))
    .bind(f64_to_decimal(proportions.technical))
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Insert one rating row + its mania_skill breakdown.
/// Skips calc types that share a rating_type already inserted (e.g. osu2016/2018/current → "osu").
pub async fn insert_unique_ratings(
    tx: &mut Transaction<'_, Postgres>,
    beatmap_pk: i32,
    ratings: &[(CalcType, CalcResult)],
) -> Result<()> {
    let mut inserted_rating_types = std::collections::HashSet::new();
    for (calc_type, result) in ratings {
        let rating_type = calc_type.rating_type();
        if inserted_rating_types.contains(rating_type) {
            continue;
        }
        insert_rating_and_skill(tx, beatmap_pk, rating_type, result).await?;
        inserted_rating_types.insert(rating_type);
    }
    Ok(())
}

async fn insert_rating_and_skill(
    tx: &mut Transaction<'_, Postgres>,
    beatmap_pk: i32,
    rating_type: &str,
    result: &CalcResult,
) -> Result<()> {
    let rating_pk = insert_beatmap_rating_row(tx, beatmap_pk, rating_type, result.rating).await?;
    insert_mania_skill_for_rating(tx, rating_pk, &result.mania_skill).await?;
    Ok(())
}

async fn insert_beatmap_rating_row(
    tx: &mut Transaction<'_, Postgres>,
    beatmap_pk: i32,
    rating_type: &str,
    rating: f64,
) -> Result<i32> {
    let row: (i32,) = sqlx::query_as(
        r#"
        INSERT INTO beatmap_rating (beatmap_id, rating, rating_type)
        VALUES ($1, $2, $3)
        ON CONFLICT (beatmap_id, rating_type) DO UPDATE SET rating = EXCLUDED.rating
        RETURNING id
        "#,
    )
    .bind(beatmap_pk)
    .bind(f64_to_decimal(rating))
    .bind(rating_type)
    .fetch_one(&mut **tx)
    .await?;
    Ok(row.0)
}

async fn insert_mania_skill_for_rating(
    tx: &mut Transaction<'_, Postgres>,
    rating_pk: i32,
    skill: &ManiaSkill,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO beatmap_mania_skill
          (rating_id, stream, jumpstream, handstream, stamina, jackspeed, chordjack, technical)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ON CONFLICT (rating_id) DO UPDATE SET
          stream     = EXCLUDED.stream,
          jumpstream = EXCLUDED.jumpstream,
          handstream = EXCLUDED.handstream,
          stamina    = EXCLUDED.stamina,
          jackspeed  = EXCLUDED.jackspeed,
          chordjack  = EXCLUDED.chordjack,
          technical  = EXCLUDED.technical
        "#,
    )
    .bind(rating_pk)
    .bind(f64_to_decimal(skill.stream))
    .bind(f64_to_decimal(skill.jumpstream))
    .bind(f64_to_decimal(skill.handstream))
    .bind(f64_to_decimal(skill.stamina))
    .bind(f64_to_decimal(skill.jackspeed))
    .bind(f64_to_decimal(skill.chordjack))
    .bind(f64_to_decimal(skill.technical))
    .execute(&mut **tx)
    .await?;
    Ok(())
}
