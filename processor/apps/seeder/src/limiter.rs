use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, Instant};

pub struct Limiter {
    rate_per_min: Arc<AtomicU32>,
    last: tokio::sync::Mutex<Option<Instant>>,
}

impl Limiter {
    pub fn new(rate_per_min: u32) -> Arc<Self> {
        Arc::new(Self {
            rate_per_min: Arc::new(AtomicU32::new(rate_per_min)),
            last: tokio::sync::Mutex::new(None),
        })
    }

    pub fn set_rate(&self, rate: u32) {
        self.rate_per_min.store(rate, Ordering::Relaxed);
    }

    pub fn rate(&self) -> u32 {
        self.rate_per_min.load(Ordering::Relaxed)
    }

    pub async fn acquire(&self) {
        let rate = self.rate().max(1);
        let interval = Duration::from_secs_f64(60.0 / rate as f64);
        let mut last_guard = self.last.lock().await;
        let now = Instant::now();
        if let Some(prev) = *last_guard {
            let elapsed = now.saturating_duration_since(prev);
            if let Some(wait) = compute_required_wait(elapsed, interval) {
                drop(last_guard);
                sleep(wait).await;
                let mut last_instant_guard = self.last.lock().await;
                *last_instant_guard = Some(Instant::now());
                return;
            }
        }
        *last_guard = Some(now);
    }
}

fn compute_required_wait(elapsed: Duration, interval: Duration) -> Option<Duration> {
    if elapsed < interval {
        Some(interval - elapsed)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_wait_when_elapsed_exceeds_interval() {
        let result = compute_required_wait(Duration::from_millis(600), Duration::from_millis(500));
        assert_eq!(result, None);
    }

    #[test]
    fn no_wait_when_elapsed_equals_interval() {
        let result = compute_required_wait(Duration::from_millis(500), Duration::from_millis(500));
        assert_eq!(result, None);
    }

    #[test]
    fn wait_is_remaining_when_elapsed_is_less() {
        let result = compute_required_wait(Duration::from_millis(300), Duration::from_millis(500));
        assert_eq!(result, Some(Duration::from_millis(200)));
    }

    #[test]
    fn zero_elapsed_returns_full_interval_as_wait() {
        let interval = Duration::from_millis(400);
        let result = compute_required_wait(Duration::ZERO, interval);
        assert_eq!(result, Some(interval));
    }
}
