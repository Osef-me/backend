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

/// Inserts one rating + skill per unique `rating_type`.
/// Skips calc types that share a `rating_type` already inserted (e.g. osu2016/2018/current → "osu").
pub async fn insert_unique_ratings(
    tx: &mut Transaction<'_, Postgres>,
    beatmap_pk: i32,
    ratings: &[(CalcType, CalcResult)],
) -> Result<()> {
    for (calc_type, result) in select_first_per_rating_type(ratings) {
        insert_rating_and_skill(tx, beatmap_pk, calc_type.rating_type(), result).await?;
    }
    Ok(())
}

fn select_first_per_rating_type(ratings: &[(CalcType, CalcResult)]) -> Vec<&(CalcType, CalcResult)> {
    let mut seen = std::collections::HashSet::new();
    ratings
        .iter()
        .filter(|(calc_type, _)| seen.insert(calc_type.rating_type()))
        .collect()
}

async fn insert_rating_and_skill(
    tx: &mut Transaction<'_, Postgres>,
    beatmap_pk: i32,
    rating_type: &str,
    result: &CalcResult,
) -> Result<()> {
    let rating_pk = insert_beatmap_rating_row(tx, beatmap_pk, rating_type, result.rating).await?;
    insert_mania_skill_for_rating(tx, rating_pk, &result.mania_skill).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use bridge::ManiaSkill;

    fn dummy_result() -> CalcResult {
        CalcResult {
            rating: 1.0,
            mania_skill: ManiaSkill {
                stream: 0.0,
                jumpstream: 0.0,
                handstream: 0.0,
                stamina: 0.0,
                jackspeed: 0.0,
                chordjack: 0.0,
                technical: 0.0,
            },
        }
    }

    #[test]
    fn select_first_per_rating_type_deduplicates_osu_variants() {
        let ratings = vec![
            (CalcType::Osu2016, dummy_result()),
            (CalcType::Osu2018, dummy_result()),
            (CalcType::OsuCurrent, dummy_result()),
            (CalcType::Quaver2025, dummy_result()),
            (CalcType::Etterna, dummy_result()),
        ];
        let selected = select_first_per_rating_type(&ratings);
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].0, CalcType::Osu2016, "first osu variant wins");
        assert_eq!(selected[1].0, CalcType::Quaver2025);
        assert_eq!(selected[2].0, CalcType::Etterna);
    }

    #[test]
    fn select_first_per_rating_type_empty_input() {
        let selected = select_first_per_rating_type(&[]);
        assert!(selected.is_empty());
    }

    #[test]
    fn select_first_per_rating_type_all_unique() {
        let ratings = vec![
            (CalcType::Osu2016, dummy_result()),
            (CalcType::Quaver2025, dummy_result()),
            (CalcType::Interlude2025, dummy_result()),
            (CalcType::SunnyXXY, dummy_result()),
            (CalcType::Etterna, dummy_result()),
        ];
        let selected = select_first_per_rating_type(&ratings);
        assert_eq!(selected.len(), ratings.len());
    }
}
