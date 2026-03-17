-- Add migration script here
ALTER TABLE jobq
ADD COLUMN locked_by UUID ,
ADD COLUMN lease_expires_at TIMESTAMPTZ;