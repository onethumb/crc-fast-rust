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