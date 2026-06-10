#![no_main]
//! Fuzzes the `ag-workers` payload decode path.
//!
//! Durable jobs outlive deploys, so the runtime decodes `rmp-serde` payload bytes
//! that may be arbitrary, truncated, or version-mismatched (RFC-0012 §9.4, §25).
//! The decode boundary is a security property: on any input it must return an
//! `Err` (which the dispatch path turns into an `InvalidPayload` dead-letter) and
//! must **never panic, hang, or read out of bounds**. This target drives both the
//! direct decode and the version-aware decode with adversarial bytes.

use ag_workers::{decode, decode_versioned, JobPayload, PayloadBytes};
use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

/// A representative typed payload with a few fields of different shapes, so the
/// decoder exercises map/int/string/seq handling rather than a single scalar.
#[derive(Debug, Serialize, Deserialize)]
struct FuzzPayload {
    id: u64,
    name: String,
    flags: Vec<bool>,
    nested: Option<Inner>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Inner {
    a: i32,
    b: String,
}

impl JobPayload for FuzzPayload {
    const PAYLOAD_VERSION: u16 = 2;
}

fuzz_target!(|data: &[u8]| {
    // The first byte (if present) seeds the stored version so the version-aware
    // path also sees mismatches that must route to the migrate hook / error, not
    // a panic. The rest are the candidate payload bytes.
    let (stored_version, rest) = match data.split_first() {
        Some((v, rest)) => (u16::from(*v), rest),
        None => (0, &[][..]),
    };
    let bytes = PayloadBytes::from_vec(rest.to_vec());

    // Direct decode: arbitrary bytes must not panic.
    let _: Result<FuzzPayload, _> = decode(&bytes);

    // Version-aware decode: arbitrary version + bytes must not panic.
    let _: Result<FuzzPayload, _> = decode_versioned(&bytes, stored_version);
});
