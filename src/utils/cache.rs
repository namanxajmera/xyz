use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: u64,
    pub ttl_seconds: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl_seconds: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            data,
            timestamp,
            ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.timestamp > self.ttl_seconds
    }
}

/// Fast in-memory cache using DashMap (lock-free)
pub static MEMORY_CACHE: LazyLock<DashMap<String, CacheEntry<String>>> =
    LazyLock::new(DashMap::new);

pub fn get_cached<T: for<'de> Deserialize<'de>>(key: &str) -> Option<T> {
    if let Some(entry) = MEMORY_CACHE.get(key) {
        if !entry.is_expired() {
            if let Ok(data) = serde_json::from_str(&entry.data) {
                println!("[CACHE HIT] {}", key);
                return Some(data);
            }
        } else {
            // Remove expired entry
            MEMORY_CACHE.remove(key);
        }
    }
    println!("[CACHE MISS] {}", key);
    None
}

pub fn set_cached<T: Serialize>(key: String, data: &T, ttl_seconds: u64) {
    if let Ok(json) = serde_json::to_string(data) {
        MEMORY_CACHE.insert(key, CacheEntry::new(json, ttl_seconds));
    }
}
