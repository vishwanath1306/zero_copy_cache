use std::default;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};
use std::thread::sleep;
use std::{collections::HashMap, hash::Hash, collections::HashSet};

use crate::pagesizes;

pub const DEFAULT_CACHE_SIZE: usize = 10_000;
// TODO: Convert all the page sizes, and stuff to an enum with constants.

pub type SegmentStatMap<ID> = HashMap<ID, Stats>;

// =========================================================

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Stats {
    pub access_count: u64,
    pub last_access_time: SystemTime,
    pub last_pinned_time: SystemTime,
    pub last_unpinned_time: SystemTime,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            access_count: 1,
            last_access_time: SystemTime::now(),
            last_pinned_time: SystemTime::UNIX_EPOCH,
            last_unpinned_time: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn update_stats(&mut self) {
        self.last_access_time = SystemTime::now();
        self.access_count += 1;
    }

    pub fn update_access_time(&mut self) {
        self.last_access_time = SystemTime::now();
    }

    pub fn update_pinned_time(&mut self) {
        self.last_pinned_time = SystemTime::now();
    }

    pub fn update_unpinned_time(&mut self) {
        self.last_unpinned_time = SystemTime::now();
    }

    pub fn increment_access_count(&mut self) {
        self.access_count += 1;
    }

    pub fn get_access_count(&self) -> i64 {
        self.access_count
    }

    pub fn get_last_access_time(&self) -> SystemTime{
        self.last_access_time
    }

    pub fn get_last_pinned_time(&self) -> SystemTime{
        self.last_pinned_time
    }

    pub fn get_last_unpinned_time(&self) -> SystemTime{
        self.last_unpinned_time
    }

}

// =========================================================
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Metrics{

    pub total_count: u64, 
    pub hit_count: u64, 
    pub miss_count: u64

}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            total_count: 0, 
            hit_count: 0,
            miss_count: 0
        }
    }

    pub fn update_total_count(&mut self) {
        self.total_count += 1;
    }

    pub fn update_hit_count(&mut self) {
        self.hit_count += 1;
    }

    pub fn update_miss_count(&mut self) {
        self.miss_count += 1;
    }

    pub fn get_total_count(&mut self) -> u64 {
        self.total_count
    }

    pub fn get_hit_count(&mut self) -> u64 {
        self.hit_count
    }

    pub fn get_miss_count(&mut self) -> u64 {
        self.miss_count
    }

}

// =========================================================

pub trait DatapathSlab {
    type SlabId: Hash + PartialEq + Eq + Clone + Copy + std::fmt::Debug;
    type IOInfo: PartialEq + Eq + Clone + Copy;
    type PinningState: std::fmt::Debug + Send + Sync;
    type PrivateInfo;

    fn default_pinning_state(&self) -> Self::PinningState;

    fn get_slab_id(&self) -> Self::SlabId;

    fn is_pinned(pinning_state: &Self::PinningState) -> bool;

    fn pin_segment(
        pinning_state: &mut Self::PinningState,
        private_info: &Self::PrivateInfo,
        start_address: *mut ::std::os::raw::c_void,
        len: usize,
    );

    fn unpin_segment(pinning_state: &mut Self::PinningState);

    fn get_io_info(pinning_state: &Self::PinningState) -> Self::IOInfo;

    fn get_total_num_pages(&self) -> usize;

    fn get_start_address(&self) -> *mut ::std::os::raw::c_void;

    fn get_page_size(&self) -> pagesizes::PageSize;

    fn get_page_size_as_num(&self) -> usize {
        match self.get_page_size() {
            pagesizes::PageSize::PG4KB => pagesizes::PGSIZE_4KB,
            pagesizes::PageSize::PG2MB => pagesizes::PGSIZE_2MB,
            pagesizes::PageSize::PG1GB => pagesizes::PGSIZE_1GB,
        }
    }
}

#[derive(Debug)]
pub struct DatapathSegment<Slab>
where
    Slab: DatapathSlab + std::fmt::Debug,
{
    start_address: *mut ::std::os::raw::c_void,
    num_pages: usize,
    page_size: pagesizes::PageSize,
    pinning_state: Slab::PinningState,
    id: (Slab::SlabId, usize),
}

unsafe impl<Slab> Send for DatapathSegment<Slab> where Slab: DatapathSlab + std::fmt::Debug {}
unsafe impl<Slab> Sync for DatapathSegment<Slab> where Slab: DatapathSlab + std::fmt::Debug {}

