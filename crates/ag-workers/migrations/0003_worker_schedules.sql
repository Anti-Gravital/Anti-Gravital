-- ag-workers recurring interval schedules. A single due tick is claimed atomically
-- (FOR UPDATE SKIP LOCKED) so an interval job fires once across N worker processes.
CREATE TABLE IF NOT EXISTS ag_worker_schedules (
    id              UUID         PRIMARY KEY,
    kind            TEXT         NOT NULL,
    queue           TEXT         NOT NULL,
    payload         BYTEA        NOT NULL,
    payload_version INTEGER      NOT NULL DEFAULT 1,
    interval_ms     BIGINT       NOT NULL,
    next_run_at     TIMESTAMPTZ  NOT NULL,
    misfire         TEXT         NOT NULL DEFAULT 'skip',
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Due scan: earliest next_run_at first.
CREATE INDEX IF NOT EXISTS ag_worker_schedules_due_idx
    ON ag_worker_schedules (next_run_at);
