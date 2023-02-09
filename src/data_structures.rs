use std::time::SystemTime;
use std::{collections::HashMap, hash::Hash};

pub const DEFAULT_CACHE_SIZE: usize = 10_000;
// TODO: Convert all the page sizes, and stuff to an enum with constants.

pub type SegmentStatMap<S> = HashMap<S, Stats>;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Stats {
    pub access_count: i64,
    pub last_access_time: SystemTime,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            access_count: 1,
            last_access_time: SystemTime::now(),
        }
    }

    pub fn update_stats(&mut self) {
        self.last_access_time = SystemTime::now();
        self.access_count += 1;
    }

    pub fn update_access_time(&mut self) {
        self.last_access_time = SystemTime::now();
    }

    pub fn increment_access_count(&mut self) {
        self.access_count += 1;
    }

    pub fn get_access_count(&self) -> i64 {
        self.access_count
    }
}

pub trait Segment {
    type SegmentId: Hash + PartialEq + Eq + Clone + Copy;
    fn get_segment_id(&self) -> Self::SegmentId;
    fn get_page_size(&self) -> u64;
}

#[derive(Debug, Eq, PartialEq)]
pub struct ZeroCopyCache<S>
where
    S: Hash + PartialEq + Eq + Clone + Segment + Default,
    // C: CacheBuilder<S>
{
    pub segment_stats: SegmentStatMap<S>,
    pub current_pinned_list: Vec<S::SegmentId>,
    // pub cache_builder: C
}

impl<S> ZeroCopyCache<S>
where
    S: Hash + PartialEq + Eq + Clone + Segment + Default,
    // C: CacheBuilder<S>
{
    pub fn new() -> Self {
        ZeroCopyCache {
            segment_stats: SegmentStatMap::default(),
            current_pinned_list: Vec::new(),
            // cache_builder: cache_builder
        }
    }

    pub fn update_stats(&mut self, segment: &S) {
        if self.segment_stats.contains_key(segment) {
            self.segment_stats.get_mut(segment).unwrap().update_stats();
        } else {
            // Stats constructor should automatically increment to 1
            self.segment_stats.insert(segment.clone(), Stats::new());
        }
    }

    pub fn get_segment_access_count(&self, segment: S) -> Option<i64> {
        match self.segment_stats.get(&segment) {
            Some(seg) => Some(seg.get_access_count()),
            None => None,
        }
    }

    pub fn calculate_hotset(&mut self) {
        let mut sorting_vec: Vec<(S::SegmentId, i64)> = Vec::new();
        let mut pinned_list = Vec::new();
        for (k, v) in self.segment_stats.clone().into_iter() {
            sorting_vec.push((k.get_segment_id(), v.access_count));
        }

        sorting_vec.sort_by(|a, b| a.1.cmp(&b.1));
        for (seg_id, _) in sorting_vec {
            pinned_list.push(seg_id);
        }
        self.current_pinned_list = pinned_list;
    }
}
