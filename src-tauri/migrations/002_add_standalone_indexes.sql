-- Add standalone indexes for account_id and category_id
-- These complement existing composite indexes for better query performance
-- when filtering by account_id or category_id alone

-- Note: idx_transactions_date already exists from initial migration
-- Note: Composite indexes (idx_transactions_account_date, idx_transactions_category_date)
--       already exist but aren't efficient for account_id/category_id-only queries

CREATE INDEX IF NOT EXISTS idx_transactions_account_id ON transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_category_id ON transactions(category_id);
