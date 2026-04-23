use std::time::Duration;

const MAX_BACKOFF_SECS: u64 = 60;
const BASE_DELAY_MS: u64 = 1000;

pub fn compute_backoff(attempt: u32) -> Duration {
    let secs = (BASE_DELAY_MS as f64 / 1000.0 * 2_f64.powi(attempt as i32 - 1)).min(MAX_BACKOFF_SECS as f64);
    Duration::from_secs_f64(secs)
}

pub fn is_transient(msg: &str) -> bool {
    let msg = msg.to_lowercase();
    is_transient_message(&msg)
}

pub fn is_rate_limited(msg: &str) -> bool {
    let msg = msg.to_lowercase();
    is_rate_limited_message(&msg)
}

fn is_transient_message(msg: &str) -> bool {
    msg.contains("connection")
        || msg.contains("timeout")
        || msg.contains("timed out")
        || msg.contains("network")
        || msg.contains("broken pipe")
        || msg.contains("reset by peer")
        || msg.contains("temporarily unavailable")
        || is_rate_limited_message(msg)
        || msg.contains("503")
        || msg.contains("502")
        || msg.contains("504")
}

fn is_rate_limited_message(msg: &str) -> bool {
    msg.contains("429")
        || msg.contains("too many requests")
        || msg.contains("rate limit")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_increases_exponentially() {
        assert_eq!(compute_backoff(1), Duration::from_secs(1));
        assert_eq!(compute_backoff(2), Duration::from_secs(2));
        assert_eq!(compute_backoff(3), Duration::from_secs(4));
        assert_eq!(compute_backoff(4), Duration::from_secs(8));
    }

    #[test]
    fn backoff_caps_at_max() {
        assert_eq!(compute_backoff(10), Duration::from_secs(MAX_BACKOFF_SECS));
        assert_eq!(compute_backoff(20), Duration::from_secs(MAX_BACKOFF_SECS));
    }

    #[test]
    fn connection_error_is_transient() {
        assert!(is_transient_message("connection refused"));
        assert!(is_transient_message("connection reset by peer"));
        assert!(is_transient_message("network unreachable"));
        assert!(is_transient_message("timeout"));
    }

    #[test]
    fn decode_error_is_not_transient() {
        assert!(!is_transient_message("decode error: invalid format"));
        assert!(!is_transient_message("not found"));
        assert!(!is_transient_message("invalid checksum"));
    }

    #[test]
    fn detects_rate_limit() {
        assert!(is_rate_limited("429 too many requests"));
        assert!(is_rate_limited("dl 123: osu file 429 Too Many Requests"));
        assert!(is_rate_limited("rate limit exceeded"));
        assert!(!is_rate_limited("connection refused"));
        assert!(!is_rate_limited("500 internal server error"));
    }
}
