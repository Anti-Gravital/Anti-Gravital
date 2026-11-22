//! Golden serde strings for generated timestamp fields (RFC-0026, issue #185).
//!
//! Generated models carry `#[serde(with = "time::serde::rfc3339")]` (and the
//! `option` module for nullable fields) so their JSON contract stays an RFC 3339
//! string. These are exact-string regression tests, not round trips: a round
//! trip passes even when the textual representation changes.

use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, macros::datetime, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Stamp {
    #[serde(with = "time::serde::rfc3339")]
    at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct OptionalStamp {
    #[serde(with = "time::serde::rfc3339::option")]
    at: Option<OffsetDateTime>,
}

#[test]
fn whole_seconds_serialize_as_rfc3339_string() {
    let stamp = Stamp {
        at: datetime!(2025-01-02 04:00:00 UTC),
    };
    let json = serde_json::to_string(&stamp).unwrap();
    assert_eq!(json, r#"{"at":"2025-01-02T04:00:00Z"}"#);
    let back: Stamp = serde_json::from_str(&json).unwrap();
    assert_eq!(back, stamp);
}

#[test]
fn fractional_seconds_keep_exact_digits() {
    let nanos = OffsetDateTime::parse("2025-01-02T04:00:00.123456789Z", &Rfc3339).unwrap();
    let json = serde_json::to_string(&Stamp { at: nanos }).unwrap();
    assert_eq!(json, r#"{"at":"2025-01-02T04:00:00.123456789Z"}"#);

    // BREAKING CHANGE (RFC-0026): chrono serialized a trailing-zero fraction as
    // `...00.100Z`; time's rfc3339 adapter emits `...00.1Z`. This is a real wire
    // change and is documented as such, not claimed as compatible.
    let hundred_ms = OffsetDateTime::parse("2025-01-02T04:00:00.1Z", &Rfc3339).unwrap();
    let json = serde_json::to_string(&Stamp { at: hundred_ms }).unwrap();
    assert_eq!(json, r#"{"at":"2025-01-02T04:00:00.1Z"}"#);
}

#[test]
fn utc_spelling_uses_z_not_plus_zero_zero() {
    let stamp = Stamp {
        at: datetime!(2025-01-02 04:00:00 UTC),
    };
    // BREAKING CHANGE (RFC-0026): chrono's explicit `to_rfc3339()` spelled UTC
    // as `+00:00`; time's `Rfc3339` formatter uses `Z`.
    assert_eq!(stamp.at.format(&Rfc3339).unwrap(), "2025-01-02T04:00:00Z");
    let json = serde_json::to_string(&stamp).unwrap();
    assert!(json.contains("Z"), "{json}");
}

#[test]
fn non_utc_offset_is_normalized_to_utc() {
    let parsed = OffsetDateTime::parse("2025-01-02T05:00:00+01:00", &Rfc3339)
        .unwrap()
        .to_offset(time::UtcOffset::UTC);
    let json = serde_json::to_string(&Stamp { at: parsed }).unwrap();
    assert_eq!(json, r#"{"at":"2025-01-02T04:00:00Z"}"#);
}

#[test]
fn optional_timestamp_missing_is_null() {
    let json = serde_json::to_string(&OptionalStamp { at: None }).unwrap();
    assert_eq!(json, r#"{"at":null}"#);

    let stamp = OptionalStamp {
        at: Some(datetime!(2025-01-02 04:00:00 UTC)),
    };
    let json = serde_json::to_string(&stamp).unwrap();
    assert_eq!(json, r#"{"at":"2025-01-02T04:00:00Z"}"#);
    let back: OptionalStamp = serde_json::from_str(&json).unwrap();
    assert_eq!(back, stamp);
}
