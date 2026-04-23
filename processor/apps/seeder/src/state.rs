use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::logfile;

const MAX_LOG_ENTRIES: usize = 100;

#[derive(Clone, Debug)]
pub enum LogType {
    Network,
    Db,
    Calc,
    Info,
    Retry,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub log_type: LogType,
    pub message: String,
}

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
    pub logs: VecDeque<LogEntry>,
    pub retry_attempt: u32,
    pub rox_bytes: u64,
    pub db_bytes: u64,
    pub current_key: Option<u32>,
}

impl Stats {
    pub fn log(&mut self, log_type: LogType, message: String) {
        logfile::log_state_entry(&log_type, &message);
        if self.logs.len() >= MAX_LOG_ENTRIES {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            timestamp: Instant::now(),
            log_type,
            message,
        });
    }
}

pub type Shared = Arc<RwLock<Stats>>;

pub fn new_shared(rate: u32) -> Shared {
    Arc::new(RwLock::new(Stats {
        rate_per_min: rate,
        started_at: Some(Instant::now()),
        logs: VecDeque::new(),
        ..Default::default()
    }))
}
