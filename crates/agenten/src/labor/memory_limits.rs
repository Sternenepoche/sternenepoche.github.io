//! Shared logical and physical limits for trusted SQLite and container SQLite.
pub const MAX_RECORDS: u64 = 4096;
pub const RECORD_OVERHEAD: u64 = 128;
pub const META_KEYS: usize = 32;
pub const META_VALUE_BYTES: usize = 65_536;
pub const META_BYTES: usize = 1_048_576;
pub const DB_MAX_PAGES: u64 = 7168;
pub fn record_bytes(kind: &str, key: &str, serialized: &str) -> u64 {
    (kind.len() + key.len() + serialized.len()) as u64 + RECORD_OVERHEAD
}
