use crate::data_structures::{CacheKey, CacheValue};
use wtinylfu::WTinyLfuCache;
use std::sync::Mutex;
use crate::data_structures::CacheBuilder;

pub struct UnboundedwTinyLfuCache {
    len: usize,
    cache: Mutex<WTinyLfuCache<CacheKey, CacheValue>>,
    hit_count: u64,
    total_count: u64,
    miss_count: u64,
}


impl UnboundedwTinyLfuCache {
    pub fn new(size: usize, sample_size: usize) -> UnboundedwTinyLfuCache {
        UnboundedwTinyLfuCache {
            len: size,
            cache: Mutex::new(WTinyLfuCache::new(size, sample_size)),
            hit_count: 0,
            miss_count: 0,
            total_count: 0,
        }
    }

}

impl CacheBuilder for UnboundedwTinyLfuCache{

    fn put(&self, key: CacheKey, value: CacheValue) -> Option<(CacheKey, CacheValue)> {
        let mut unlocked_cache = self.cache.lock().unwrap();
        
        if self.len > unlocked_cache.len() {
            unlocked_cache.put(key, value);
            Some((key, value))
        } else {
            let dropped_buffer = unlocked_cache.pop_entry();
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

    fn resize_cache(&mut self, new_size: usize) {
        if new_size >= self.len{
            self.len = new_size;
        }else{
            let mut unlocked_cache = self.cache.lock().unwrap();
            let difference = self.len - new_size;
            for _ in 0..difference{
                unlocked_cache.pop_lru();
            }
            self.len = new_size;
        }
    }

}


