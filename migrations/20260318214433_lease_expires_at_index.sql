-- Add migration script here
CREATE INDEX IF NOT EXISTS idx_lease_expiry
ON jobq(lease_expires_at);
