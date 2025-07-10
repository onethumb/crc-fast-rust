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

    // Thread safety tests
    #[test]
    fn test_concurrent_cache_reads() {
        use std::sync::{Arc, Barrier};
        use std::thread;
        
        clear_cache();
        
        // Pre-populate cache with a known value
        let expected_keys = get_or_generate_keys(32, 0x04C11DB7, true);
        
        let num_threads = 8;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = Vec::new();
        
        // Spawn multiple threads that all read the same cached value simultaneously
        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();
                
                // All threads read from cache simultaneously
                let keys = get_or_generate_keys(32, 0x04C11DB7, true);
                (i, keys)
            });
            handles.push(handle);
        }
        
        // Collect results from all threads
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().expect("Thread should not panic"));
        }
        
        // Verify all threads got the same cached keys
        assert_eq!(results.len(), num_threads);
        for (thread_id, keys) in results {
            assert_eq!(keys, expected_keys, 
                "Thread {} should get same cached keys", thread_id);
        }
    }

    #[test]
    fn test_concurrent_cache_writes() {
        use std::sync::{Arc, Barrier};
        use std::thread;
        
        clear_cache();
        
        let num_threads = 6;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = Vec::new();
        
        // Test parameters for different cache entries
        let test_params = [
            (32, 0x04C11DB7, true),
            (32, 0x04C11DB7, false), 
            (32, 0x1EDC6F41, true),
            (64, 0x42F0E1EBA9EA3693, true),
            (64, 0x42F0E1EBA9EA3693, false),
            (64, 0xD800000000000000, true),
        ];
        
        // Spawn threads that write different cache entries simultaneously
        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let (width, poly, reflected) = test_params[i];
            
            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();
                
                // Each thread generates and caches different parameters
                let keys = get_or_generate_keys(width, poly, reflected);
                (i, width, poly, reflected, keys)
            });
            handles.push(handle);
        }
        
        // Collect results from all threads
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().expect("Thread should not panic"));
        }
        
        // Verify all threads completed successfully and got correct keys
        assert_eq!(results.len(), num_threads);
        
        // Verify each thread's keys match what we'd expect from direct generation
        for (thread_id, width, poly, reflected, keys) in results {
            let expected_keys = generate::keys(width, poly, reflected);
            assert_eq!(keys, expected_keys, 
                "Thread {} should generate correct keys for params ({}, {:#x}, {})", 
                thread_id, width, poly, reflected);
            
            // Verify the keys are now cached by reading them again
            let cached_keys = get_or_generate_keys(width, poly, reflected);
            assert_eq!(keys, cached_keys, 
                "Thread {} keys should be cached", thread_id);
        }
    }

    #[test]
    fn test_read_write_contention() {
        use std::sync::{Arc, Barrier};
        use std::thread;
        use std::time::Duration;
        
        clear_cache();
        
        // Pre-populate cache with some values
        let _keys1 = get_or_generate_keys(32, 0x04C11DB7, true);
        let _keys2 = get_or_generate_keys(64, 0x42F0E1EBA9EA3693, false);
        
        let num_readers = 6;
        let num_writers = 3;
        let total_threads = num_readers + num_writers;
        let barrier = Arc::new(Barrier::new(total_threads));
        let mut handles = Vec::new();
        
        // Spawn reader threads that continuously read cached values
        for i in 0..num_readers {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                
                let mut read_count = 0;
                let start = std::time::Instant::now();
                
                // Read for a short duration
                while start.elapsed() < Duration::from_millis(50) {
                    let keys1 = get_or_generate_keys(32, 0x04C11DB7, true);
                    let keys2 = get_or_generate_keys(64, 0x42F0E1EBA9EA3693, false);
                    
                    // Verify we get consistent results
                    assert_eq!(keys1.len(), 23);
                    assert_eq!(keys2.len(), 23);
                    read_count += 1;
                }
                
                (format!("reader_{}", i), read_count)
            });
            handles.push(handle);
        }
        
        // Spawn writer threads that add new cache entries
        for i in 0..num_writers {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                
                let mut write_count = 0;
                let start = std::time::Instant::now();
                
                // Write new entries for a short duration
                while start.elapsed() < Duration::from_millis(50) {
                    // Use different parameters to create new cache entries
                    let poly = 0x1EDC6F41 + (i as u64 * 0x1000) + (write_count as u64);
                    let keys = get_or_generate_keys(32, poly, true);
                    
                    assert_eq!(keys.len(), 23);
                    write_count += 1;
                }
                
                (format!("writer_{}", i), write_count)
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().expect("Thread should not panic"));
        }
        
        // Verify all threads completed successfully
        assert_eq!(results.len(), total_threads);
        
        // Verify readers and writers both made progress
        let reader_results: Vec<_> = results.iter()
            .filter(|(name, _)| name.starts_with("reader_"))
            .collect();
        let writer_results: Vec<_> = results.iter()
            .filter(|(name, _)| name.starts_with("writer_"))
            .collect();
        
        assert_eq!(reader_results.len(), num_readers);
        assert_eq!(writer_results.len(), num_writers);
        
        // All threads should have made some progress
        for (name, count) in &results {
            assert!(*count > 0, "Thread {} should have made progress", name);
        }
    }

    #[test]
    fn test_cache_consistency_under_concurrent_access() {
        use std::sync::{Arc, Barrier};
        use std::thread;
        
        clear_cache();
        
        let num_threads = 10;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = Vec::new();
        
        // All threads will try to get the same cache entry simultaneously
        // This tests the race condition where multiple threads might try to 
        // generate and cache the same keys at the same time
        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                
                // All threads request the same parameters simultaneously
                let keys = get_or_generate_keys(32, 0x04C11DB7, true);
                (i, keys)
            });
            handles.push(handle);
        }
        
        // Collect results from all threads
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().expect("Thread should not panic"));
        }
        
        // Verify all threads got identical keys
        assert_eq!(results.len(), num_threads);
        let first_keys = results[0].1;
        
        for (thread_id, keys) in results {
            assert_eq!(keys, first_keys, 
                "Thread {} should get identical keys to other threads", thread_id);
        }
        
        // Verify the keys are correct by comparing with direct generation
        let expected_keys = generate::keys(32, 0x04C11DB7, true);
        assert_eq!(first_keys, expected_keys, 
            "Cached keys should match directly generated keys");
        
        // Verify subsequent access still returns the same keys
        let final_keys = get_or_generate_keys(32, 0x04C11DB7, true);
        assert_eq!(final_keys, first_keys, 
            "Final cache access should return same keys");
    }

    #[test]
    fn test_mixed_concurrent_operations() {
        use std::sync::{Arc, Barrier};
        use std::thread;
        use std::time::Duration;
        
        clear_cache();
        
        let num_threads = 8;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = Vec::new();
        
        // Mix of operations: some threads do cache hits, some do cache misses,
        // some clear the cache, all happening concurrently
        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                
                let mut operations = 0;
                let start = std::time::Instant::now();
                
                while start.elapsed() < Duration::from_millis(30) {
                    match i % 4 {
                        0 => {
                            // Cache hit operations - same parameters
                            let _keys = get_or_generate_keys(32, 0x04C11DB7, true);
                        },
                        1 => {
                            // Cache miss operations - different parameters each time
                            let poly = 0x1EDC6F41 + (operations as u64);
                            let _keys = get_or_generate_keys(32, poly, true);
                        },
                        2 => {
                            // Mixed read operations
                            let _keys1 = get_or_generate_keys(32, 0x04C11DB7, true);
                            let _keys2 = get_or_generate_keys(64, 0x42F0E1EBA9EA3693, false);
                        },
                        3 => {
                            // Occasional cache clear (but not too often to avoid disrupting other tests)
                            if operations % 10 == 0 {
                                clear_cache();
                            }
                            let _keys = get_or_generate_keys(32, 0x04C11DB7, true);
                        },
                        _ => unreachable!(),
                    }
                    operations += 1;
                }
                
                (i, operations)
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().expect("Thread should not panic"));
        }
        
        // Verify all threads completed successfully
        assert_eq!(results.len(), num_threads);
        
        for (thread_id, operations) in results {
            assert!(operations > 0, 
                "Thread {} should have completed some operations", thread_id);
        }
        
        // Verify cache is still functional after all the concurrent operations
        let final_keys = get_or_generate_keys(32, 0x04C11DB7, true);
        let expected_keys = generate::keys(32, 0x04C11DB7, true);
        assert_eq!(final_keys, expected_keys, 
            "Cache should still work correctly after concurrent operations");
    }

    #[test]
    fn test_lock_poisoning_recovery() {
        use std::sync::{Arc, Barrier};
        use std::thread;
        
        clear_cache();
        
        // This test is tricky because we need to poison the lock without 
        // actually breaking our test. We'll simulate lock poisoning by
        // creating a scenario where a thread panics while holding a write lock.
        // However, since our implementation uses best-effort error handling,
        // it should gracefully degrade rather than propagate panics.
        
        // First, verify normal operation
        let keys_before = get_or_generate_keys(32, 0x04C11DB7, true);
        assert_eq!(keys_before.len(), 23);
        
        // Test that even if internal operations fail, the function still returns valid keys
        // We can't easily poison the lock in a controlled way, but we can verify
        // that our error handling works by testing edge cases
        
        // Multiple rapid cache operations that might stress the locking mechanism
        let num_threads = 4;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = Vec::new();
        
        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                
                // Rapid cache operations that might cause contention
                for j in 0..20 {
                    let poly = 0x04C11DB7 + (i as u64 * 1000) + (j as u64);
                    let keys = get_or_generate_keys(32, poly, true);
                    assert_eq!(keys.len(), 23, 
                        "Thread {} iteration {} should return valid keys", i, j);
                    
                    // Occasional cache clear to increase contention
                    if j % 7 == 0 {
                        clear_cache();
                    }
                }
                
                i
            });
            handles.push(handle);
        }
        
        // All threads should complete successfully
        for handle in handles {
            let thread_id = handle.join().expect("Thread should not panic");
            assert!(thread_id < num_threads);
        }
        
        // Verify cache is still functional after stress testing
        let keys_after = get_or_generate_keys(32, 0x04C11DB7, true);
        assert_eq!(keys_after.len(), 23);
        
        // Keys should be mathematically correct regardless of cache state
        let expected_keys = generate::keys(32, 0x04C11DB7, true);
        assert_eq!(keys_after, expected_keys);
    }

    #[test]
    fn test_cache_behavior_with_thread_local_access() {
        use std::thread;
        
        clear_cache();
        
        // Test that cache works correctly when accessed from different threads
        // in sequence (not concurrently)
        
        let keys_main = get_or_generate_keys(32, 0x04C11DB7, true);
        
        let handle = thread::spawn(|| {
            // This thread should see the cached value from the main thread
            let keys_thread = get_or_generate_keys(32, 0x04C11DB7, true);
            keys_thread
        });
        
        let keys_from_thread = handle.join().expect("Thread should not panic");
        
        // Both should be identical
        assert_eq!(keys_main, keys_from_thread);
        
        // Test multiple sequential threads
        let mut thread_keys = Vec::new();
        
        for i in 0..5 {
            let handle = thread::spawn(move || {
                let keys = get_or_generate_keys(32, 0x04C11DB7, true);
                (i, keys)
            });
            
            let (thread_id, keys) = handle.join().expect("Thread should not panic");
            thread_keys.push((thread_id, keys));
        }
        
        // All threads should get the same cached keys
        for (thread_id, keys) in thread_keys {
            assert_eq!(keys, keys_main, 
                "Thread {} should get same cached keys", thread_id);
        }
    }
}