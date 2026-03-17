-- Add migration script here
CREATE INDEX IF NOT EXISTS idx_jobq_state_run_at 
ON jobq(state,run_at);