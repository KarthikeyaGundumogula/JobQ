-- Add migration script here
CREATE TYPE job_state AS ENUM (
  'Queued',
  'Running',
  'Succeeded',
  'Failed',
  'Dead',
  'Cancelled'
)q

CREATE TABLE
  jobq (
    job_id UUID NOT NULL PRIMARY KEY,
    job_type VARCHAR(25) NOT NULL,
    payload JSONB NOT NULL,
    state job_state NOT NULL DEFAULT 'Queued',
    attempts SMALLINT NOT NULL DEFAULT 0,
    max_attempts SMALLINT NOT NULL CHECK (max_attempts > 0),
    run_at TIMESTAMPTZ NOT NULL,
    retry_policy JSONB NOT NULL
  );