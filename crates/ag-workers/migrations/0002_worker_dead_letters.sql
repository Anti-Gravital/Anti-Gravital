-- ag-workers dead-letter table. A dead-lettered job is preserved here, never dropped.
CREATE TABLE IF NOT EXISTS ag_worker_dead_letters (
    id               UUID         PRIMARY KEY,
    kind             TEXT         NOT NULL,
    queue            TEXT         NOT NULL,
    payload          BYTEA        NOT NULL,
    payload_version  INTEGER      NOT NULL DEFAULT 1,
    priority         SMALLINT     NOT NULL DEFAULT 1,
    attempts         INTEGER      NOT NULL DEFAULT 0,
    max_attempts     INTEGER      NOT NULL,
    timeout_ms       BIGINT       NOT NULL,
    created_at       TIMESTAMPTZ  NOT NULL,
    dedup_key        TEXT,
    tenant_id        TEXT,
    trace_id         TEXT,
    last_error       TEXT,
    reason           TEXT         NOT NULL,
    dead_lettered_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Inspection by queue, newest first.
CREATE INDEX IF NOT EXISTS ag_worker_dead_letters_queue_idx
    ON ag_worker_dead_letters (queue, dead_lettered_at DESC);
