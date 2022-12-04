use crate::data_structures::{CacheKey, CacheValue};
use lru::LruCache;
use std::sync::Mutex;
use crate::data_structures::CacheBuilder;

pub struct UnboundedLRUCache {
    len: usize,
    cache: Mutex<LruCache<CacheKey, CacheValue>>,
    hit_count: u64,
    total_count: u64,
    miss_count: u64,
}

impl Default for UnboundedLRUCache {
    fn default() -> Self {
        UnboundedLRUCache {
            len: crate::data_structures::DEFAULT_CACHE_SIZE,
            cache: Mutex::new(LruCache::unbounded()),
            hit_count: 0,
            miss_count: 0,
            total_count: 0,
        }
    }
}

impl UnboundedLRUCache {
    pub fn new(size: usize) -> UnboundedLRUCache {
        UnboundedLRUCache {
            len: size,
            cache: Mutex::new(LruCache::unbounded()),
            hit_count: 0,
            miss_count: 0,
            total_count: 0,
        }
    }

}

impl CacheBuilder for UnboundedLRUCache{

    fn put(&self, key: CacheKey, value: CacheValue) -> Option<(CacheKey, CacheValue)> {
        let mut unlocked_cache = self.cache.lock().unwrap();
        
        if self.len > unlocked_cache.len() {
            unlocked_cache.put(key, value);
            Some((key, value))
        } else {
            let dropped_buffer = unlocked_cache.pop_lru();
            unlocked_cache.put(key, value);
            dropped_buffer
        }
    }

    fn get(&self, key: CacheKey) -> Option<CacheValue> {
        let mut unlocked_cache = self.cache.lock().unwrap();
        let return_value = unlocked_cache.get(&key);
        return_value.copied()
    }

    fn get_cache_size(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    fn get_hit_rate(&self) -> f64 {
        todo!()
    }

}


