use super::pagesizes;
use color_eyre::eyre::{bail, ensure, Result};
use std::{
    collections::{HashMap, HashSet, LinkedList},
    hash::Hash,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Instant,
};

pub const DEFAULT_CACHE_SIZE: usize = 10_000;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CacheType {
    OnDemandLru,
    TimestampLru,
    LinkedListLru,
    Mfu,
    NoAlg,
}

impl std::str::FromStr for CacheType {
    type Err = color_eyre::eyre::Error;
    fn from_str(s: &str) -> Result<CacheType> {
        Ok(match s {
            "ondemandlru" | "OnDemandLru" | "ONDEMANDLRU" => CacheType::OnDemandLru,
            "timestamplru" | "TimestampLru" | "TIMESTAMPLRU" => CacheType::TimestampLru,
            "linkedlistlru" | "LinkedListLru" | "LINKEDLISTLRU" => CacheType::LinkedListLru,
            "mfu" | "Mfu" | "MFU" => CacheType::Mfu,
            "noalg" | "NoAlg" | "NOALG" => CacheType::NoAlg,
            x => bail!("{} cache type unknown", x),
        })
    }
}

pub trait CacheBuilder<Slab>
where
    Slab: DatapathSlab,
{
    /// Returns cache builder that returns no more than limit segments.
    fn new(limit: usize) -> Self
    where
        Self: Sized;
    /// Returns (up to) top n segments that should be pinned.
    fn return_top_segments_to_pin(&self) -> HashSet<(Slab::SlabId, usize)>;
    /// Inserts given slab id and returns slab id to evict, if necessary.
    fn insert_and_evict(&mut self, id: (Slab::SlabId, usize)) -> Option<(Slab::SlabId, usize)>;
    /// Update access for the given segment.
    fn update_access(&mut self, id: (Slab::SlabId, usize));
    /// Resets all segments.
    fn reset(&mut self);
    /// Returns current list of pinned segments.
    fn current_pinned_segments(&self) -> &HashSet<(Slab::SlabId, usize)>;
    /// Returns number of bytes currently pinned.
    fn current_bytes_pinned(&self, segment_size: usize) -> usize {
        self.current_pinned_segments().len() * segment_size
    }
    fn set_current_pinned_list(&mut self, list: HashSet<(Slab::SlabId, usize)>);
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NoAlgCache<Slab>
where
    Slab: DatapathSlab,
{
    limit: usize,
    current_pinned_list: HashSet<(Slab::SlabId, usize)>,
}

unsafe impl<Slab> Send for NoAlgCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}
unsafe impl<Slab> Sync for NoAlgCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}

impl<Slab> CacheBuilder<Slab> for NoAlgCache<Slab>
where
    Slab: DatapathSlab,
{
    fn new(limit: usize) -> Self
    where
        Self: Sized,
    {
        NoAlgCache {
            limit,
            current_pinned_list: HashSet::default(),
        }
    }

    fn return_top_segments_to_pin(&self) -> HashSet<(Slab::SlabId, usize)> {
        return self.current_pinned_list.clone();
    }

    fn insert_and_evict(&mut self, _id: (Slab::SlabId, usize)) -> Option<(Slab::SlabId, usize)> {
        unreachable!();
    }

    fn update_access(&mut self, _id: (Slab::SlabId, usize)) {}

    fn reset(&mut self) {}

    fn current_pinned_segments(&self) -> &HashSet<(Slab::SlabId, usize)> {
        &self.current_pinned_list
    }

    fn set_current_pinned_list(&mut self, list: HashSet<(Slab::SlabId, usize)>) {
        self.current_pinned_list = list;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OnDemandLruCache<Slab>
where
    Slab: DatapathSlab,
{
    limit: usize,
    timestamps: HashMap<(Slab::SlabId, usize), Instant>,
    current_pinned_list: HashSet<(Slab::SlabId, usize)>,
}

unsafe impl<Slab> Send for OnDemandLruCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}
unsafe impl<Slab> Sync for OnDemandLruCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}

