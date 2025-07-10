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
/// All cache operations are best-effort with graceful degradation - if any cache
/// operation fails, the function falls back to direct key generation
pub fn get_or_generate_keys(width: u8, poly: u64, reflected: bool) -> [u64; 23] {
    let cache_key = CrcParamsCacheKey::new(width, poly, reflected);
    
    // Try cache read first - multiple threads can read simultaneously
    // If lock is poisoned or read fails, continue to key generation
    if let Ok(cache) = get_cache().read() {
        if let Some(keys) = cache.get(&cache_key) {
            return *keys;
        }
    }
    
    // Generate keys outside of write lock to minimize lock hold time
    let keys = generate::keys(width, poly, reflected);
    
    // Try to cache the result (best effort - if this fails, we still return valid keys)
    // Lock poisoning or write failure doesn't affect functionality
    let _ = get_cache().write().map(|mut cache| {
        cache.insert(cache_key, keys)
    });
    
    keys
}

/// Clear all cached CRC parameter keys
/// 
/// This function is primarily intended for testing and memory management.
/// It performs a best-effort clear operation - if the cache lock is poisoned
/// or unavailable, the operation silently fails without affecting program execution.
/// 
/// # Thread Safety
/// This function is thread-safe and can be called concurrently with other cache operations.
/// However, clearing the cache while other threads are actively using it may reduce
/// performance temporarily as keys will need to be regenerated.
pub fn clear_cache() {
    // Best-effort cache clear - if lock is poisoned or unavailable, silently continue
    // This ensures the function never panics or blocks program execution
    let _ = get_cache().write().map(|mut cache| {
        cache.clear()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_management_utilities() {
        // Clear cache to start with clean state
        clear_cache();
        
        // Generate and cache some keys
        let keys1 = get_or_generate_keys(32, 0x04C11DB7, true);
        let keys2 = get_or_generate_keys(64, 0x42F0E1EBA9EA3693, false);
        
        // Verify cache hits return same keys
        let cached_keys1 = get_or_generate_keys(32, 0x04C11DB7, true);
        let cached_keys2 = get_or_generate_keys(64, 0x42F0E1EBA9EA3693, false);
        
        assert_eq!(keys1, cached_keys1);
        assert_eq!(keys2, cached_keys2);
        
        // Clear cache
        clear_cache();
        
        // Verify cache was cleared by checking that new calls still work
        // (we can't directly verify cache is empty, but we can verify functionality)
        let new_keys1 = get_or_generate_keys(32, 0x04C11DB7, true);
        assert_eq!(keys1, new_keys1); // Should be same values, but freshly generated
    }
    
    #[test]
    fn test_cache_error_handling() {
        // Test that cache operations don't panic even if called multiple times
        clear_cache();
        clear_cache(); // Should not panic on empty cache
        
        // Test that get_or_generate_keys works even after multiple clears
        let keys = get_or_generate_keys(32, 0x04C11DB7, true);
        clear_cache();
        let keys2 = get_or_generate_keys(32, 0x04C11DB7, true);
        
        // Keys should be identical (same parameters produce same keys)
        assert_eq!(keys, keys2);
    }
}