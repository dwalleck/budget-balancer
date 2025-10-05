# Data Model
**Feature**: Budget Balancer - Debt Management & Spending Insights
**Date**: 2025-10-04

## Overview
SQLite database schema for local storage of financial data. Optimized for read-heavy analytics queries while maintaining data integrity through normalization.

## Entity Relationship Diagram

```
┌─────────────┐       ┌──────────────┐       ┌─────────────────┐
│  accounts   │       │ transactions │       │   categories    │
├─────────────┤       ├──────────────┤       ├─────────────────┤
│ id (PK)     │───┐   │ id (PK)      │   ┌───│ id (PK)         │
│ name        │   └──>│ account_id   │   │   │ name            │
│ type        │       │ category_id  │<──┘   │ type            │
│ created_at  │       │ date         │       │ parent_id (FK)  │
└─────────────┘       │ amount       │       │ created_at      │
                      │ description  │       └─────────────────┘
                      │ merchant     │                │
                      │ hash         │                │
                      │ created_at   │                │
                      └──────────────┘                │
                                                      │
┌─────────────────┐                                   │
│ category_rules  │                                   │
├─────────────────┤                                   │
│ id (PK)         │                                   │
│ pattern         │                                   │
│ category_id (FK)│<──────────────────────────────────┘
│ priority        │
│ created_at      │
└─────────────────┘

┌──────────────┐       ┌───────────────┐       ┌──────────────┐
│    debts     │       │ debt_payments │       │  debt_plans  │
├──────────────┤       ├───────────────┤       ├──────────────┤
│ id (PK)      │───┐   │ id (PK)       │   ┌───│ id (PK)      │
│ name         │   └──>│ debt_id (FK)  │   │   │ strategy     │
│ balance      │       │ amount        │   │   │ monthly_amt  │
│ original_bal │       │ date          │   │   │ created_at   │
│ interest_rate│       │ plan_id (FK)  │<──┘   │ updated_at   │
│ min_payment  │       │ created_at    │       └──────────────┘
│ created_at   │       └───────────────┘
│ updated_at   │
└──────────────┘

┌──────────────────┐
│ spending_targets │
├──────────────────┤
│ id (PK)          │
│ category_id (FK) │───┐
│ amount           │   │
│ period           │   │
│ start_date       │   │
│ end_date         │   │
│ created_at       │   │
└──────────────────┘   │
                       │
                  (References categories)

┌─────────────────┐
│ column_mappings │
├─────────────────┤
│ id (PK)         │
│ source_name     │
│ date_col        │
│ amount_col      │
│ desc_col        │
│ merchant_col    │
│ created_at      │
└─────────────────┘
```

## Table Schemas

### transactions
Stores all financial transactions imported from CSV files.

```sql
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    date TEXT NOT NULL,              -- ISO 8601 format: YYYY-MM-DD
    amount REAL NOT NULL,             -- Positive for income, negative for expenses
    description TEXT NOT NULL,
    merchant TEXT,
    hash TEXT NOT NULL UNIQUE,        -- SHA-256 of (date + amount + description)
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT
);

CREATE INDEX idx_transactions_date ON transactions(date DESC);
CREATE INDEX idx_transactions_category_date ON transactions(category_id, date);
CREATE INDEX idx_transactions_account_date ON transactions(account_id, date);
CREATE INDEX idx_transactions_hash ON transactions(hash);
```

**Constraints**:
- `hash` ensures duplicate detection (date + amount + description)
- `amount` precision: REAL (sufficient for currency, typically 2 decimal places)
- `date` stored as TEXT in ISO format for SQLite compatibility and sorting

### categories
Predefined and user-created spending categories.

```sql
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL CHECK(type IN ('predefined', 'custom')),
    parent_id INTEGER,                -- For subcategories (future)
    icon TEXT,                        -- Emoji or icon name
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX idx_categories_type ON categories(type);
```

**Predefined Categories** (inserted on app init):
```sql
INSERT INTO categories (name, type, icon) VALUES
    ('Groceries', 'predefined', '🛒'),
    ('Dining', 'predefined', '🍽️'),
    ('Transportation', 'predefined', '🚗'),
    ('Entertainment', 'predefined', '🎬'),
    ('Utilities', 'predefined', '⚡'),
    ('Healthcare', 'predefined', '🏥'),
    ('Shopping', 'predefined', '🛍️'),
    ('Travel', 'predefined', '✈️'),
    ('Income', 'predefined', '💰'),
    ('Uncategorized', 'predefined', '❓');
```

### category_rules
Merchant keyword patterns for automatic categorization.

```sql
CREATE TABLE category_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL,            -- Lowercase merchant keyword (e.g., "starbucks")
    category_id INTEGER NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,  -- Higher priority matches first
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX idx_category_rules_pattern ON category_rules(pattern);
CREATE INDEX idx_category_rules_priority ON category_rules(priority DESC);
```

**Predefined Rules** (examples, inserted on app init):
```sql
INSERT INTO category_rules (pattern, category_id, priority)
SELECT 'starbucks', id, 10 FROM categories WHERE name = 'Dining';
INSERT INTO category_rules (pattern, category_id, priority)
SELECT 'uber', id, 10 FROM categories WHERE name = 'Transportation';
-- ... more rules
```

**Categorization Logic**:
1. Normalize merchant name to lowercase
2. Query rules ordered by priority DESC
3. First pattern match wins (case-insensitive substring)
4. If no match: Assign to "Uncategorized"