impl<Slab> CacheBuilder<Slab> for OnDemandLruCache<Slab>
where
    Slab: DatapathSlab,
{
    fn new(limit: usize) -> Self
    where
        Self: Sized,
    {
        OnDemandLruCache {
            limit,
            timestamps: HashMap::default(),
            current_pinned_list: HashSet::default(),
        }
    }

    fn return_top_segments_to_pin(&self) -> HashSet<(Slab::SlabId, usize)> {
        unimplemented!();
    }

    fn insert_and_evict(&mut self, _id: (Slab::SlabId, usize)) -> Option<(Slab::SlabId, usize)> {
        unimplemented!();
    }

    fn update_access(&mut self, id: (Slab::SlabId, usize)) {
        self.timestamps.insert(id, Instant::now());
    }

    fn reset(&mut self) {
        unimplemented!();
    }

    fn current_pinned_segments(&self) -> &HashSet<(Slab::SlabId, usize)> {
        &self.current_pinned_list
    }

    fn set_current_pinned_list(&mut self, list: HashSet<(Slab::SlabId, usize)>) {
        self.current_pinned_list = list;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TimestampLruCache<Slab>
where
    Slab: DatapathSlab,
{
    limit: usize,
    timestamps: HashMap<(Slab::SlabId, usize), Instant>,
    current_pinned_list: HashSet<(Slab::SlabId, usize)>,
}

impl<Slab> CacheBuilder<Slab> for TimestampLruCache<Slab>
where
    Slab: DatapathSlab,
{
    fn new(limit: usize) -> Self
    where
        Self: Sized,
    {
        TimestampLruCache {
            limit,
            timestamps: HashMap::default(),
            current_pinned_list: HashSet::default(),
        }
    }

    fn return_top_segments_to_pin(&self) -> HashSet<(Slab::SlabId, usize)> {
        unimplemented!();
    }

    fn insert_and_evict(&mut self, _id: (Slab::SlabId, usize)) -> Option<(Slab::SlabId, usize)> {
        unimplemented!();
    }

    fn update_access(&mut self, id: (Slab::SlabId, usize)) {
        self.timestamps.insert(id, Instant::now());
    }

    fn reset(&mut self) {
        unimplemented!();
    }

    fn current_pinned_segments(&self) -> &HashSet<(Slab::SlabId, usize)> {
        &self.current_pinned_list
    }

    fn set_current_pinned_list(&mut self, list: HashSet<(Slab::SlabId, usize)>) {
        self.current_pinned_list = list;
    }
}

unsafe impl<Slab> Send for TimestampLruCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}
unsafe impl<Slab> Sync for TimestampLruCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LinkedListLruCache<Slab>
where
    Slab: DatapathSlab,
{
    limit: usize,
    list: LinkedList<(Slab::SlabId, usize)>,
    current_pinned_list: HashSet<(Slab::SlabId, usize)>,
}

impl<Slab> CacheBuilder<Slab> for LinkedListLruCache<Slab>
where
    Slab: DatapathSlab,
{
    fn new(limit: usize) -> Self
    where
        Self: Sized,
    {
        LinkedListLruCache {
            limit,
            list: LinkedList::default(),
            current_pinned_list: HashSet::default(),
        }
    }

    fn return_top_segments_to_pin(&self) -> HashSet<(Slab::SlabId, usize)> {
        unimplemented!();
    }

    fn insert_and_evict(&mut self, _id: (Slab::SlabId, usize)) -> Option<(Slab::SlabId, usize)> {
        unimplemented!();
    }

    fn update_access(&mut self, _id: (Slab::SlabId, usize)) {
        unimplemented!();
    }

    fn reset(&mut self) {
        unimplemented!();
    }

    fn current_pinned_segments(&self) -> &HashSet<(Slab::SlabId, usize)> {
        &self.current_pinned_list
    }

    fn set_current_pinned_list(&mut self, list: HashSet<(Slab::SlabId, usize)>) {
        self.current_pinned_list = list;
    }
}
unsafe impl<Slab> Send for LinkedListLruCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}
unsafe impl<Slab> Sync for LinkedListLruCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MfuCache<Slab>
where
    Slab: DatapathSlab,
{
    limit: usize,
    access_counts: HashMap<(Slab::SlabId, usize), usize>,
    current_pinned_list: HashSet<(Slab::SlabId, usize)>,
}

