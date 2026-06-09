-- ag-workers durable job table. One row per live job (non-terminal; terminal jobs
-- are removed on ack or moved to the dead-letter table).
CREATE TABLE IF NOT EXISTS ag_worker_jobs (
    id              UUID         PRIMARY KEY,
    kind            TEXT         NOT NULL,
    queue           TEXT         NOT NULL,
    payload         BYTEA        NOT NULL,
    payload_version INTEGER      NOT NULL DEFAULT 1,
    priority        SMALLINT     NOT NULL DEFAULT 1,
    status          TEXT         NOT NULL DEFAULT 'queued',
    attempts        INTEGER      NOT NULL DEFAULT 0,
    max_attempts    INTEGER      NOT NULL,
    timeout_ms      BIGINT       NOT NULL,
    scheduled_at    TIMESTAMPTZ,
    available_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    locked_by       TEXT,
    lock_until      TIMESTAMPTZ,
    dedup_key       TEXT,
    tenant_id       TEXT,
    trace_id        TEXT,
    last_error      TEXT
);

-- Leasing scan: eligible rows per queue ordered by priority then eligibility time.
CREATE INDEX IF NOT EXISTS ag_worker_jobs_lease_idx
    ON ag_worker_jobs (queue, priority DESC, available_at ASC, created_at ASC)
    WHERE status IN ('queued', 'retry_scheduled', 'scheduled');

-- Optional best-effort dedup: at most one non-terminal job per (queue, dedup_key).
CREATE UNIQUE INDEX IF NOT EXISTS ag_worker_jobs_dedup_idx
    ON ag_worker_jobs (queue, dedup_key)
    WHERE dedup_key IS NOT NULL
      AND status NOT IN ('succeeded', 'failed', 'dead_lettered', 'cancelled', 'expired');
