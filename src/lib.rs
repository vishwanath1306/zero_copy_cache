pub mod data_structures;
pub mod zerocopylru;

// #[cfg(test)]
// mod test {
//     use core::panic;

//     use crate::data_structures::{CacheKey, CacheValue, CacheBuilder};
//     use crate::zerocopylru::UnboundedLRUCache;
//     use rand::Rng;

//     #[test]
//     pub fn test_lru_cache_put() {

//         let curr_cache = UnboundedLRUCache::new(5);

//         let input_values = generate_key_value(7);
//         for val in input_values{
//             curr_cache.put(val.0, val.1);
//         }

//         assert_eq!(curr_cache.get_cache_size(), 5);
//         assert_ne!(curr_cache.get_cache_size(), 7);
//     }

//     #[test]
//     pub fn test_lru_retrieval(){

//         let curr_cache = UnboundedLRUCache::new(5);
        
//         let input_values = generate_key_value(7);
//         for val in input_values.clone(){
//             curr_cache.put(val.0, val.1);
//         }

//         assert_eq!(None, curr_cache.get(input_values[0].0));
//         assert_eq!(input_values[3].1, match curr_cache.get(input_values[3].0) {
//             Some(x) => {
//                 x
//             },
//             _ => panic!()
//         });
//     }

//     #[test]
//     pub fn resize_cache_bigger(){
//         let mut curr_cache = UnboundedLRUCache::new(10);
//         let input_values = generate_key_value(10);

//         for val in input_values{
//             curr_cache.put(val.0, val.1);
//         }

//         assert_eq!(10, curr_cache.get_cache_size());
//         curr_cache.resize_cache(15);

//         let input_values_2 = generate_key_value(5);

//         for val in input_values_2.clone(){
//             curr_cache.put(val.0, val.1);
//         }

//         assert_eq!(15, curr_cache.get_cache_size());
//         assert_eq!(input_values_2[3].1, match curr_cache.get(input_values_2[3].0) {
//             Some(x) => {
//                 x
//             },
//             _ => panic!()
//         });

//     }

//     #[test]
//     pub fn resize_cache_smaller(){

//         let mut curr_cache = UnboundedLRUCache::new(15);

//         let input_values = generate_key_value(10);
//         let input_values_2 = generate_key_value(5);
        
//         for val in input_values_2.clone(){
//             curr_cache.put(val.0, val.1);
//         }

//         for val in input_values.clone(){
//             curr_cache.put(val.0, val.1);
//         }

//         assert_eq!(15, curr_cache.get_cache_size());
//         curr_cache.resize_cache(10);
//         assert_eq!(10, curr_cache.get_cache_size());
//         assert_eq!(None, curr_cache.get(input_values_2[0].0));
//         assert_eq!(input_values[0].1, match curr_cache.get(input_values[0].0) {
//             Some(x) => {
//                 x
//             },
//             _ => panic!()
//         });
        
//     }

//     pub fn generate_key_value(no_of_pairs: usize) -> Vec<(CacheKey, CacheValue)>{

//         let mut value_vec: Vec<(CacheKey, CacheValue)> = Vec::new();
//         let mut rng = rand::thread_rng();
//         for _ in 0..no_of_pairs{
//             value_vec.push(
//                 (
//                     CacheKey::new(rng.gen()),
//                     CacheValue::new(rng.gen())
//                 )
//             )
//         }
//         value_vec
//     }
    
    
// }
