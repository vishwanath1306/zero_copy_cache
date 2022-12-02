#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CacheKey {
    pub key: i64,
}

impl CacheKey {
    pub fn new(entry: i64) -> Self {
        CacheKey { key: entry }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CacheValue {
    pub value: i64,
}

impl CacheValue {
    pub fn new(entry: i64) -> Self {
        CacheValue { value: entry }
    }
}

pub(crate) const DEFAULT_CACHE_SIZE: usize = 10_000;

pub(crate) trait CacheBuilder{
    fn put(&self, key: CacheKey, value: CacheValue) -> Option<(CacheKey, CacheValue)>;
    fn get_cache_size(&self) -> usize;
    fn get(&self, key: CacheKey) -> Option<(CacheKey, CacheValue)>;
    fn get_hit_rate(&self) -> f64;
}