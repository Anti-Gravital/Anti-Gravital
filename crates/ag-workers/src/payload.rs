//! Versioned payload encoding.
//!
//! Job payloads are encoded with `rmp-serde` (MessagePack) for compact,
//! schema-tolerant binary persistence. Durable jobs outlive deploys, so every payload
//! type carries a [`JobPayload::PAYLOAD_VERSION`]; a job enqueued before a redeploy can
//! still be decoded — or safely dead-lettered as `InvalidPayload` — after it.
//!
//! Typed structs live only at the API boundary; the runtime stores and moves opaque
//! [`PayloadBytes`] and never reasons about untyped maps.

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// Opaque, encoded payload bytes as stored and moved by the runtime.
///
/// On the PostgreSQL backend this maps to a `BYTEA` column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadBytes(Vec<u8>);

impl PayloadBytes {
    /// Wraps already-encoded bytes.
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the encoded bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns the number of encoded bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are no encoded bytes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consumes the wrapper and returns the owned bytes.
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

/// Errors produced while encoding, decoding or migrating a payload.
#[derive(Debug, Error)]
pub enum PayloadError {
    /// The typed payload could not be serialized.
    #[error("payload encode failed: {0}")]
    Encode(String),

    /// The bytes could not be deserialized into the target type.
    #[error("payload decode failed: {0}")]
    Decode(String),

    /// The stored payload version is not understood and no migration handled it.
    #[error("payload version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        /// Version the current handler understands.
        expected: u16,
        /// Version recorded on the stored job.
        found: u16,
    },

    /// The payload exceeds the configured maximum size.
    #[error("payload too large: {actual} bytes exceeds limit of {limit} bytes")]
    TooLarge {
        /// Encoded size that was rejected.
        actual: usize,
        /// Configured maximum.
        limit: usize,
    },
}

/// Encodes a typed payload into [`PayloadBytes`].
pub fn encode<P: Serialize>(payload: &P) -> Result<PayloadBytes, PayloadError> {
    rmp_serde::to_vec_named(payload)
        .map(PayloadBytes::from_vec)
        .map_err(|e| PayloadError::Encode(e.to_string()))
}

/// Decodes [`PayloadBytes`] into a typed payload, ignoring versioning.
///
/// Prefer [`decode_versioned`] on the dispatch path so version mismatches become an
/// explicit, recoverable error instead of a decode failure.
pub fn decode<P: DeserializeOwned>(bytes: &PayloadBytes) -> Result<P, PayloadError> {
    rmp_serde::from_slice(bytes.as_slice()).map_err(|e| PayloadError::Decode(e.to_string()))
}

/// A typed job payload with a stable version and an optional migration hook.
///
/// Implemented by the structs generated from the DSL `worker` declaration (or written
/// by hand). The default [`migrate`](JobPayload::migrate) refuses older versions; a
/// payload type overrides it to upgrade older encodings in place.
pub trait JobPayload: Serialize + DeserializeOwned + Sized {
    /// Current schema version of this payload. Bump on incompatible changes.
    const PAYLOAD_VERSION: u16 = 1;

    /// Upgrades a previously-encoded payload of `from_version` into the current type.
    ///
    /// The default rejects the mismatch; the dispatch path then dead-letters the job
    /// as `InvalidPayload` rather than panicking or dropping it.
    fn migrate(from_version: u16, _bytes: &[u8]) -> Result<Self, PayloadError> {
        Err(PayloadError::VersionMismatch {
            expected: Self::PAYLOAD_VERSION,
            found: from_version,
        })
    }
}

/// Decodes a payload taking the stored version into account.
///
/// If `stored_version` matches the type's [`JobPayload::PAYLOAD_VERSION`], the bytes are
/// decoded directly. Otherwise the type's [`JobPayload::migrate`] hook is given a
/// chance to upgrade them. Any failure is returned as a [`PayloadError`] that the
/// dispatch path turns into an `InvalidPayload` dead-letter.
pub fn decode_versioned<P: JobPayload>(
    bytes: &PayloadBytes,
    stored_version: u16,
) -> Result<P, PayloadError> {
    if stored_version == P::PAYLOAD_VERSION {
        decode(bytes)
    } else {
        P::migrate(stored_version, bytes.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct SendReceipt {
        user_id: u64,
        amount_cents: u64,
    }

    impl JobPayload for SendReceipt {
        const PAYLOAD_VERSION: u16 = 2;
    }

    // A v1 shape that migrates into the current SendReceipt (amount in whole units).
    #[derive(Debug, Serialize, Deserialize)]
    struct SendReceiptV1 {
        user_id: u64,
        amount: u64,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct WithMigration {
        user_id: u64,
        amount_cents: u64,
    }

    impl JobPayload for WithMigration {
        const PAYLOAD_VERSION: u16 = 2;

        fn migrate(from_version: u16, bytes: &[u8]) -> Result<Self, PayloadError> {
            if from_version == 1 {
                let old: SendReceiptV1 = rmp_serde::from_slice(bytes)
                    .map_err(|e| PayloadError::Decode(e.to_string()))?;
                Ok(WithMigration {
                    user_id: old.user_id,
                    amount_cents: old.amount * 100,
                })
            } else {
                Err(PayloadError::VersionMismatch {
                    expected: Self::PAYLOAD_VERSION,
                    found: from_version,
                })
            }
        }
    }

    #[test]
    fn encode_decode_roundtrip() {
        let p = SendReceipt {
            user_id: 7,
            amount_cents: 999,
        };
        let bytes = encode(&p).unwrap();
        let back: SendReceipt = decode(&bytes).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn matching_version_decodes_directly() {
        let p = SendReceipt {
            user_id: 1,
            amount_cents: 50,
        };
        let bytes = encode(&p).unwrap();
        let back: SendReceipt = decode_versioned(&bytes, SendReceipt::PAYLOAD_VERSION).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn missing_migration_is_version_mismatch() {
        let p = SendReceipt {
            user_id: 1,
            amount_cents: 50,
        };
        let bytes = encode(&p).unwrap();
        let err = decode_versioned::<SendReceipt>(&bytes, 1).unwrap_err();
        assert!(matches!(
            err,
            PayloadError::VersionMismatch {
                expected: 2,
                found: 1
            }
        ));
    }

    #[test]
    fn migrate_hook_upgrades_old_version() {
        let old = SendReceiptV1 {
            user_id: 42,
            amount: 3,
        };
        let bytes = PayloadBytes::from_vec(rmp_serde::to_vec_named(&old).unwrap());
        let migrated: WithMigration = decode_versioned(&bytes, 1).unwrap();
        assert_eq!(
            migrated,
            WithMigration {
                user_id: 42,
                amount_cents: 300
            }
        );
    }
}