impl<Slab> CacheBuilder<Slab> for MfuCache<Slab>
where
    Slab: DatapathSlab,
{
    fn new(limit: usize) -> Self
    where
        Self: Sized,
    {
        MfuCache {
            limit,
            access_counts: HashMap::default(),
            current_pinned_list: HashSet::default(),
        }
    }

    fn return_top_segments_to_pin(&self) -> HashSet<(Slab::SlabId, usize)> {
        // TODO: is there a more efficient way to do this?
        let mut counts: Vec<((Slab::SlabId, usize), usize)> =
            self.access_counts.clone().into_iter().collect();
        counts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        return HashSet::from_iter(
            counts
                .iter()
                .enumerate()
                .take_while(|(i, _val)| *i < self.limit)
                .map(|(_i, val)| val.0.clone())
                .collect::<Vec<(Slab::SlabId, usize)>>(),
        );
    }

    fn insert_and_evict(&mut self, _id: (Slab::SlabId, usize)) -> Option<(Slab::SlabId, usize)> {
        // should not be calling insert and evict for this method.
        unimplemented!();
    }

    fn update_access(&mut self, id: (Slab::SlabId, usize)) {
        let val = self.access_counts.get(&id).unwrap_or(&0);
        self.access_counts.insert(id, *val + 1);
    }

    fn reset(&mut self) {
        for (_k, val) in self.access_counts.iter_mut() {
            *val = 0;
        }
    }

    fn current_pinned_segments(&self) -> &HashSet<(Slab::SlabId, usize)> {
        &self.current_pinned_list
    }

    fn set_current_pinned_list(&mut self, list: HashSet<(Slab::SlabId, usize)>) {
        self.current_pinned_list = list;
    }
}

unsafe impl<Slab> Send for MfuCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}
unsafe impl<Slab> Sync for MfuCache<Slab> where Slab: DatapathSlab + std::fmt::Debug {}

