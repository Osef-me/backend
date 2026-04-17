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
}

#[derive(Debug, Clone)]
pub struct CalcResult {
    pub rating: f64,
    pub mania_skill: ManiaSkill,
}

struct Proportions {
    overall: f64,
    stream: f64,
    jumpstream: f64,
    handstream: f64,
    stamina: f64,
    jackspeed: f64,
    chordjack: f64,
    technical: f64,
}

fn minacalc_at(chart: &RoxChart, centirate: u32) -> Result<Proportions, String> {
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

pub fn calculate_one(chart: &RoxChart, calc_type: &CalcType, centirate: u32) -> Result<CalcResult, String> {
    let proportions = minacalc_at(chart, centirate)?;

    let (rating, mania_skill) = if matches!(calc_type, CalcType::Etterna) {
        let r = proportions.overall;
        let s = ManiaSkill {
            stream:     proportions.stream * r,
            jumpstream: proportions.jumpstream * r,
            handstream: proportions.handstream * r,
            stamina:    proportions.stamina * r,
            jackspeed:  proportions.jackspeed * r,
            chordjack:  proportions.chordjack * r,
            technical:  proportions.technical * r,
        };
        (r, s)
    } else {
        let raw = raw_rating(chart, calc_type, centirate)?;
        let s = ManiaSkill {
            stream:     raw * proportions.stream,
            jumpstream: raw * proportions.jumpstream,
            handstream: raw * proportions.handstream,
            stamina:    raw * proportions.stamina,
            jackspeed:  raw * proportions.jackspeed,
            chordjack:  raw * proportions.chordjack,
            technical:  raw * proportions.technical,
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
            .join("../../crates/metron/assets/test.osu");
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
    fn test_mania_skills_positive() {
        let chart = test_chart();
        let result = calculate_one(&chart, &CalcType::Osu2018, 100).unwrap();
        assert!(result.mania_skill.stream > 0.0);
        assert!(result.mania_skill.jumpstream > 0.0);
    }

    #[test]
    fn test_higher_rate_higher_rating() {
        let chart = test_chart();
        let r100 = calculate_one(&chart, &CalcType::Etterna, 100).unwrap().rating;
        let r150 = calculate_one(&chart, &CalcType::Etterna, 150).unwrap().rating;
        assert!(r150 > r100, "150% should be harder than 100%");
    }

    #[test]
    fn test_invalid_calc_type() {
        assert!(CalcType::from_str("leyna").is_none());
        assert!(CalcType::from_str("unknown").is_none());
    }
}
