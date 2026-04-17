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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalcType {
    Osu2016,
    Osu2018,
    OsuCurrent,
    Quaver2025,
    Interlude2025,
    SunnyXXY,
    Etterna,
}

pub const ALL_CALC_TYPES: &[CalcType] = &[
    CalcType::Osu2016,
    CalcType::Osu2018,
    CalcType::OsuCurrent,
    CalcType::Quaver2025,
    CalcType::Interlude2025,
    CalcType::SunnyXXY,
    CalcType::Etterna,
];

impl CalcType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "osu2016"       => Some(Self::Osu2016),
            "osu2018"       => Some(Self::Osu2018),
            "osu_current"   => Some(Self::OsuCurrent),
            "quaver2025"    => Some(Self::Quaver2025),
            "interlude2025" => Some(Self::Interlude2025),
            "sunnyxxy"      => Some(Self::SunnyXXY),
            "etterna"       => Some(Self::Etterna),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Osu2016       => "osu2016",
            Self::Osu2018       => "osu2018",
            Self::OsuCurrent    => "osu_current",
            Self::Quaver2025    => "quaver2025",
            Self::Interlude2025 => "interlude2025",
            Self::SunnyXXY      => "sunnyxxy",
            Self::Etterna       => "etterna",
        }
    }

    /// Maps CalcType to a `rating_type` string accepted by the DB schema.
    /// Schema CHECK: ('osu','etterna','quaver','malody','interlude','sunnyxxy')
    pub fn rating_type(&self) -> &'static str {
        match self {
            Self::Osu2016 | Self::Osu2018 | Self::OsuCurrent => "osu",
            Self::Quaver2025    => "quaver",
            Self::Interlude2025 => "interlude",
            Self::SunnyXXY      => "sunnyxxy",
            Self::Etterna       => "etterna",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CalcResult {
    pub rating: f64,
    pub mania_skill: ManiaSkill,
}

pub struct Proportions {
    pub overall: f64,
    pub stream: f64,
    pub jumpstream: f64,
    pub handstream: f64,
    pub stamina: f64,
    pub jackspeed: f64,
    pub chordjack: f64,
    pub technical: f64,
}

pub fn minacalc_at(chart: &RoxChart, centirate: u32) -> Result<Proportions, String> {
    let clock_rate = ClockRate::from_percentage(centirate)
        .map_err(|e| e.to_string())?;
    let ctx = MinaCalcDifficultyContext { clock_rate, mode: CalcMode::Msd };
    let d = MinaCalc515
        .calculate_difficulty(chart, &ctx)
        .map_err(|e| e.to_string())?;
    let overall = d.overall as f64;
    if overall <= 0.0 {
        return Err("minacalc returned zero overall".into());
    }
    Ok(Proportions {
        overall,
        stream:     d.stream as f64 / overall,
        jumpstream: d.jumpstream as f64 / overall,
        handstream: d.handstream as f64 / overall,
        stamina:    d.stamina as f64 / overall,
        jackspeed:  d.jackspeed as f64 / overall,
        chordjack:  d.chordjack as f64 / overall,
        technical:  d.technical as f64 / overall,
    })
}

fn raw_rating(chart: &RoxChart, calc_type: &CalcType, centirate: u32) -> Result<f64, String> {
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
        CalcType::Etterna => unreachable!("etterna handled separately"),
    }
}

/// Batch variant: compute minacalc proportions once, derive all calc-type ratings.
pub fn calculate_all(
    chart: &RoxChart,
    centirate: u32,
) -> Result<(Proportions, Vec<(CalcType, CalcResult)>), String> {
    let p = minacalc_at(chart, centirate)?;
    let mut out = Vec::with_capacity(ALL_CALC_TYPES.len());
    for ct in ALL_CALC_TYPES {
        let (rating, mania_skill) = if matches!(ct, CalcType::Etterna) {
            let r = p.overall;
            let s = ManiaSkill {
                stream:     p.stream * r,
                jumpstream: p.jumpstream * r,
                handstream: p.handstream * r,
                stamina:    p.stamina * r,
                jackspeed:  p.jackspeed * r,
                chordjack:  p.chordjack * r,
                technical:  p.technical * r,
            };
            (r, s)
        } else {
            let raw = raw_rating(chart, ct, centirate)?;
            let s = ManiaSkill {
                stream:     raw * p.stream,
                jumpstream: raw * p.jumpstream,
                handstream: raw * p.handstream,
                stamina:    raw * p.stamina,
                jackspeed:  raw * p.jackspeed,
                chordjack:  raw * p.chordjack,
                technical:  raw * p.technical,
            };
            (raw, s)
        };
        out.push((ct.clone(), CalcResult { rating, mania_skill }));
    }
    Ok((p, out))
}

pub fn calculate_one(chart: &RoxChart, calc_type: &CalcType, centirate: u32) -> Result<CalcResult, String> {
    let p = minacalc_at(chart, centirate)?;

    let (rating, mania_skill) = if matches!(calc_type, CalcType::Etterna) {
        let r = p.overall;
        let s = ManiaSkill {
            stream:     p.stream * r,
            jumpstream: p.jumpstream * r,
            handstream: p.handstream * r,
            stamina:    p.stamina * r,
            jackspeed:  p.jackspeed * r,
            chordjack:  p.chordjack * r,
            technical:  p.technical * r,
        };
        (r, s)
    } else {
        let raw = raw_rating(chart, calc_type, centirate)?;
        let s = ManiaSkill {
            stream:     raw * p.stream,
            jumpstream: raw * p.jumpstream,
            handstream: raw * p.handstream,
            stamina:    raw * p.stamina,
            jackspeed:  raw * p.jackspeed,
            chordjack:  raw * p.chordjack,
            technical:  raw * p.technical,
        };
        (raw, s)
    };

    Ok(CalcResult { rating, mania_skill })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rox_formats::auto::auto_decode;

    fn test_chart() -> RoxChart {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../metron/assets/test.osu");
        auto_decode(path).expect("decode test.osu")
    }

    #[test]
    fn test_etterna_rating_positive() {
        let chart = test_chart();
        let result = calculate_one(&chart, &CalcType::Etterna, 100).unwrap();
        assert!(result.rating > 0.0);
    }

    #[test]
    fn test_osu2018_rating_positive() {
        let chart = test_chart();
        let result = calculate_one(&chart, &CalcType::Osu2018, 100).unwrap();
        assert!(result.rating > 0.0);
    }

    #[test]
    fn test_higher_rate_higher_rating() {
        let chart = test_chart();
        let r100 = calculate_one(&chart, &CalcType::Etterna, 100).unwrap().rating;
        let r150 = calculate_one(&chart, &CalcType::Etterna, 150).unwrap().rating;
        assert!(r150 > r100);
    }
}