pub trait DatapathSlab {
    type SlabId: Hash + PartialEq + Eq + Clone + Copy + std::fmt::Debug;
    type IOInfo: PartialEq + Eq + Clone + Copy;
    type PinningState: std::fmt::Debug + Send + Sync;
    type PrivateInfo: Clone + Send + Sync;

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
pub struct ZeroCopyCache<Slab, CB>
where
    Slab: DatapathSlab + std::fmt::Debug,
    CB: CacheBuilder<Slab> + std::fmt::Debug + Clone + PartialEq + Eq + Send + Sync,
{
    /// Most bytes that can be pinned at once in bytes.
    pinning_limit: usize,
    /// Size of each segment that should be maintained by ZCC in bytes.
    segment_size: usize,
    /// Whether to pin on demand,
    pin_on_demand: bool,
    /// Time to sleep between pins in pin-unpin thread.
    sleep_duration: std::time::Duration,
    /// Actual segments themselves to be pinned or unpinned, along with associated metadata.
    segments: HashMap<(Slab::SlabId, usize), Arc<Mutex<(DatapathSegment<Slab>, usize, bool)>>>,
    /// Cache module that maintains statistics on segments themselves. TODO: work on more fine
    /// grained locking.
    cache_builder: Arc<Mutex<CB>>,
    /// Cache page addresses to segment ID of size 2mb.
    page_cache_2mb: HashMap<usize, (Slab::SlabId, usize)>,
    /// Cache page addresses to segment ID for size 4kb.
    page_cache_4kb: HashMap<usize, (Slab::SlabId, usize)>,
    /// Cache page addresses to segment ID for size 1gb.
    page_cache_1gb: HashMap<usize, (Slab::SlabId, usize)>,
    /// Private (datapath-specific) info necessary for pinning/unpinning.
    priv_info: Slab::PrivateInfo,
}

impl<Slab, CB> Clone for ZeroCopyCache<Slab, CB>
where
    Slab: DatapathSlab + std::fmt::Debug,
    CB: CacheBuilder<Slab> + std::fmt::Debug + Clone + PartialEq + Eq + Send + Sync,
{
    fn clone(&self) -> Self {
        ZeroCopyCache {
            pinning_limit: self.pinning_limit.clone(),
            segment_size: self.segment_size.clone(),
            pin_on_demand: self.pin_on_demand,
            sleep_duration: self.sleep_duration.clone(),
            cache_builder: self.cache_builder.clone(),
            segments: self.segments.clone(),
            page_cache_2mb: self.page_cache_2mb.clone(),
            page_cache_4kb: self.page_cache_4kb.clone(),
            page_cache_1gb: self.page_cache_1gb.clone(),
            priv_info: self.priv_info.clone(),
        }
    }
}

impl<Slab, CB> ZeroCopyCache<Slab, CB>
where
    Slab: DatapathSlab + std::fmt::Debug,
    CB: CacheBuilder<Slab> + std::fmt::Debug + Clone + PartialEq + Eq + Send + Sync,
{
    pub fn new(
        pinning_limit: usize,
        segment_size: usize,
        pin_on_demand: bool,
        sleep_duration: std::time::Duration,
        priv_info: Slab::PrivateInfo,
    ) -> Result<Self> {
        ensure!(
            segment_size <= pinning_limit,
            "Segment size cannot be larger than pinning limit."
        );
        ensure!(
            segment_size == 0 && pinning_limit == 0 || pinning_limit % segment_size == 0,
            "Pinning limit must be a multiple of segment size"
        );
        Ok(ZeroCopyCache {
            pinning_limit,
            segment_size,
            pin_on_demand,
            sleep_duration,
            cache_builder: Arc::new(Mutex::new(CacheBuilder::new(pinning_limit / segment_size))),
            segments: HashMap::default(),
            page_cache_2mb: HashMap::default(),
            page_cache_4kb: HashMap::default(),
            page_cache_1gb: HashMap::default(),
            priv_info,
        })
    }

    pub fn pin_on_demand(&self) -> bool {
        self.pin_on_demand
    }

    /// Returns current data pinned. Assumes all segments are the same sixe.
    pub fn current_bytes_pinned(&self) -> usize {
        self.cache_builder
            .lock()
            .expect("Could not lock cache builder")
            .current_bytes_pinned(self.segment_size)
    }

    fn pin_segment(
        &mut self,
        id: &(Slab::SlabId, usize),
        priv_info: &Slab::PrivateInfo,
    ) -> Result<Slab::IOInfo> {
        let segment = self.segments.get(id);
        match segment {
            Some(extracted_segment) => {
                let mut locked_segment = extracted_segment.lock().unwrap();
                locked_segment.0.register(priv_info);
                tracing::debug!("Pinning segment: {:?}", locked_segment);
                return Ok(locked_segment.0.get_io_info());
            }
            None => {
                bail!("Trying to pin segment ID: {:?} Not found", id);
            }
        }
    }

    fn unpin_segment(&mut self, id: &(Slab::SlabId, usize)) -> Result<()> {
        let segment = self.segments.get(id);
        match segment {
            Some(extracted_segment) => loop {
                let mut locked_segment = extracted_segment.lock().unwrap();
                locked_segment.2 = true;
                if locked_segment.1 == 0 {
                    tracing::debug!("Unpinning segment: {:?}", locked_segment);
                    locked_segment.0.unregister();
                    locked_segment.2 = false;
                    break;
                }
            },
            None => {
                tracing::error!("Segment ID: {:?} Not found", id);
            }
        }
        Ok(())
    }

    pub fn update_pinned_list(&mut self, priv_info: &Slab::PrivateInfo) -> Result<()> {
        let new_pinned_list = self
            .cache_builder
            .lock()
            .expect("Could not lock cache builder")
            .return_top_segments_to_pin();

        let current_pinned_list = self
            .cache_builder
            .lock()
            .expect("Could not lock cache builder")
            .current_pinned_segments()
            .clone();

        for item in current_pinned_list.difference(&new_pinned_list) {
            self.unpin_segment(item)?;
        }

        for item in new_pinned_list.difference(&current_pinned_list) {
            let _ = self.pin_segment(item, priv_info)?;
        }

        self.cache_builder
            .lock()
            .expect("Could not lock cache builder")
            .set_current_pinned_list(new_pinned_list);
        Ok(())
    }

    pub fn pin_and_unpin_thread(&mut self, priv_info: Slab::PrivateInfo) -> Result<()> {
        if self.pin_on_demand {
            bail!("Initialized pin and unpin thread even though pin on demand configured");
        }
        loop {
            self.update_pinned_list(&priv_info)?;
            sleep(self.sleep_duration);
        }
    }

    pub fn initialize_slab(
        &mut self,
        slab: &Slab,
        register_at_start: bool,
        priv_info: Slab::PrivateInfo,
    ) -> Result<()> {
        let mempool_size = slab.get_total_num_pages() * slab.get_page_size_as_num();
        ensure!(
            mempool_size >= self.segment_size && mempool_size % self.segment_size == 0,
            format!(
                "Reg size not larger and multiple of segment size: {}",
                self.segment_size
            )
        );
        // mempool size is in bytes, segment size is in terms of half of 2MB pages
        let num_registrations = mempool_size / self.segment_size;
        tracing::debug!("Initializing slab with {} registrations", num_registrations);
        let pages_per_registration = slab.get_total_num_pages() / num_registrations;
        let reg_size = pages_per_registration * slab.get_page_size_as_num();
        let mut cur_pinned_list = self
            .cache_builder
            .lock()
            .expect("Could not lock cache builder")
            .current_pinned_segments()
            .clone();
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
                    false,
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
                        if self.current_bytes_pinned() < self.pinning_limit {
                            s.0.register(&priv_info);
                            cur_pinned_list.insert((slab.get_slab_id(), reg));
                        }
                    }
                }

