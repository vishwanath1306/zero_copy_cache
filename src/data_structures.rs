use lru::LruCache;
use std::sync::{Mutex, Arc};

#[derive(PartialEq, Eq)]
#[derive(Hash, Clone, Copy)]
#[derive(Debug)]
pub struct CacheKey{
    key: i64
}

impl CacheKey{
    pub fn new(entry: i64) -> Self{
        CacheKey { key: entry }
    }
}

pub struct UnboundedLRUCache{
    len: usize,
    cache: Mutex<LruCache<CacheKey, i64>>,
}

impl UnboundedLRUCache {
    pub fn new(size: usize) -> UnboundedLRUCache{
        UnboundedLRUCache { 
            len: size, 
            cache: Mutex::new(LruCache::unbounded()) 
        }
    }

    pub fn insert(&self, key: CacheKey) -> Option<(CacheKey, i64)>{
        let mut unlocked_cache = self.cache.lock().unwrap();
        if self.len >= unlocked_cache.len() {
            unlocked_cache.put(key, 10);
            println!("Key is: {:?}", key.key);
            Some((key, 10))
        }
        else {
            let dropped_buffer = unlocked_cache.pop_lru();
            unlocked_cache.put(key, 10);
            dropped_buffer
        }
    }
}

impl Default for UnboundedLRUCache {
    
    fn default() -> Self { 
        UnboundedLRUCache { 
            len: 100, 
            cache: Mutex::new(LruCache::unbounded()) }
    }
}
