pub mod data_structures;
pub mod zerocopylru;


#[cfg(test)]
mod test{

    use crate::data_structures::SegmentId;
    use crate::data_structures::Segment;
    use crate::data_structures::ZeroCopyCache;
    use rand::Rng;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ExampleSegment {
        segment_id: SegmentId,
        page_size: usize,
    }

    impl ExampleSegment{
        pub fn new(segment_id: SegmentId, page_size: usize) -> Self{
            ExampleSegment { 
                segment_id: segment_id, 
                page_size: page_size
            }
        }
    }

    impl Segment for ExampleSegment {
        
        fn get_page_size(&self) -> u64 {
            self.page_size as _
        }

        fn get_segment_id(&self) -> i64 {
            self.segment_id as _
        }
    }

    #[test]
    pub fn test_zcc_segment_insert(){
        let mut zero_copy_cache = ZeroCopyCache::new();
        let new_segments = create_segments(5);
        let access_list = create_random_array(5, 50);
        for val in &access_list{
            zero_copy_cache.update_stats(&new_segments[*val]);
        }

        let value_count = access_list.clone().iter().filter(|&n| *n == 3).count() as i64;
        assert_eq!(value_count, zero_copy_cache.get_segment_access_count(new_segments[3]));
    }

    pub fn create_random_array(no_of_segments: usize, no_of_elements: usize) -> Vec<usize>{
        let mut rand_vec: Vec<usize> = Vec::new();
        let mut rand_rng = rand::thread_rng();
        for _ in 0..no_of_elements{
            rand_vec.push(rand_rng.gen_range(0..no_of_segments));
        }
        rand_vec
    }

    pub fn create_segments(no_of_segments: usize) -> Vec<ExampleSegment>{

        let mut segment_vector = Vec::new();
        for i in 0..no_of_segments{
            segment_vector.push(ExampleSegment::new((i+1).try_into().unwrap(), 4096));
        }
        segment_vector
    }
}