                seg
            })
            .collect();

        for (i, seg) in segs.into_iter().enumerate() {
            self.segments.insert((slab.get_slab_id(), i), seg);
        }
        self.cache_builder
            .lock()
            .expect("Could not lock cache builder")
            .set_current_pinned_list(cur_pinned_list);
        Ok(())
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

    fn record_and_pin_on_demand(
        &mut self,
        segment_id: (Slab::SlabId, usize),
        priv_info: Slab::PrivateInfo,
    ) -> Result<Option<(Slab::SlabId, Slab::IOInfo)>> {
        let seg_id_option = {
            let mut cache_builder = self
                .cache_builder
                .lock()
                .expect("Could not lock cache builder");
            cache_builder.update_access(segment_id);
            cache_builder.insert_and_evict(segment_id)
        };
        if let Some(seg_id) = seg_id_option {
            self.unpin_segment(&seg_id)?;
        };

        // pin new segment
        let io_info = self.pin_segment(&segment_id, &priv_info)?;
        return Ok(Some((segment_id.0, io_info)));
    }

    pub fn record_access_and_get_io_info_if_pinned(
        &mut self,
        buf: &[u8],
        priv_info: Slab::PrivateInfo,
    ) -> Result<Option<(Slab::SlabId, Slab::IOInfo)>> {
        match self.get_segment_id(buf) {
            Some(segment_id) => {
                tracing::debug!("IO was in segment: {:?}", segment_id);
                if self.pin_on_demand {
                    return self.record_and_pin_on_demand(segment_id, priv_info);
                }
                // update access to segment
                self.cache_builder
                    .lock()
                    .expect("Failed to lock cache builder")
                    .update_access(segment_id);

                // not running ondemandlru, try to get pinning info or return None
                match self.segments.get(&segment_id) {
                    Some(segment_arc) => {
                        let mut lock = segment_arc.try_lock();
                        // if we can lock
                        if let Ok(ref mut mutex) = lock {
                            if mutex.0.is_pinned() {
                                // increment IO count
                                mutex.1 += 1;
                                // Checking for pinned segment
                                if mutex.2 {
                                    return Ok(None);
                                }
                                // return segment id and io info to caller
                                let slab_id = segment_id.0;
                                return Ok(Some((slab_id, mutex.0.get_io_info())));
                            } else {
                                // not pinned
                                return Ok(None);
                            }
                        } else {
                            // someone else has lock
                            return Ok(None);
                        }
                    }
                    None => {
                        // memory not managed by us
                        return Ok(None);
                    }
                }
            }
            None => {
                return Ok(None);
            }
        };
    }
}
