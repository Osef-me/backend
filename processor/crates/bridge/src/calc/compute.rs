use metron_rs::{
    calculator::Calculator,
    clock_rate::ClockRate,
    etterna::minacalc515::{MinaCalc515, MinaCalcDifficultyContext},
    interlude::interlude2025::{Interlude2025, Interlude2025DifficultyContext},
    osu::{
        osu2016::Osu2016,
        osu2018::{Osu2018, Osu2018DifficultyContext},
        osu_current::OsuCurrent,
    },
    custom::sunnyxxy::{SunnyXXY, SunnyxxyDifficultyContext},
    quaver::quaver2025::difficulty::{Quaver2025, QuaverDifficultyContext},
};
use minacalc_rs::CalcMode;
use rox::model::RoxChart;

use crate::types::ManiaSkill;

use super::types::{ALL_CALC_TYPES, CalcResult, CalcType, Proportions};

pub fn minacalc_at(chart: &RoxChart, centirate: u32) -> Result<Proportions, String> {
    let clock_rate = ClockRate::from_percentage(centirate).map_err(|e| e.to_string())?;
    let ctx = MinaCalcDifficultyContext { clock_rate, mode: CalcMode::Msd };
    let difficulty = MinaCalc515
        .calculate_difficulty(chart, &ctx)
        .map_err(|e| e.to_string())?;
    let overall = difficulty.overall as f64;
    if overall <= 0.0 {
        return Err("minacalc returned zero overall".into());
    }
    Ok(Proportions {
        overall,
        stream:     difficulty.stream as f64 / overall,
        jumpstream: difficulty.jumpstream as f64 / overall,
        handstream: difficulty.handstream as f64 / overall,
        stamina:    difficulty.stamina as f64 / overall,
        jackspeed:  difficulty.jackspeed as f64 / overall,
        chordjack:  difficulty.chordjack as f64 / overall,
        technical:  difficulty.technical as f64 / overall,
    })
}

fn raw_rating(chart: &RoxChart, calc_type: CalcType, centirate: u32) -> Result<f64, String> {
    let clock_rate = ClockRate::from_percentage(centirate).map_err(|e| e.to_string())?;
    let od = chart.metadata.difficulty_value;
    match calc_type {
        CalcType::Osu2016 => {
            let ctx = Osu2018DifficultyContext { clock_rate: Some(clock_rate), overall_difficulty: od };
            Ok(Osu2016.calculate_difficulty(chart, &ctx).map_err(|e| e.to_string())?.stars)
        }
        CalcType::Osu2018 => {
            let ctx = Osu2018DifficultyContext { clock_rate: Some(clock_rate), overall_difficulty: od };
            Ok(Osu2018.calculate_difficulty(chart, &ctx).map_err(|e| e.to_string())?.stars)
        }
        CalcType::OsuCurrent => {
            let ctx = Osu2018DifficultyContext { clock_rate: Some(clock_rate), overall_difficulty: od };
            Ok(OsuCurrent.calculate_difficulty(chart, &ctx).map_err(|e| e.to_string())?.stars)
        }
        CalcType::Quaver2025 => {
            let ctx = QuaverDifficultyContext { clock_rate };
            Ok(Quaver2025.calculate_difficulty(chart, &ctx).map_err(|e| e.to_string())?.stars)
        }
        CalcType::Interlude2025 => {
            let ctx = Interlude2025DifficultyContext { clock_rate: Some(clock_rate) };
            Ok(Interlude2025.calculate_difficulty(chart, &ctx).map_err(|e| e.to_string())?.stars)
        }
        CalcType::SunnyXXY => {
            let ctx = SunnyxxyDifficultyContext { clock_rate: Some(clock_rate), overall_difficulty: od };
            Ok(SunnyXXY.calculate_difficulty(chart, &ctx).map_err(|e| e.to_string())?.stars)
        }
        CalcType::Etterna => unreachable!("etterna handled separately in compute_result_for_calc_type"),
    }
}

fn build_mania_skill(proportions: &Proportions, scale: f64) -> ManiaSkill {
    ManiaSkill {
        stream:     proportions.stream * scale,
        jumpstream: proportions.jumpstream * scale,
        handstream: proportions.handstream * scale,
        stamina:    proportions.stamina * scale,
        jackspeed:  proportions.jackspeed * scale,
        chordjack:  proportions.chordjack * scale,
        technical:  proportions.technical * scale,
    }
}

fn compute_result_for_calc_type(
    chart: &RoxChart,
    calc_type: CalcType,
    centirate: u32,
    proportions: &Proportions,
) -> Result<CalcResult, String> {
    let (rating, mania_skill) = if matches!(calc_type, CalcType::Etterna) {
        let rating = proportions.overall;
        (rating, build_mania_skill(proportions, rating))
    } else {
        let rating = raw_rating(chart, calc_type, centirate)?;
        (rating, build_mania_skill(proportions, rating))
    };
    Ok(CalcResult { rating, mania_skill })
}

/// Computes minacalc proportions once, then derives all calc-type ratings from them.
pub fn calculate_all(
    chart: &RoxChart,
    centirate: u32,
) -> Result<(Proportions, Vec<(CalcType, CalcResult)>), String> {
    let proportions = minacalc_at(chart, centirate)?;
    let mut results = Vec::with_capacity(ALL_CALC_TYPES.len());
    for &calc_type in ALL_CALC_TYPES {
        results.push((calc_type, compute_result_for_calc_type(chart, calc_type, centirate, &proportions)?));
    }
    Ok((proportions, results))
}

pub fn calculate_one(chart: &RoxChart, calc_type: CalcType, centirate: u32) -> Result<CalcResult, String> {
    let proportions = minacalc_at(chart, centirate)?;
    compute_result_for_calc_type(chart, calc_type, centirate, &proportions)
}
