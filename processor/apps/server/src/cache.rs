use moka::future::Cache;
use std::sync::Arc;
use crate::types::ManiaSkill;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub hash: String,
    pub calc_type: String,
    pub centirate: u32,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub rating: f64,
    pub mania_skill: ManiaSkill,
}

/// Approx. bytes per entry (key + value). Used to bound cache to 1 GB.
const ENTRY_WEIGHT: u32 = 200;
const MAX_CAPACITY_BYTES: u64 = 1_073_741_824; // 1 GB

pub type AppCache = Arc<Cache<CacheKey, CacheEntry>>;

pub fn new_cache() -> AppCache {
    Arc::new(
        Cache::builder()
            .weigher(|_k: &CacheKey, _v: &CacheEntry| ENTRY_WEIGHT)
            .max_capacity(MAX_CAPACITY_BYTES)
            .build(),
    )
}
