// Error handling utilities for sanitizing error messages
// Week 2: Error message sanitization
// Week 3: Domain-specific error types with thiserror

use std::fmt::Display;
use thiserror::Error;

/// Sanitizes a database error by logging it internally and returning a generic message
pub fn sanitize_db_error<E: Display>(error: E, operation: &str) -> String {
    // Log the detailed error internally for debugging with structured logging
    tracing::error!(
        operation = operation,
        error = %error,
        "Database error occurred"
    );

    // Return a safe, generic message to the user
    format!("Failed to {}", operation)
}

/// Sanitizes a general error by logging it internally and returning a generic message
pub fn sanitize_error<E: Display>(error: E, context: &str, user_message: &str) -> String {
    // Log the detailed error internally for debugging with structured logging
    tracing::error!(
        context = context,
        error = %error,
        user_message = user_message,
        "Error occurred"
    );

    // Return a safe, generic message to the user
    user_message.to_string()
}

// ===== Domain-specific error types =====

/// Errors related to debt operations
#[derive(Debug, Error)]
pub enum DebtError {
    #[error("Balance must be non-negative, got {0}")]
    InvalidBalance(f64),

    #[error("Minimum payment must be non-negative, got {0}")]
    InvalidMinPayment(f64),

    #[error("Interest rate must be between {min} and {max}, got {actual}")]
    InvalidInterestRate { min: f64, max: f64, actual: f64 },

    #[error("Debt not found with ID {0}")]
    NotFound(i64),

    #[error("Insufficient funds: monthly amount ${monthly:.2} is less than total minimum payments ${min_payments:.2}")]
    InsufficientFunds {
        monthly: f64,
        min_payments: f64,
    },

    #[error("No debts available for calculation")]
    NoDebts,

    #[error("Payoff calculation exceeded {0} years - check debt parameters")]
    PayoffExceeded(i32),

    #[error("Invalid strategy '{0}': must be 'avalanche' or 'snowball'")]
    InvalidStrategy(String),

    #[error("Payment ${payment:.2} exceeds debt balance ${balance:.2}")]
    PaymentExceedsBalance { payment: f64, balance: f64 },

    #[error("Debt plan not found with ID {0}")]
    PlanNotFound(i64),

    #[error("Payment amount must be positive, got {0}")]
    InvalidPaymentAmount(f64),

    #[error("Database error: {0}")]
    Database(String),
}

impl DebtError {
    /// Convert to user-friendly error message (sanitized)
    pub fn to_user_message(&self) -> String {
        match self {
            // These errors are already safe to show to users
            DebtError::InvalidBalance(_) => self.to_string(),
            DebtError::InvalidMinPayment(_) => self.to_string(),
            DebtError::InvalidInterestRate { .. } => self.to_string(),
            DebtError::NotFound(_) => self.to_string(),
            DebtError::InsufficientFunds { .. } => self.to_string(),
            DebtError::NoDebts => self.to_string(),
            DebtError::PayoffExceeded(_) => self.to_string(),
            DebtError::InvalidStrategy(_) => self.to_string(),
            DebtError::PaymentExceedsBalance { .. } => self.to_string(),
            DebtError::PlanNotFound(_) => self.to_string(),
            DebtError::InvalidPaymentAmount(_) => self.to_string(),

            // Database errors should be sanitized
            DebtError::Database(e) => {
                tracing::error!(error = %e, "Database error in debt operation");
                "Failed to complete debt operation".to_string()
            }
        }
    }
}

/// Errors related to transaction operations
#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Transaction not found with ID {0}")]
    NotFound(i64),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Category not found with ID {0}")]
    CategoryNotFound(i64),

    #[error("Account not found with ID {0}")]
    AccountNotFound(i64),

    #[error("Failed to categorize transaction")]
    CategorizationError,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl TransactionError {
    /// Convert to user-friendly error message (sanitized)
    pub fn to_user_message(&self) -> String {
        match self {
            // These errors are safe to show
            TransactionError::NotFound(_) => self.to_string(),
            TransactionError::InvalidAmount(_) => self.to_string(),
            TransactionError::InvalidDate(_) => self.to_string(),
            TransactionError::CategoryNotFound(_) => self.to_string(),
            TransactionError::AccountNotFound(_) => self.to_string(),
            TransactionError::CategorizationError => self.to_string(),
            TransactionError::ValidationError(_) => self.to_string(),

            // Database errors should be sanitized
            TransactionError::Database(e) => {
                tracing::error!(error = %e, "Database error in transaction operation");
                "Failed to complete transaction operation".to_string()
            }
        }
    }
}

/// Errors related to CSV import operations
#[derive(Debug, Error)]
pub enum CsvImportError {
    #[error("File too large: {size} bytes (max {max} bytes)")]
    FileTooLarge { size: usize, max: usize },

    #[error("Too many rows: {count} rows (max {max} rows)")]
    TooManyRows { count: usize, max: usize },

    #[error("Rate limit exceeded. Please wait {0:.1} seconds before trying again")]
    RateLimitExceeded(f64),

    #[error("Invalid CSV format: {0}")]
    InvalidFormat(String),

    #[error("Missing required column: {0}")]
    MissingColumn(String),

    #[error("Column mapping '{0}' already exists")]
    DuplicateMapping(String),

    #[error("Failed to parse CSV: {0}")]
    ParseError(String),

    #[error("Categorization error: {0}")]
    CategorizationError(String),

    #[error("Duplicate detection error: {0}")]
    DuplicateDetectionError(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl CsvImportError {
    /// Convert to user-friendly error message (sanitized)
    pub fn to_user_message(&self) -> String {
        match self {
            // These errors are safe and informative
            CsvImportError::FileTooLarge { size: _, max } => {
                format!("File too large. Maximum size is {} MB.", max / crate::constants::BYTES_PER_MB)
            }
            CsvImportError::TooManyRows { count, max } => {
                format!("Too many rows. Maximum is {} rows, found approximately {}.", max, count)
            }
            CsvImportError::RateLimitExceeded(secs) => {
                format!("Rate limit exceeded. Please wait {:.1} seconds before trying again.", secs)
            }
            CsvImportError::InvalidFormat(_) => "Failed to parse CSV file. Please check the file format.".to_string(),
            CsvImportError::MissingColumn(col) => format!("Missing required column: {}", col),
            CsvImportError::DuplicateMapping(name) => format!("A mapping with the name '{}' already exists", name),
            CsvImportError::ParseError(_) => "Failed to parse CSV file. Please check the file format.".to_string(),

            // Internal errors should be sanitized
            CsvImportError::CategorizationError(e) => {
                tracing::error!(error = %e, "Categorization error during CSV import");
                "Failed to categorize transactions".to_string()
            }
            CsvImportError::DuplicateDetectionError(e) => {
                tracing::error!(error = %e, "Duplicate detection error during CSV import");
                "Failed to detect duplicates".to_string()
            }
            CsvImportError::Database(e) => {
                tracing::error!(error = %e, "Database error during CSV import");
                "Failed to import CSV file. Please check the file format.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_db_error() {
        let result = sanitize_db_error("connection refused", "load data");
        assert_eq!(result, "Failed to load data");
        // In real usage, eprintln would log "Database error during load data: connection refused"
    }

    #[test]
    fn test_sanitize_error() {
        let result = sanitize_error(
            "file not found",
            "File operation error",
            "Unable to access file"
        );
        assert_eq!(result, "Unable to access file");
        // In real usage, eprintln would log "File operation error: file not found"
    }
}
