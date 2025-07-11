# Design Document

## Overview

This design implements a future-proof CrcParams structure using an internal enum-based key storage system that can expand to support different key array sizes without breaking compatibility. The approach maintains the simplicity of const definitions while providing safe key access and zero runtime overhead through compiler optimizations.

## Architecture

### Core Components

1. **CrcKeysStorage Enum**: Internal storage that can hold different key array sizes
2. **CrcParams Structure**: Updated to use CrcKeysStorage internally while maintaining public API
3. **Safe Accessor Methods**: Bounds-checked key access methods on CrcParams
4. **Helper Functions**: Const-friendly constructors for CrcKeysStorage variants

### Design Principles

- **Zero Runtime Overhead**: Enum dispatch optimized away by compiler
- **Backwards Compatibility**: Existing const definitions require minimal changes
- **Gradual Migration**: Can be implemented in phases without breaking builds
- **Safety First**: Bounds checking prevents panics from out-of-range access

## Components and Interfaces

### CrcKeysStorage Enum

```rust
#[derive(Clone, Copy, Debug)]
enum CrcKeysStorage {
    /// Current 23-key format (for existing algorithms which includes 256 byte folding distances)
    KeysFold256([u64; 23]),
    /// Future 25-key format (for potential future expanded folding distances, for testing purposes only)
    KeysFutureTest([u64; 25]),
    // Additional variants can be added as needed
}
```

**Key Methods:**
- `get_key(index: usize) -> u64`: Safe key access with bounds checking
- `key_count() -> usize`: Returns actual number of keys available
- `from_keys_fold_256(keys: [u64; 23]) -> Self`: Const constructor for 23-key arrays
- `from_keys_fold_future_test(keys: [u64; 25]) -> Self`: Const constructor for 25-key arrays

### Updated CrcParams Structure

```rust
#[derive(Clone, Copy, Debug)]
pub struct CrcParams {
    pub algorithm: CrcAlgorithm,
    pub name: &'static str,
    pub width: u8,
    pub poly: u64,
    pub init: u64,
    pub refin: bool,
    pub refout: bool,
    pub xorout: u64,
    pub check: u64,
    pub keys: CrcKeysStorage,  // Changed from [u64; 23]
}
```

**Key Methods:**
- `get_key(index: usize) -> u64`: Delegates to CrcKeysStorage
- `get_key_checked(index: usize) -> Option<u64>`: Optional key access
- `key_count() -> usize`: Returns actual key count

### Const Definition Pattern

```rust
// Before (Phase 2):
pub const CRC32_ISCSI: CrcParams = CrcParams {
    // ... other fields unchanged ...
    keys: KEYS_1EDC6F41_REFLECTED,  // [u64; 23]
};

// After (Phase 3):
pub const CRC32_ISCSI: CrcParams = CrcParams {
    // ... other fields unchanged ...
    keys: CrcKeysStorage::from_keys_fold_256(KEYS_1EDC6F41_REFLECTED),
};
```

## Data Models

### Key Storage Variants

| Variant | Array Size | Use Case |
|---------|------------|----------|
| KeysFold256 | [u64; 23] | Current implementation (128/256-byte folding) |
| KeysFutureTest | [u64; 25] | Future expansion |

### Migration States

| Phase | CrcParams.keys Type | Architecture Code | Const Definitions |
|-------|-------------------|------------------|------------------|
| 1 | [u64; 23] | Direct access | Unchanged |
| 2 | [u64; 23] | Safe accessors | Unchanged |
| 3 | CrcKeysStorage | Safe accessors | Updated |

## Error Handling

### Bounds Checking Strategy

1. **Safe Default**: Out-of-bounds key access returns 0 instead of panicking
2. **Optional Access**: `get_key_checked()` returns `None` for invalid indices
3. **Graceful Degradation**: Code continues to function with missing keys

### Error Scenarios

| Scenario | Behavior | Rationale |
|----------|----------|-----------|
| Access key[30] with 23-key storage | Returns 0 | Allows future expansion without breaking existing code |
| Invalid key index | Returns 0 | Prevents panics, maintains stability |
| Empty key storage | Returns 0 for all indices | Defensive programming |

## Testing Strategy

### Unit Tests

1. **CrcKeysStorage Tests**:
   - Verify correct key storage and retrieval for each variant
   - Test bounds checking behavior
   - Validate const constructor functions

2. **CrcParams Integration Tests**:
   - Verify safe accessor methods work correctly
   - Test backwards compatibility with existing const definitions
   - Validate zero runtime overhead through benchmarks

3. **Migration Tests**:
   - Test each phase independently
   - Verify existing functionality remains intact
   - Validate const definition updates

### Compatibility Tests

1. **Third-Party Simulation**:
   - Create mock third-party const definitions
   - Verify they continue working through all phases
   - Test key access patterns used by external code

2. **Performance Tests**:
   - Benchmark key access performance vs direct array access
   - Verify compiler optimizations eliminate enum dispatch
   - Measure memory usage impact

### Integration Tests

1. **Architecture Code Tests**:
   - Update existing architecture tests to use safe accessors
   - Verify SIMD operations work correctly with new key access
   - Test folding operations across different key storage variants

2. **End-to-End Tests**:
   - Verify CRC calculations remain correct after migration
   - Test custom CrcParams creation and usage
   - Validate `get-custom-params` binary output

## Implementation Phases

### Phase 1: Add New Types
- Add CrcKeysStorage enum to codebase
- Add helper methods to CrcParams (delegating to existing keys field)
- Maintain existing [u64; 23] field for compatibility
- All tests continue to pass

### Phase 2: Update Architecture Code
- Replace direct key array access with safe accessor methods
- Update SIMD and folding code to use `params.get_key(index)`
- Maintain backwards compatibility
- Performance remains identical

### Phase 3: Switch to New Storage
- Change CrcParams.keys field from [u64; 23] to CrcKeysStorage
- Update all const definitions to use CrcKeysStorage::from_keys_23()
- Update `get-custom-params` binary output format
- This is the only breaking change, but minimal impact

## Performance Considerations

### Compiler Optimizations

The Rust compiler optimizes enum dispatch when:
1. All variants have the same access pattern
2. The enum is used in hot paths with predictable patterns
3. Inlining is enabled for accessor methods

Expected assembly output for `params.get_key(21)`:
```assembly
; Same as direct array access keys[21]
mov rax, qword ptr [rdi + 168 + 21*8]
```

### Memory Layout

| Storage Type | Memory Usage | Alignment |
|--------------|--------------|-----------|
| KeysFold256 | 184 bytes | 8-byte aligned |
| KeysFutureTest | 200 bytes | 8-byte aligned |
| Enum overhead | 0 bytes | (optimized away) |

## Security Considerations

### Bounds Safety

The new design eliminates array bounds panics, which could be exploited in unsafe contexts. Safe key access prevents:
- Buffer overflow attacks through malicious key indices
- Denial of service through panic-induced crashes
- Information disclosure through out-of-bounds memory access

### Const Safety

All const definitions remain compile-time validated, preventing:
- Runtime key generation vulnerabilities
- Dynamic key modification attacks
- Timing-based side-channel attacks on key access