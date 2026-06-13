-- Smoke migration fixture for the ag-data live integration test
-- (connect_and_run_migrations). It only creates a marker table so the test
-- can assert that run_migrations applied a real migration. Not a product
-- migration: ag-data ships no migrations of its own; consuming crates own theirs.
CREATE TABLE IF NOT EXISTS ag_data_smoke (
    id BIGINT PRIMARY KEY
);
