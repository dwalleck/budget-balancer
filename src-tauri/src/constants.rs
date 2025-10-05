// Application-wide constants
// This module centralizes all magic numbers and configuration values

// ===== Database Constants =====

/// Default category ID for uncategorized transactions
pub const DEFAULT_CATEGORY_ID: i64 = 10;

// ===== CSV Import Limits =====

/// Bytes per megabyte constant for file size calculations
pub const BYTES_PER_MB: usize = 1024 * 1024;

/// Maximum CSV file size in bytes (10 MB)
pub const MAX_CSV_FILE_SIZE: usize = 10 * BYTES_PER_MB;

/// Maximum number of rows allowed in a CSV import
pub const MAX_CSV_ROWS: usize = 10_000;

/// Minimum interval between CSV imports in milliseconds (2 seconds)
pub const MIN_CSV_IMPORT_INTERVAL_MS: u64 = 2000;

// ===== Validation Limits =====

/// Minimum valid interest rate percentage
pub const MIN_INTEREST_RATE: f64 = 0.0;

/// Maximum valid interest rate percentage
pub const MAX_INTEREST_RATE: f64 = 100.0;

/// Maximum reasonable transaction amount (1 billion)
pub const MAX_TRANSACTION_AMOUNT: f64 = 1_000_000_000.0;

/// Maximum description length
pub const MAX_DESCRIPTION_LENGTH: usize = 500;

/// Maximum merchant name length
pub const MAX_MERCHANT_LENGTH: usize = 200;

// ===== Pagination Defaults =====

/// Default number of items per page
pub const DEFAULT_PAGE_SIZE: i64 = 50;

/// Maximum number of items per page
pub const MAX_PAGE_SIZE: i64 = 100;

/// Default offset for pagination
pub const DEFAULT_OFFSET: i64 = 0;

// ===== Database Connection Pool =====

/// Maximum number of concurrent database connections
pub const MAX_DB_CONNECTIONS: u32 = 5;

// ===== Financial Calculation Constants =====

/// Number of months in a year (for interest rate calculations)
pub const MONTHS_PER_YEAR: f64 = 12.0;

/// Divisor to convert percentage to decimal (e.g., 18% -> 0.18)
pub const PERCENT_TO_DECIMAL_DIVISOR: f64 = 100.0;

/// Maximum years allowed for debt payoff calculations
pub const MAX_PAYOFF_YEARS: i32 = 100;

// ===== Spending Tracker Thresholds =====

/// Percentage threshold for "under budget" status
pub const SPENDING_UNDER_THRESHOLD_PERCENT: f64 = 80.0;

/// Percentage threshold for "on track" status (at or below target)
pub const SPENDING_ON_TRACK_THRESHOLD_PERCENT: f64 = 100.0;
