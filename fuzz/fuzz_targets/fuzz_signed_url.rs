#![no_main]
//! Fuzzes ag-storage signed-URL token verification.
//!
//! `verify_signed_url` parses an untrusted token (timestamp + HMAC) and must
//! never panic on malformed input. A rejected or expired token must return an
//! error, never a false positive, and the HMAC comparison is constant-time.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(token) = std::str::from_utf8(data) {
        let _ = ag_storage::verify_signed_url("fuzz-secret-key", "objects/key", token);
    }
});
