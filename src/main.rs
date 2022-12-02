pub mod data_structures;
pub mod zerocopylru;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use crate::data_structures::{CacheKey, CacheValue, CacheBuilder};
    use crate::zerocopylru::UnboundedLRUCache;

    pub fn test_lru_cache_() {
        let curr_cache = UnboundedLRUCache::new(5);

        let buff = curr_cache.put(CacheKey::new(100), CacheValue::new(200));

        match buff {
            Some(x) => {
                println!("The cachekey is: {:?}", x.0)
            }

            None => {
                println!("Returned Nothing");
            }
        }
    }
}
