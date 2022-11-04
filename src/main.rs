use lru::LruCache;
use wtinylfu::WTinyLfuCache;

struct CacheKey{
    key: String
}

struct CacheValue{
    val: i32
}
struct ZeroCopyLRUCacheEntry{
    key: CacheKey,
    value: CacheValue
}


fn main() {
    println!("Hello, world!");
}
