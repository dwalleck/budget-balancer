-- Add updated_at column to column_mappings table
-- This supports tracking when mappings are modified via the upsert behavior

ALTER TABLE column_mappings ADD COLUMN updated_at TEXT;
