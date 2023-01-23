use std::{collections::HashMap, hash::Hash};

pub const DEFAULT_CACHE_SIZE: usize = 10_000;
// TODO: Convert all the page sizes, and stuff to an enum with constants. 

pub type SegmentStatMap< S >  = HashMap<S, Stats>;
pub type SegmentId = i64;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Stats{
    pub access_count: i64,
}

impl Stats{
    pub fn new() -> Self{
        Stats { access_count: 0 }
    }
}

pub trait Segment {

    fn get_segment_id(&self) -> i64;
    fn get_page_size(&self) -> u64;

}
 
pub struct ZeroCopyCache<S, C> where
S: Hash + PartialEq + Eq + Clone + Segment,
C: CacheBuilder<S>
 {
    pub segment_stats: SegmentStatMap<S>, 
    pub current_pinned_list: Vec<SegmentId>,
    pub cache_builder: C
}

pub trait CacheBuilder<S: Hash + PartialEq + Clone + Eq + Segment>{

    fn insert(&mut self, segment: &S);
    fn get_curr_pinned_list(&self) -> Vec<SegmentId>;
    fn get_cache_size(&self) -> usize;
    fn resize_cache(&mut self, new_size: usize);

}