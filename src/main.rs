pub mod data_structures;

use data_structures::UnboundedLRUCache;

use crate::data_structures::CacheKey;
fn main() {

    let curr_cache = UnboundedLRUCache::default();
    let buff = curr_cache.insert(CacheKey::new(100));
    match buff {
        Some(x) => {
            println!("The cachekey is: {:?}", x.0)
        }
        
        None => {
            println!("Returned Nothing");
        }
        
    }
    println!("Hello, world!");
}