impl<Slab> DatapathSegment<Slab>
where
    Slab: DatapathSlab + std::fmt::Debug,
{
    pub fn new(
        start_address: *mut ::std::os::raw::c_void,
        num_pages: usize,
        page_size: pagesizes::PageSize,
        segment_id: usize,
        slab: &Slab,
    ) -> Self {
        DatapathSegment {
            start_address,
            num_pages,
            page_size,
            pinning_state: slab.default_pinning_state(),
            id: (slab.get_slab_id(), segment_id),
        }
    }

    pub fn get_id(&self) -> (Slab::SlabId, usize) {
        self.id
    }

    pub fn get_page_size_as_num(&self) -> usize {
        match self.page_size {
            pagesizes::PageSize::PG4KB => pagesizes::PGSIZE_4KB,
            pagesizes::PageSize::PG2MB => pagesizes::PGSIZE_2MB,
            pagesizes::PageSize::PG1GB => pagesizes::PGSIZE_1GB,
        }
    }

    pub fn register(&mut self, priv_info: &Slab::PrivateInfo) {
        let reglen = self.num_pages * self.get_page_size_as_num();
        Slab::pin_segment(
            &mut self.pinning_state,
            priv_info,
            self.start_address,
            reglen,
        )
    }

    pub fn unregister(&mut self) {
        Slab::unpin_segment(&mut self.pinning_state);
    }

    pub fn is_pinned(&self) -> bool {
        Slab::is_pinned(&self.pinning_state)
    }

    pub fn get_page_size(&self) -> pagesizes::PageSize {
        self.page_size.clone()
    }

    pub fn get_start_address(&self) -> *mut ::std::os::raw::c_void {
        self.start_address
    }

    pub fn get_io_info(&self) -> Slab::IOInfo {
        Slab::get_io_info(&self.pinning_state)
    }

    fn get_num_pages(&self) -> usize {
        self.num_pages
    }

    fn get_1gb_pages(&self) -> Vec<usize> {
        match self.page_size {
            pagesizes::PageSize::PG1GB => (0..self.get_num_pages())
                .map(|i| self.get_start_address() as usize + self.get_page_size_as_num() * i)
                .collect::<Vec<usize>>(),
            _ => {
                vec![]
            }
        }
    }

    fn get_2mb_pages(&self) -> Vec<usize> {
        match self.page_size {
            pagesizes::PageSize::PG2MB => (0..self.get_num_pages())
                .map(|i| self.get_start_address() as usize + self.get_page_size_as_num() * i)
                .collect::<Vec<usize>>(),
            _ => {
                vec![]
            }
        }
    }

    fn get_4kb_pages(&self) -> Vec<usize> {
        match self.page_size {
            pagesizes::PageSize::PG4KB => (0..self.get_num_pages())
                .map(|i| self.get_start_address() as usize + self.get_page_size_as_num() * i)
                .collect::<Vec<usize>>(),
            _ => {
                vec![]
            }
        }
    }
}

#[derive(Debug)]
pub struct ZeroCopyCache<Slab>
where
    Slab: DatapathSlab + std::fmt::Debug, // C: CacheBuilder<S>
{
    /// Stats maintained for each segment.
    // TODO: Work on locking this 
    pub segment_stats: Arc<Mutex<SegmentStatMap<(Slab::SlabId, usize)>>>,
    /// Current hotset.
    pub current_pinned_list: HashSet<(Slab::SlabId, usize)>,
    /// Actual segments themselves to be pinned or unpinned, along with associated metadata.
    // TODO: Convert the segment part into a struct
    segments: HashMap<(Slab::SlabId, usize), Arc<Mutex<(DatapathSegment<Slab>, usize, bool)>>>,
    /// Cache page addresses to segment ID of size 2mb.
    page_cache_2mb: HashMap<usize, (Slab::SlabId, usize)>,
    /// Cache page addresses to segment ID for size 4kb.
    page_cache_4kb: HashMap<usize, (Slab::SlabId, usize)>,
    /// Cache page addresses to segment ID for size 1gb.
    page_cache_1gb: HashMap<usize, (Slab::SlabId, usize)>,
    // pub cache_builder: C
}

