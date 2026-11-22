//! TIMESTAMPTZ binary round-trip for `time::OffsetDateTime` (RFC-0026 step 3).
//!
//! Requires a live PostgreSQL instance via `DATABASE_URL`; ignored by default so
//! CI without a database stays green (repository convention). Run with:
//!
//! ```text
//! DATABASE_URL=postgres://user:pass@localhost/ag_workers_test \
//!   cargo test -p ag-workers --features postgres --test timestamptz_roundtrip -- --ignored
//! ```
#![cfg(feature = "postgres")]

use time::{format_description::well_known::Rfc3339, macros::datetime, OffsetDateTime};

#[tokio::test]
#[ignore = "requires DATABASE_URL pointing at a PostgreSQL instance"]
async fn timestamptz_roundtrip_preserves_microseconds() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for this test");
    let pool = sqlx::PgPool::connect(&url)
        .await
        .expect("connect to PostgreSQL");

    let original = datetime!(2025-01-02 04:00:00.123456 UTC);
    let stored: OffsetDateTime = sqlx::query_scalar("SELECT $1::timestamptz")
        .bind(original)
        .fetch_one(&pool)
        .await
        .expect("bind and read TIMESTAMPTZ");
    assert_eq!(stored, original);

    // PostgreSQL TIMESTAMPTZ has microsecond resolution: nanoseconds beyond six
    // digits are truncated. Tests must not assert nanosecond equality (RFC-0026).
    let nanos = OffsetDateTime::parse("2025-01-02T04:00:00.123456789Z", &Rfc3339).unwrap();
    let stored: OffsetDateTime = sqlx::query_scalar("SELECT $1::timestamptz")
        .bind(nanos)
        .fetch_one(&pool)
        .await
        .expect("bind and read TIMESTAMPTZ");
    assert_eq!(stored, datetime!(2025-01-02 04:00:00.123456 UTC));
    assert_eq!(stored.unix_timestamp_nanos() % 1_000, 0);
}
