-- ag-mail persistent queue. One row per queued email.
CREATE TABLE IF NOT EXISTS ag_mail_queue (
    id            UUID PRIMARY KEY,
    payload       JSONB        NOT NULL,
    status        TEXT         NOT NULL DEFAULT 'pending',
    attempts      INTEGER      NOT NULL DEFAULT 0,
    max_retries   INTEGER      NOT NULL,
    next_retry_at TIMESTAMPTZ  NOT NULL DEFAULT now(),
    last_error    TEXT,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Worker polls pending rows whose retry time has passed, oldest first.
CREATE INDEX IF NOT EXISTS ag_mail_queue_due_idx
    ON ag_mail_queue (status, next_retry_at)
    WHERE status = 'pending';
