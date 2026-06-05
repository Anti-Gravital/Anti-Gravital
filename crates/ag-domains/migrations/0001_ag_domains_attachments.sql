-- ag-domains control-plane store (RFC-0009 phase D).
-- Applied at runtime by SqlAttachmentStore::migrate; kept here for reference
-- and for use with external migration tooling.

CREATE TABLE IF NOT EXISTS domain_attachment (
    id             TEXT PRIMARY KEY,
    hostname_ascii TEXT NOT NULL UNIQUE,
    lifecycle      TEXT NOT NULL,
    data           JSONB NOT NULL,
    created_at     BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS domain_tombstone (
    hostname_ascii     TEXT PRIMARY KEY,
    last_attachment_id TEXT NOT NULL,
    tombstone_until    BIGINT NOT NULL,
    reason             TEXT NOT NULL
);
