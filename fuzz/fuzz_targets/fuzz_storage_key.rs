#![no_main]
//! Fuzzes ag-storage object-key validation.
//!
//! `validate_key` is the security boundary against path traversal and unsafe
//! object keys. It must never panic on arbitrary input and must reject keys
//! with `..`, leading/trailing `/`, `//`, null bytes or control characters.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(key) = std::str::from_utf8(data) {
        let _ = ag_storage::store::validate_key(key);
    }
});
