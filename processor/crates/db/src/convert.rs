use chrono::NaiveDateTime;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

pub fn f64_to_decimal(value: f64) -> Decimal {
    Decimal::from_f64(value.clamp(-999_999.0, 999_999.0)).unwrap_or_default()
}

pub fn map_osu_status_to_db(status: &str) -> &'static str {
    match status {
        "ranked" | "approved" => "ranked",
        "qualified" => "qualified",
        "loved" => "loved",
        "graveyard" | "wip" => "graveyard",
        _ => "pending",
    }
}

pub fn parse_rfc3339_to_naive(value: Option<&str>) -> Option<NaiveDateTime> {
    value
        .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
        .map(|dt| dt.naive_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_to_decimal_normal_value() {
        let d = f64_to_decimal(5.5);
        assert_eq!(d.to_string(), "5.5");
    }

    #[test]
    fn test_f64_to_decimal_clamps_above_max() {
        let d = f64_to_decimal(1_000_000.0);
        assert_eq!(d, f64_to_decimal(999_999.0));
    }

    #[test]
    fn test_f64_to_decimal_clamps_below_min() {
        let d = f64_to_decimal(-1_000_000.0);
        assert_eq!(d, f64_to_decimal(-999_999.0));
    }

    #[test]
    fn test_map_osu_status_ranked() {
        assert_eq!(map_osu_status_to_db("ranked"), "ranked");
        assert_eq!(map_osu_status_to_db("approved"), "ranked");
    }

    #[test]
    fn test_map_osu_status_qualified() {
        assert_eq!(map_osu_status_to_db("qualified"), "qualified");
    }

    #[test]
    fn test_map_osu_status_loved() {
        assert_eq!(map_osu_status_to_db("loved"), "loved");
    }

    #[test]
    fn test_map_osu_status_graveyard() {
        assert_eq!(map_osu_status_to_db("graveyard"), "graveyard");
        assert_eq!(map_osu_status_to_db("wip"), "graveyard");
    }

    #[test]
    fn test_map_osu_status_unknown_falls_back_to_pending() {
        assert_eq!(map_osu_status_to_db("unknown"), "pending");
        assert_eq!(map_osu_status_to_db(""), "pending");
    }

    #[test]
    fn test_parse_rfc3339_valid() {
        let result = parse_rfc3339_to_naive(Some("2024-01-15T12:00:00Z"));
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_rfc3339_none_input() {
        assert!(parse_rfc3339_to_naive(None).is_none());
    }

    #[test]
    fn test_parse_rfc3339_invalid_string() {
        assert!(parse_rfc3339_to_naive(Some("not-a-date")).is_none());
    }
}
