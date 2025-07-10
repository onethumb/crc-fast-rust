use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use crate::generate;

/// Global cache storage for CRC parameter keys
static CACHE: OnceLock<RwLock<HashMap<CrcParamsCacheKey, [u64; 23]>>> = OnceLock::new();

/// Cache key for storing CRC parameters that affect key generation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CrcParamsCacheKey {
    /// CRC width (32 or 64 bits)
    pub width: u8,
    /// Polynomial value used for CRC calculation
    pub poly: u64,
    /// Whether the CRC uses reflected input/output
    pub reflected: bool,
}

impl CrcParamsCacheKey {
    pub fn new(width: u8, poly: u64, reflected: bool) -> Self {
        Self {
            width,
            poly,
            reflected,
        }
    }
}

/// Initialize and return reference to the global cache
/// 
/// Uses OnceLock to ensure thread-safe lazy initialization
fn get_cache() -> &'static RwLock<HashMap<CrcParamsCacheKey, [u64; 23]>> {
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Get cached keys or generate and cache them if not present
/// 
/// This function implements a read-then-write pattern for optimal performance:
/// 1. First attempts a read lock to check for cached keys
/// 2. If cache miss, generates keys outside of any lock
/// 3. Then acquires write lock to store the generated keys
/// 
/// Falls back to direct key generation if lock poisoning occurs
pub fn get_or_generate_keys(width: u8, poly: u64, reflected: bool) -> [u64; 23] {
    let cache_key = CrcParamsCacheKey::new(width, poly, reflected);
    
    // Try cache read first - multiple threads can read simultaneously
    if let Ok(cache) = get_cache().read() {
        if let Some(keys) = cache.get(&cache_key) {
            return *keys;
        }
    }
    
    // Generate keys outside of write lock to minimize lock hold time
    let keys = generate::keys(width, poly, reflected);
    
    // Try to cache the result (best effort - if this fails, we still return valid keys)
    if let Ok(mut cache) = get_cache().write() {
        cache.insert(cache_key, keys);
    }
    
    keys
}