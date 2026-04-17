use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub total_known: Option<u64>,
    pub sets_inserted: u64,
    pub maps_inserted: u64,
    pub ratings_inserted: u64,
    pub rox_saved: u64,
    pub pages_fetched: u64,
    pub errors: u64,
    pub skipped: u64,
    pub last_cursor: Option<String>,
    pub last_title: Option<String>,
    pub rate_per_min: u32,
    pub paused: bool,
    pub shutdown: bool,
    pub started_at: Option<Instant>,
    pub done: bool,
    pub message: Option<String>,
}

pub type Shared = Arc<RwLock<Stats>>;

pub fn new_shared(rate: u32) -> Shared {
    Arc::new(RwLock::new(Stats {
        rate_per_min: rate,
        started_at: Some(Instant::now()),
        ..Default::default()
    }))
}
