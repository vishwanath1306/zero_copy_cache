pub(crate) const DEFAULT_CACHE_SIZE: usize = 10_000;
// TODO: Convert all the page sizes, and stuff to an enum with constants. 

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
pub struct Stats{
    pub access_count: i64,
}

impl Stats{
    pub fn new() -> Self{
        Stats { access_count: 0 }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Segment{
    pub segment_id: i64,
    pub segment_stats: Stats,
    pub page_size: i64,
}


impl Segment {

    pub fn new(seg_id: i64, page_size: i64) -> Self {

        Segment { segment_id: seg_id, 
                segment_stats: Stats::new(), 
                page_size: page_size 
        }
    }
    
}
pub struct HotSet{
    // TODO Experiment this with a hyperloglog to reduce space usage;
    pub current_hotset: Vec<Segment>
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CacheValue {
    pub value: Segment,
}

impl CacheValue {
    pub fn new(entry: i64) -> Self {
        CacheValue { value: Segment::new(0, 2048) }
    }
}



pub(crate) trait CacheBuilder{
    fn put(&self, key: CacheKey, value: CacheValue) -> Option<(CacheKey, CacheValue)>;
    fn get_cache_size(&self) -> usize;
    fn get(&self, key: CacheKey) -> Option<CacheValue>;
    fn get_hit_rate(&self) -> f64;
    fn resize_cache(&mut self, new_size: usize);
}