### accounts
Bank accounts and credit cards.

```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL CHECK(type IN ('checking', 'savings', 'credit_card')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### debts
Debt accounts (credit cards, loans, etc.).

```sql
CREATE TABLE debts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    balance REAL NOT NULL CHECK(balance >= 0),
    original_balance REAL NOT NULL CHECK(original_balance >= 0),
    interest_rate REAL NOT NULL CHECK(interest_rate >= 0 AND interest_rate <= 100),  -- Annual percentage
    min_payment REAL NOT NULL CHECK(min_payment >= 0),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_debts_balance ON debts(balance);
CREATE INDEX idx_debts_interest_rate ON debts(interest_rate DESC);
```

### debt_payments
Recorded payments toward debts (from plans or manual entries).

```sql
CREATE TABLE debt_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    debt_id INTEGER NOT NULL,
    amount REAL NOT NULL CHECK(amount > 0),
    date TEXT NOT NULL,               -- ISO 8601 format
    plan_id INTEGER,                  -- NULL for manual payments
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (debt_id) REFERENCES debts(id) ON DELETE CASCADE,
    FOREIGN KEY (plan_id) REFERENCES debt_plans(id) ON DELETE SET NULL
);

CREATE INDEX idx_debt_payments_debt_date ON debt_payments(debt_id, date);
CREATE INDEX idx_debt_payments_plan ON debt_payments(plan_id);
```

### debt_plans
Saved debt payoff strategies.

```sql
CREATE TABLE debt_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    strategy TEXT NOT NULL CHECK(strategy IN ('avalanche', 'snowball')),
    monthly_amount REAL NOT NULL CHECK(monthly_amount > 0),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Note**: Plan calculations are performed on-demand and cached in-memory. The `debt_payments` table links payments to plans for historical tracking.

### spending_targets
Budget targets per category and time period.

```sql
CREATE TABLE spending_targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    amount REAL NOT NULL CHECK(amount > 0),
    period TEXT NOT NULL CHECK(period IN ('monthly', 'quarterly', 'yearly')),
    start_date TEXT NOT NULL,         -- ISO 8601 format
    end_date TEXT,                    -- NULL for recurring targets
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX idx_spending_targets_category ON spending_targets(category_id);
CREATE INDEX idx_spending_targets_dates ON spending_targets(start_date, end_date);
```

### column_mappings
Saved CSV column mappings for repeat imports.

```sql
CREATE TABLE column_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_name TEXT NOT NULL UNIQUE, -- User-defined name (e.g., "Chase Credit Card")
    date_col TEXT NOT NULL,           -- Column name or index for date
    amount_col TEXT NOT NULL,
    description_col TEXT NOT NULL,
    merchant_col TEXT,                -- Optional
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## Data Integrity Rules

### Constraints
1. **Referential Integrity**: All foreign keys use ON DELETE CASCADE or RESTRICT based on data sensitivity
2. **Check Constraints**: Amount fields validate positive values, enums enforce valid states
3. **Unique Constraints**: Prevent duplicate categories, accounts, column mappings
4. **Hash Uniqueness**: Transactions table uses hash to prevent duplicate imports

### Validation Rules (Application Layer)
1. **Date Validation**: Dates must be valid ISO 8601 format, not in future (except for targets)
2. **Amount Validation**: Precision to 2 decimal places, range checks for realistic values
3. **Interest Rate**: 0-100% range, typically 0-35% for credit cards
4. **Category Rules**: Pattern must be non-empty, lowercase, alphanumeric with spaces/hyphens

## Data Lifecycle

### Transaction Import Flow
1. User selects CSV file
2. System checks for saved column mapping
3. If found: Parse using mapping → Compute hashes → Skip duplicates → Insert new transactions
4. If not found: Preview columns → User maps → Save mapping → Proceed with import
5. Auto-categorize using category_rules → Assign to categories

### Debt Calculation Flow
1. User creates debt_plan with strategy and monthly amount
2. System loads all debts from debts table
3. Simulate month-by-month payments (in-memory)
4. Return payment schedule and projections (not persisted until user confirms)
5. On confirm: Insert records into debt_payments

### Data Cleanup
- **Soft Deletes**: Not used (hard deletes acceptable for local personal app)
- **Archival**: User can export data to JSON/CSV before deletion
- **Cascading Deletes**: Removing category removes its rules and updates transactions to "Uncategorized"

## Performance Considerations

### Indexes
- **transactions table**: Indexed on date (DESC for recent-first), category+date, account+date
- **category_rules**: Indexed on pattern for fast lookup during categorization
- **debt_payments**: Indexed on debt_id+date for payment history queries

### Query Optimization
1. **Spending Analysis**: Use indexes on (category_id, date) for fast aggregation
2. **Debt Progress**: Index on (debt_id, date) for payment history
3. **Time Series**: Date indexes support efficient range queries

### Scalability
- Expected data: ~10k transactions/year, ~10 debts, ~20 categories
- SQLite handles this scale efficiently (<50MB database)
- Queries optimized for <100ms response time

## Migration Strategy

### Initial Schema (v1.0)
```sql
-- migrations/001_initial_schema.sql
-- Contains all CREATE TABLE statements above
```

### Future Migrations (examples)
- v1.1: Add `accounts.institution` for bank name
- v1.2: Add `categories.color` for UI customization
- v2.0: Add `budgets` table for comprehensive budget management

**Migration Tool**: Use Rust `rusqlite` migrations or Tauri SQL plugin migration support.