impl<Slab> Clone for ZeroCopyCache<Slab>
where
    Slab: DatapathSlab + std::fmt::Debug,
{
    fn clone(&self) -> Self {
        ZeroCopyCache {
            segment_stats: self.segment_stats.clone(),
            current_pinned_list: self.current_pinned_list.clone(),
            segments: self.segments.clone(),
            page_cache_2mb: self.page_cache_2mb.clone(),
            page_cache_4kb: self.page_cache_4kb.clone(),
            page_cache_1gb: self.page_cache_1gb.clone(),
        }
    }
}

impl<Slab> ZeroCopyCache<Slab>
where
    Slab: DatapathSlab + std::fmt::Debug,
{
    pub fn new() -> Self {
        ZeroCopyCache {
            // segment_stats: Arc::new(SegmentStatMap::<(Slab::SlabId, usize)>::default()),
            segment_stats: Arc::new(Mutex::new(SegmentStatMap::<(Slab::SlabId, usize)>::default())),
            current_pinned_list: HashSet::default(),
            segments: HashMap::default(),
            page_cache_2mb: HashMap::default(),
            page_cache_4kb: HashMap::default(),
            page_cache_1gb: HashMap::default(),
        }
    }

    pub fn pin_and_unpin_thread(&mut self, priv_info: Slab::PrivateInfo) {
        loop {
            let new_pinned_list = self.return_all_segments_sized();
            tracing::debug!("The current hotset is: {:?}", new_pinned_list);
            // tracing::debug!("The segment stats is: {:?}", self.segment_stats);
            for item in self.current_pinned_list.difference(&new_pinned_list){
                // UNPINNING THE ITEMS
                let segment = self.segments.get(item);
                match segment{
                    Some(extracted_segment) => {
                        loop {
                            let mut locked_segment = extracted_segment.lock().unwrap();
                            locked_segment.2 = true;
                            if locked_segment.1 == 0 {
                                tracing::debug!("Unpinning segment: {:?}", locked_segment);
                                locked_segment.0.unregister();
                                locked_segment.2 = false; 
                                break;
                            }
                        }
                    }
                    None => {
                        tracing::error!("Segment ID: {:?} Not found", item.0);
                    }
                }
            }

            for item in new_pinned_list.difference(&self.current_pinned_list){
                let segment = self.segments.get(item);
                match segment{
                    Some(extracted_segment) => {
                        let mut locked_segment = extracted_segment.lock().unwrap();
                        locked_segment.0.register(&priv_info);
                        tracing::debug!("Pinning segment: {:?}", locked_segment);
                    },
                    None => {
                        tracing::error!("Segment ID: {:?} Not found", item.0);
                    }
                }
            }

            self.current_pinned_list = new_pinned_list;
            sleep(Duration::new(1, 0));
        }
    }

    pub fn initialize_slab(
        &mut self,
        slab: &Slab,
        num_registrations: usize,
        register_at_start: bool,
        priv_info: Slab::PrivateInfo,
    ) {
        tracing::debug!("Initializing slab with {} registrations", num_registrations);
        let pages_per_registration = slab.get_total_num_pages() / num_registrations;
        let reg_size = pages_per_registration * slab.get_page_size_as_num();
        let segs: Vec<Arc<Mutex<(DatapathSegment<Slab>, usize, bool)>>> = (0..num_registrations)
            .map(|reg| {
                let start_address = slab.get_start_address() as usize + reg_size * reg;
                let seg = Arc::new(Mutex::new((
                    DatapathSegment::new(
                        start_address as *mut ::std::os::raw::c_void,
                        pages_per_registration,
                        slab.get_page_size(),
                        reg,
                        slab,
                    ),
                    0usize,
                    false
                )));
                if let Ok(ref mut s) = seg.lock() {
                    for page in s.0.get_4kb_pages() {
                        self.page_cache_4kb.insert(page, (slab.get_slab_id(), reg));
                    }
                    for page in s.0.get_2mb_pages() {
                        self.page_cache_2mb.insert(page, (slab.get_slab_id(), reg));
                    }
                    for page in s.0.get_1gb_pages() {
                        self.page_cache_1gb.insert(page, (slab.get_slab_id(), reg));
                    }
                    // if register at start, register slab
                    if register_at_start {
                        s.0.register(&priv_info);
                    }
                }

                seg
            })
            .collect();

        for (i, seg) in segs.into_iter().enumerate() {
            self.segments.insert((slab.get_slab_id(), i), seg);
        }
    }

    /// Get segment ID for raw address.
    pub fn get_segment_id(&self, buf: &[u8]) -> Option<(Slab::SlabId, usize)> {
        match self
            .page_cache_2mb
            .get(&pagesizes::closest_2mb_page(buf.as_ptr()))
        {
            Some(m) => {
                return Some(*m);
            }
            None => {}
        }
        match self
            .page_cache_4kb
            .get(&pagesizes::closest_4k_page(buf.as_ptr()))
        {
            Some(m) => {
                return Some(*m);
            }
            None => {}
        }
        match self
            .page_cache_1gb
            .get(&pagesizes::closest_1g_page(buf.as_ptr()))
        {
            Some(m) => {
                return Some(*m);
            }
            None => {}
        }
        return None;
    }

    pub fn record_io_completion(&mut self, addr: &[u8]) {
        if let Some(segment_id) = self.get_segment_id(addr) {
            if let Some(segment_arc) = self.segments.get(&segment_id) {
                segment_arc.lock().unwrap().1 -= 1;
            }
        }
    }

    pub fn record_access_and_get_io_info_if_pinned(
        &mut self,
        buf: &[u8],
    ) -> Option<(Slab::SlabId, Slab::IOInfo)> {
        match self.get_segment_id(buf) {
            Some(segment_id) => {
                tracing::debug!("IO was in segment: {:?}", segment_id);
                // update access to segment
                self.update_stats(segment_id);

                // try to get lock around segment and count to update
                match self.segments.get(&segment_id) {
                    Some(segment_arc) => {
                        let mut lock = segment_arc.try_lock();
                        // if we can lock
                        if let Ok(ref mut mutex) = lock {
                            if mutex.0.is_pinned() {
                                // increment IO count
                                mutex.1 += 1;
                                // Checking for pinned segment
                                if mutex.2{
                                    return None;
                                }
                                // return segment id and io info to caller
                                let slab_id = segment_id.0;
                                return Some((slab_id, mutex.0.get_io_info()));
                            } else {
                                return None;
                            }
                        } else {
                            // someone else has lock
                            return None;
                        }
                    }

                    None => {
                        return None;
                    }
                }
            }
            None => {
                return None;
            }
        };
    }

    pub fn update_stats(&mut self, segment_id: (Slab::SlabId, usize)) {
        let mut unlocked_segment_stats = self.segment_stats.lock().unwrap();
        if unlocked_segment_stats.contains_key(&segment_id){
            unlocked_segment_stats.get_mut(&segment_id)
            .unwrap()
            .update_stats();
        }else {
            unlocked_segment_stats.insert(segment_id, Stats::new());
        }
    }

    pub fn get_segment_access_count(&self, segment_id: (Slab::SlabId, usize)) -> Option<i64> {
        let cloned_segment = self.segment_stats.lock().unwrap();
        match cloned_segment.get(&segment_id) {
            Some(s) => Some(s.get_access_count()),
            None => None
        }
    }

    /// Currently ineffecient strategy of sorting through the vector and getting the top segments. 
    /// Need better strategies to performance these actions.
    pub fn calculate_hotset_v0(&mut self) -> HashSet<(Slab::SlabId, usize)>{

        let mut sorting_vec: Vec<((Slab::SlabId, usize), i64)> = Vec::new();
        let mut pinned_list = HashSet::new();
        let cloned_segment_list = self.segment_stats.lock().unwrap();
        let curr_val = cloned_segment_list.clone();
        std::mem::drop(cloned_segment_list);
        for (k, v) in curr_val.clone().into_iter() {
            sorting_vec.push((k, v.access_count));
        }
        sorting_vec.sort_by(|a, b| a.1.cmp(&b.1));
        for (seg_id, _) in sorting_vec {
            pinned_list.insert(seg_id);
        }
        
        pinned_list
    }

     pub fn return_all_segments_sized(&mut self) -> HashSet<(Slab::SlabId, usize)>{
        tracing::debug!("Going into the return all segments");
        let mut pinned_list = HashSet::new();
        let cloned_segment_list = self.segment_stats.lock().unwrap();
        let current_values = cloned_segment_list.clone();
        std::mem::drop(cloned_segment_list);
        for (seg_id, _) in current_values{
            pinned_list.insert(seg_id);
        }
        pinned_list
     }
}
