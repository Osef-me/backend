use std::fmt;
use std::str::FromStr;

use crate::types::ManiaSkill;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalcType {
    Osu2016,
    OsuCurrent,
    Quaver2025,
    Interlude2025,
    SunnyXXY,
    Etterna,
}

pub const ALL_CALC_TYPES: &[CalcType] = &[
    CalcType::Osu2016,
    CalcType::OsuCurrent,
    CalcType::Quaver2025,
    CalcType::Interlude2025,
    CalcType::SunnyXXY,
    CalcType::Etterna,
];

impl CalcType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Osu2016       => "osu2016",
            Self::OsuCurrent    => "osu_current",
            Self::Quaver2025    => "quaver2025",
            Self::Interlude2025 => "interlude2025",
            Self::SunnyXXY      => "sunnyxxy",
            Self::Etterna       => "etterna",
        }
    }

    /// Maps to the `rating_type` CHECK constraint in the DB schema.
    pub fn rating_type(&self) -> &'static str {
        match self {
            Self::Osu2016       => "osu2016",
            Self::OsuCurrent    => "osu_current",
            Self::Quaver2025    => "quaver",
            Self::Interlude2025 => "interlude",
            Self::SunnyXXY      => "sunnyxxy",
            Self::Etterna       => "etterna",
        }
    }
}

impl FromStr for CalcType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "osu2016"       => Ok(Self::Osu2016),
            "osu_current"   => Ok(Self::OsuCurrent),
            "quaver2025"    => Ok(Self::Quaver2025),
            "interlude2025" => Ok(Self::Interlude2025),
            "sunnyxxy"      => Ok(Self::SunnyXXY),
            "etterna"       => Ok(Self::Etterna),
            _               => Err(format!("unknown calc_type: {s}")),
        }
    }
}

impl fmt::Display for CalcType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_type_round_trips_through_str() {
        for calc_type in ALL_CALC_TYPES {
            let parsed: CalcType = calc_type.as_str().parse().unwrap();
            assert_eq!(parsed, *calc_type);
        }
    }

    #[test]
    fn calc_type_display_matches_as_str() {
        for calc_type in ALL_CALC_TYPES {
            assert_eq!(calc_type.to_string(), calc_type.as_str());
        }
    }

    #[test]
    fn unknown_calc_type_returns_error() {
        assert!("unknown".parse::<CalcType>().is_err());
    }

    #[test]
    fn osu_variants_have_distinct_rating_types() {
        assert_eq!(CalcType::Osu2016.rating_type(), "osu2016");
        assert_eq!(CalcType::OsuCurrent.rating_type(), "osu_current");
    }

    #[test]
    fn all_calc_types_have_distinct_rating_types() {
        let types: Vec<_> = ALL_CALC_TYPES.iter().map(|ct| ct.rating_type()).collect();
        let unique: std::collections::HashSet<_> = types.iter().collect();
        assert_eq!(types.len(), unique.len(), "every CalcType must have a distinct rating_type");
    }
}
