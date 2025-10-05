-- Budget Balancer Initial Schema
-- Created: 2025-10-04

-- Accounts table
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL CHECK(type IN ('checking', 'savings', 'credit_card')),
    balance REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Categories table
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL CHECK(type IN ('predefined', 'custom')),
    parent_id INTEGER,
    icon TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX idx_categories_type ON categories(type);

-- Category rules for automatic categorization
CREATE TABLE category_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL,
    category_id INTEGER NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX idx_category_rules_pattern ON category_rules(pattern);
CREATE INDEX idx_category_rules_priority ON category_rules(priority DESC);

-- Transactions table
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    description TEXT NOT NULL,
    merchant TEXT,
    hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT
);

CREATE INDEX idx_transactions_date ON transactions(date DESC);
CREATE INDEX idx_transactions_category_date ON transactions(category_id, date);
CREATE INDEX idx_transactions_account_date ON transactions(account_id, date);
CREATE INDEX idx_transactions_hash ON transactions(hash);

-- Debts table
CREATE TABLE debts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    balance REAL NOT NULL CHECK(balance >= 0),
    original_balance REAL NOT NULL CHECK(original_balance >= 0),
    interest_rate REAL NOT NULL CHECK(interest_rate >= 0 AND interest_rate <= 100),
    min_payment REAL NOT NULL CHECK(min_payment >= 0),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_debts_balance ON debts(balance);
CREATE INDEX idx_debts_interest_rate ON debts(interest_rate DESC);

-- Debt plans table
CREATE TABLE debt_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    strategy TEXT NOT NULL CHECK(strategy IN ('avalanche', 'snowball')),
    monthly_amount REAL NOT NULL CHECK(monthly_amount > 0),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Debt payments table
CREATE TABLE debt_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    debt_id INTEGER NOT NULL,
    amount REAL NOT NULL CHECK(amount > 0),
    date TEXT NOT NULL,
    plan_id INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (debt_id) REFERENCES debts(id) ON DELETE CASCADE,
    FOREIGN KEY (plan_id) REFERENCES debt_plans(id) ON DELETE SET NULL
);

CREATE INDEX idx_debt_payments_debt_date ON debt_payments(debt_id, date);
CREATE INDEX idx_debt_payments_plan ON debt_payments(plan_id);

-- Spending targets table
CREATE TABLE spending_targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    amount REAL NOT NULL CHECK(amount > 0),
    period TEXT NOT NULL CHECK(period IN ('monthly', 'quarterly', 'yearly')),
    start_date TEXT NOT NULL,
    end_date TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX idx_spending_targets_category ON spending_targets(category_id);
CREATE INDEX idx_spending_targets_dates ON spending_targets(start_date, end_date);

-- Column mappings table
CREATE TABLE column_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_name TEXT NOT NULL UNIQUE,
    date_col TEXT NOT NULL,
    amount_col TEXT NOT NULL,
    description_col TEXT NOT NULL,
    merchant_col TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Seed data: Predefined categories
INSERT OR IGNORE INTO categories (id, name, type, icon) VALUES
    (1, 'Groceries', 'predefined', '🛒'),
    (2, 'Dining', 'predefined', '🍽️'),
    (3, 'Transportation', 'predefined', '🚗'),
    (4, 'Entertainment', 'predefined', '🎬'),
    (5, 'Utilities', 'predefined', '⚡'),
    (6, 'Healthcare', 'predefined', '🏥'),
    (7, 'Shopping', 'predefined', '🛍️'),
    (8, 'Travel', 'predefined', '✈️'),
    (9, 'Income', 'predefined', '💰'),
    (10, 'Uncategorized', 'predefined', '❓');

-- Seed data: Category rules for automatic categorization
INSERT OR IGNORE INTO category_rules (pattern, category_id, priority) VALUES
    -- Groceries
    ('walmart', 1, 10),
    ('target', 1, 10),
    ('whole foods', 1, 10),
    ('trader joe', 1, 10),
    ('safeway', 1, 10),
    ('kroger', 1, 10),
    ('aldi', 1, 10),
    ('costco', 1, 10),

    -- Dining
    ('starbucks', 2, 10),
    ('mcdonalds', 2, 10),
    ('chipotle', 2, 10),
    ('subway', 2, 10),
    ('dunkin', 2, 10),
    ('pizza', 2, 10),
    ('restaurant', 2, 10),
    ('cafe', 2, 10),
    ('coffee', 2, 10),

    -- Transportation
    ('uber', 3, 10),
    ('lyft', 3, 10),
    ('shell', 3, 10),
    ('chevron', 3, 10),
    ('exxon', 3, 10),
    ('gas station', 3, 10),
    ('parking', 3, 10),
    ('metro', 3, 10),
    ('transit', 3, 10),

    -- Entertainment
    ('netflix', 4, 10),
    ('spotify', 4, 10),
    ('amazon prime', 4, 10),
    ('hulu', 4, 10),
    ('disney', 4, 10),
    ('movie', 4, 10),
    ('theater', 4, 10),
    ('cinema', 4, 10),

    -- Utilities
    ('electric', 5, 10),
    ('water', 5, 10),
    ('gas company', 5, 10),
    ('internet', 5, 10),
    ('phone', 5, 10),
    ('verizon', 5, 10),
    ('at&t', 5, 10),
    ('t-mobile', 5, 10),

    -- Healthcare
    ('pharmacy', 6, 10),
    ('cvs', 6, 10),
    ('walgreens', 6, 10),
    ('doctor', 6, 10),
    ('hospital', 6, 10),
    ('clinic', 6, 10),
    ('medical', 6, 10),

    -- Shopping
    ('amazon', 7, 10),
    ('ebay', 7, 10),
    ('best buy', 7, 10),
    ('home depot', 7, 10),
    ('lowes', 7, 10),
    ('macy', 7, 10),

    -- Travel
    ('airline', 8, 10),
    ('hotel', 8, 10),
    ('airbnb', 8, 10),
    ('booking', 8, 10),
    ('expedia', 8, 10);
