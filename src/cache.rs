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
    use std::collections::HashSet;

    #[test]
    fn test_cache_key_creation() {
        let key1 = CrcParamsCacheKey::new(32, 0x04C11DB7, true);
        let key2 = CrcParamsCacheKey::new(64, 0x42F0E1EBA9EA3693, false);
        
        assert_eq!(key1.width, 32);
        assert_eq!(key1.poly, 0x04C11DB7);
        assert_eq!(key1.reflected, true);
        
        assert_eq!(key2.width, 64);
        assert_eq!(key2.poly, 0x42F0E1EBA9EA3693);
        assert_eq!(key2.reflected, false);
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = CrcParamsCacheKey::new(32, 0x04C11DB7, true);
        let key2 = CrcParamsCacheKey::new(32, 0x04C11DB7, true);
        let key3 = CrcParamsCacheKey::new(32, 0x04C11DB7, false); // Different reflected
        let key4 = CrcParamsCacheKey::new(64, 0x04C11DB7, true);  // Different width
        let key5 = CrcParamsCacheKey::new(32, 0x1EDC6F41, true);  // Different poly
        
        // Test equality
        assert_eq!(key1, key2);
        assert_eq!(key1.clone(), key2.clone());
        
        // Test inequality
        assert_ne!(key1, key3);
        assert_ne!(key1, key4);
        assert_ne!(key1, key5);
        assert_ne!(key3, key4);
        assert_ne!(key3, key5);
        assert_ne!(key4, key5);
    }

    #[test]
    fn test_cache_key_hashing() {
        let key1 = CrcParamsCacheKey::new(32, 0x04C11DB7, true);
        let key2 = CrcParamsCacheKey::new(32, 0x04C11DB7, true);
        let key3 = CrcParamsCacheKey::new(32, 0x04C11DB7, false);
        
        // Create a HashSet to test that keys can be used as hash keys
        let mut set = HashSet::new();
        set.insert(key1.clone());
        set.insert(key2.clone());
        set.insert(key3.clone());
        
        // Should only have 2 unique keys (key1 and key2 are equal)
        assert_eq!(set.len(), 2);
        
        // Test that equal keys can be found in the set
        assert!(set.contains(&key1));
        assert!(set.contains(&key2));
        assert!(set.contains(&key3));
        
        // Test that a new equivalent key can be found
        let key4 = CrcParamsCacheKey::new(32, 0x04C11DB7, true);
        assert!(set.contains(&key4));
    }

    #[test]
    fn test_cache_hit_scenarios() {
        clear_cache();
        
        // First call should be a cache miss and generate keys
        let keys1 = get_or_generate_keys(32, 0x04C11DB7, true);
        
        // Second call with same parameters should be a cache hit
        let keys2 = get_or_generate_keys(32, 0x04C11DB7, true);
        
        // Keys should be identical (same array contents)
        assert_eq!(keys1, keys2);
        
        // Test multiple cache hits
        let keys3 = get_or_generate_keys(32, 0x04C11DB7, true);
        let keys4 = get_or_generate_keys(32, 0x04C11DB7, true);
        
        assert_eq!(keys1, keys3);
        assert_eq!(keys1, keys4);
        assert_eq!(keys2, keys3);
        assert_eq!(keys2, keys4);
    }

    #[test]
    fn test_cache_miss_scenarios() {
        clear_cache();
        
        // Different width - should be cache miss
        let keys_32 = get_or_generate_keys(32, 0x04C11DB7, true);
        let keys_64 = get_or_generate_keys(64, 0x04C11DB7, true);
        assert_ne!(keys_32, keys_64);
        
        // Different poly - should be cache miss
        let keys_poly1 = get_or_generate_keys(32, 0x04C11DB7, true);
        let keys_poly2 = get_or_generate_keys(32, 0x1EDC6F41, true);
        assert_ne!(keys_poly1, keys_poly2);
        
        // Different reflected - should be cache miss
        let keys_refl_true = get_or_generate_keys(32, 0x04C11DB7, true);
        let keys_refl_false = get_or_generate_keys(32, 0x04C11DB7, false);
        assert_ne!(keys_refl_true, keys_refl_false);
        
        // Verify each parameter set is cached independently
        let keys_32_again = get_or_generate_keys(32, 0x04C11DB7, true);
        let keys_64_again = get_or_generate_keys(64, 0x04C11DB7, true);
        let keys_poly1_again = get_or_generate_keys(32, 0x04C11DB7, true);
        let keys_poly2_again = get_or_generate_keys(32, 0x1EDC6F41, true);
        let keys_refl_true_again = get_or_generate_keys(32, 0x04C11DB7, true);
        let keys_refl_false_again = get_or_generate_keys(32, 0x04C11DB7, false);
        
        assert_eq!(keys_32, keys_32_again);
        assert_eq!(keys_64, keys_64_again);
        assert_eq!(keys_poly1, keys_poly1_again);
        assert_eq!(keys_poly2, keys_poly2_again);
        assert_eq!(keys_refl_true, keys_refl_true_again);
        assert_eq!(keys_refl_false, keys_refl_false_again);
    }

    #[test]
    fn test_cached_keys_identical_to_generated_keys() {
        clear_cache();
        
        // Test CRC32 parameters
        let width = 32;
        let poly = 0x04C11DB7;
        let reflected = true;
        
        // Generate keys directly (bypassing cache)
        let direct_keys = generate::keys(width, poly, reflected);
        
        // Get keys through cache (first call will be cache miss, generates and caches)
        let cached_keys_first = get_or_generate_keys(width, poly, reflected);
        
        // Get keys through cache again (should be cache hit)
        let cached_keys_second = get_or_generate_keys(width, poly, reflected);
        
        // All should be identical
        assert_eq!(direct_keys, cached_keys_first);
        assert_eq!(direct_keys, cached_keys_second);
        assert_eq!(cached_keys_first, cached_keys_second);
        
        // Test CRC64 parameters
        let width64 = 64;
        let poly64 = 0x42F0E1EBA9EA3693;
        let reflected64 = false;
        
        let direct_keys64 = generate::keys(width64, poly64, reflected64);
        let cached_keys64_first = get_or_generate_keys(width64, poly64, reflected64);
        let cached_keys64_second = get_or_generate_keys(width64, poly64, reflected64);
        
        assert_eq!(direct_keys64, cached_keys64_first);
        assert_eq!(direct_keys64, cached_keys64_second);
        assert_eq!(cached_keys64_first, cached_keys64_second);
        
        // Verify different parameters produce different keys
        assert_ne!(direct_keys, direct_keys64);
        assert_ne!(cached_keys_first, cached_keys64_first);
    }

    #[test]
    fn test_multiple_parameter_combinations() {
        clear_cache();
        
        // Test various common CRC parameter combinations
        let test_cases = [
            (32, 0x04C11DB7, true),   // CRC32
            (32, 0x04C11DB7, false),  // CRC32 non-reflected
            (32, 0x1EDC6F41, true),   // CRC32C
            (64, 0x42F0E1EBA9EA3693, true),  // CRC64 ISO
            (64, 0x42F0E1EBA9EA3693, false), // CRC64 ISO non-reflected
            (64, 0xD800000000000000, true),  // CRC64 ECMA
        ];
        
        let mut all_keys = Vec::new();
        
        // Generate keys for all test cases
        for &(width, poly, reflected) in &test_cases {
            let keys = get_or_generate_keys(width, poly, reflected);
            all_keys.push(keys);
        }
        
        // Verify all keys are different (no collisions)
        for i in 0..all_keys.len() {
            for j in (i + 1)..all_keys.len() {
                assert_ne!(all_keys[i], all_keys[j], 
                    "Keys should be different for test cases {} and {}", i, j);
            }
        }
        
        // Verify cache hits return same keys
        for (i, &(width, poly, reflected)) in test_cases.iter().enumerate() {
            let cached_keys = get_or_generate_keys(width, poly, reflected);
            assert_eq!(all_keys[i], cached_keys, 
                "Cache hit should return same keys for test case {}", i);
        }
    }

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

    #[test]
    fn test_cache_key_debug_and_clone() {
        let key = CrcParamsCacheKey::new(32, 0x04C11DB7, true);
        
        // Test Debug trait
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("CrcParamsCacheKey"));
        assert!(debug_str.contains("32"));
        assert!(debug_str.contains("0x4c11db7") || debug_str.contains("79764919"));
        assert!(debug_str.contains("true"));
        
        // Test Clone trait
        let cloned_key = key.clone();
        assert_eq!(key, cloned_key);
        assert_eq!(key.width, cloned_key.width);
        assert_eq!(key.poly, cloned_key.poly);
        assert_eq!(key.reflected, cloned_key.reflected);
    }
}