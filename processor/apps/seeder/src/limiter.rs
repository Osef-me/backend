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
        let mut guard = self.last.lock().await;
        let now = Instant::now();
        if let Some(prev) = *guard {
            let elapsed = now.saturating_duration_since(prev);
            if elapsed < interval {
                let wait = interval - elapsed;
                drop(guard);
                sleep(wait).await;
                let mut guard2 = self.last.lock().await;
                *guard2 = Some(Instant::now());
                return;
            }
        }
        *guard = Some(now);
    }
}
