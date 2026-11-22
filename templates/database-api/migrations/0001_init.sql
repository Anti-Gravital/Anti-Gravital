-- Initial migration for {{name}}.
-- Applied by ag_data::run_migrations when the postgres feature is enabled.

CREATE TABLE IF NOT EXISTS records (
    id    TEXT   PRIMARY KEY,
    name  TEXT   NOT NULL,
    value BIGINT NOT NULL
);
