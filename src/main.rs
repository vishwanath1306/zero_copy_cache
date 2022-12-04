pub mod data_structures;
pub mod zerocopylru;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use crate::data_structures::{CacheKey, CacheValue, CacheBuilder};
    use crate::zerocopylru::UnboundedLRUCache;
    use rand::Rng;

    pub fn test_lru_cache_put() {

        let curr_cache = UnboundedLRUCache::new(5);

        let input_values = generate_key_value(6);
        for val in input_values{
            curr_cache.put(val.0, val.1);
        }

        assert_eq!(curr_cache.get_cache_size(), 5);
        assert_ne!(curr_cache.get_cache_size(), 6);
    }

    pub fn generate_key_value(no_of_pairs: usize) -> Vec<(CacheKey, CacheValue)>{

        let mut value_vec: Vec<(CacheKey, CacheValue)> = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..no_of_pairs{
            value_vec.push(
                (
                    CacheKey::new(rng.gen()),
                    CacheValue::new(rng.gen())
                )
            )
        }
        value_vec
    }
    
}
