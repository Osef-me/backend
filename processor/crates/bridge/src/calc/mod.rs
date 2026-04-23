mod compute;
mod types;

pub use compute::{calculate_all, calculate_one, minacalc_at};
pub use types::{CalcResult, CalcType, Proportions, ALL_CALC_TYPES};

#[cfg(test)]
mod tests {
    use super::*;
    use rox_formats::auto::auto_decode;

    fn test_chart() -> rox::model::RoxChart {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../metron/assets/test.osu");
        auto_decode(path).expect("decode test.osu")
    }

    #[test]
    fn calculate_one_etterna_positive() {
        let chart = test_chart();
        let result = calculate_one(&chart, CalcType::Etterna, 100).unwrap();
        assert!(result.rating > 0.0);
    }

    #[test]
    fn calculate_one_osu2016_positive() {
        let chart = test_chart();
        let result = calculate_one(&chart, CalcType::Osu2016, 100).unwrap();
        assert!(result.rating > 0.0);
    }

    #[test]
    fn higher_rate_yields_higher_etterna_rating() {
        let chart = test_chart();
        let r100 = calculate_one(&chart, CalcType::Etterna, 100).unwrap().rating;
        let r150 = calculate_one(&chart, CalcType::Etterna, 150).unwrap().rating;
        assert!(r150 > r100);
    }

    #[test]
    fn calculate_all_returns_all_calc_types() {
        let chart = test_chart();
        let (_, results) = calculate_all(&chart, 100).unwrap();
        assert_eq!(results.len(), ALL_CALC_TYPES.len());
        for (i, (calc_type, result)) in results.iter().enumerate() {
            assert_eq!(*calc_type, ALL_CALC_TYPES[i]);
            assert!(result.rating > 0.0, "rating must be positive for {calc_type}");
        }
    }

    #[test]
    fn calculate_all_etterna_rating_matches_minacalc_overall() {
        let chart = test_chart();
        let (proportions, results) = calculate_all(&chart, 100).unwrap();
        let etterna = results.iter().find(|(ct, _)| *ct == CalcType::Etterna).unwrap();
        let delta = (etterna.1.rating - proportions.overall).abs();
        assert!(delta < 1e-9, "etterna rating must equal minacalc overall");
    }

    #[test]
    fn minacalc_proportions_are_non_negative() {
        let chart = test_chart();
        let p = minacalc_at(&chart, 100).unwrap();
        assert!(p.overall > 0.0);
        assert!(p.stream >= 0.0);
        assert!(p.jumpstream >= 0.0);
        assert!(p.handstream >= 0.0);
        assert!(p.stamina >= 0.0);
        assert!(p.jackspeed >= 0.0);
        assert!(p.chordjack >= 0.0);
        assert!(p.technical >= 0.0);
    }
}
