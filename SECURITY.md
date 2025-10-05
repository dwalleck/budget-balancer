# Security Guidelines - Budget Balancer

## Overview

This document outlines security best practices and requirements for Budget Balancer development. All contributors must follow these guidelines to ensure the application remains secure.

**Security Principle**: Defense in depth - assume every layer could fail and protect at multiple levels.

---

## SQL Injection Prevention

### ❌ NEVER DO THIS

```rust
// DON'T: Build SQL by string concatenation
let mut query = format!("SELECT * FROM transactions WHERE id = {}", user_input);

// DON'T: Interpolate user input into SQL
let query = format!("SELECT * FROM users WHERE name = '{}'", username);
```

### ✅ ALWAYS DO THIS

```rust
// DO: Use parameterized queries
let transactions = sqlx::query_as::<_, Transaction>(
    "SELECT * FROM transactions WHERE id = ?"
)
.bind(transaction_id)  // Parameters are properly escaped
.fetch_all(db)
.await?;

// DO: Use SQLx query builder
let transactions = sqlx::query_as::<_, Transaction>(
    "SELECT * FROM transactions WHERE category_id = ? AND date >= ?"
)
.bind(category_id)
.bind(start_date)
.fetch_all(db)
.await?;
```

### Guidelines

1. **ALWAYS use parameterized queries** - Never build SQL strings with user input
2. **Use SQLx's compile-time query checking** - `sqlx::query!` macro catches SQL errors at compile time
3. **Validate input before database** - Don't rely only on database constraints
4. **Audit all dynamic SQL** - Any query built at runtime needs extra scrutiny

### Common Patterns to Watch

- ❌ String concatenation: `query + user_input`
- ❌ Format macros with user input: `format!("... {}", user_data)`
- ❌ Push strings: `query.push_str(&user_input)`
- ✅ Bind parameters: `.bind(user_input)`
- ✅ Query macros: `sqlx::query!` and `sqlx::query_as!`

---

## Input Validation

### File Uploads (CSV Import)

#### Size Limits
```rust
const MAX_CSV_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_CSV_ROWS: usize = 10_000;

// Validate file size before processing
if file_size > MAX_CSV_FILE_SIZE {
    return Err("File too large. Maximum size is 10MB".to_string());
}

// Validate row count during parsing
if row_count > MAX_CSV_ROWS {
    return Err("Too many rows. Maximum is 10,000 rows".to_string());
}
```

#### Rate Limiting
```rust
// Simple time-based throttle
const MIN_IMPORT_INTERVAL_MS: u64 = 2000; // 2 seconds between imports

// Track last import time in application state
if elapsed_since_last_import() < Duration::from_millis(MIN_IMPORT_INTERVAL_MS) {
    return Err("Please wait before importing another file".to_string());
}
```

#### CSV Validation
```rust
// Validate data types
fn validate_amount(amount_str: &str) -> Result<f64, String> {
    amount_str.parse::<f64>()
        .map_err(|_| "Invalid amount format".to_string())
        .and_then(|amt| {
            if amt.is_finite() && amt.abs() < 1_000_000_000.0 {
                Ok(amt)
            } else {
                Err("Amount out of reasonable range".to_string())
            }
        })
}

// Validate dates
fn validate_date(date_str: &str) -> Result<String, String> {
    // Must be ISO 8601: YYYY-MM-DD
    if !date_str.matches('-').count() == 2 {
        return Err("Invalid date format".to_string());
    }
    // Additional validation logic...
    Ok(date_str.to_string())
}

// Validate string lengths
fn validate_description(desc: &str) -> Result<String, String> {
    const MAX_DESCRIPTION_LENGTH: usize = 500;
    if desc.len() > MAX_DESCRIPTION_LENGTH {
        Err("Description too long".to_string())
    } else {
        Ok(desc.to_string())
    }
}
```

### User Input Validation

```rust
// Validate numeric ranges
fn validate_interest_rate(rate: f64) -> Result<f64, String> {
    if rate < 0.0 || rate > 100.0 {
        return Err("Interest rate must be between 0% and 100%".to_string());
    }
    Ok(rate)
}

// Validate positive amounts
fn validate_positive_amount(amount: f64) -> Result<f64, String> {
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    Ok(amount)
}

// Sanitize text input
fn sanitize_text(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".-,()&".contains(*c))
        .collect()
}
```

---

## Error Message Sanitization

### ❌ NEVER EXPOSE

```rust
// DON'T: Expose database errors to users
.map_err(|e| e.to_string())  // Shows "no such table" or file paths

// DON'T: Expose file paths
Err(format!("Failed to read {}", db_path.display()))

// DON'T: Expose stack traces
Err(format!("Error: {:#?}", error))
```

### ✅ SAFE ERROR MESSAGES

```rust
// DO: Use generic user-facing messages
.map_err(|e| {
    eprintln!("Database error: {}", e);  // Log detailed error
    "Failed to load transactions".to_string()  // Return generic message
})

// DO: Create custom error types
#[derive(Debug)]
pub enum AppError {
    DatabaseError,
    InvalidInput(String),
    NotFound,
    PermissionDenied,
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            Self::DatabaseError => "A database error occurred".to_string(),
            Self::InvalidInput(msg) => msg.clone(),  // Only if msg is safe
            Self::NotFound => "The requested item was not found".to_string(),
            Self::PermissionDenied => "Permission denied".to_string(),
        }
    }
}
```

### Error Handling Best Practices

1. **Log detailed errors internally** - Use `eprintln!`, `log::error!`, or tracing
2. **Return generic messages to users** - Don't expose implementation details
3. **Never show**:
   - File system paths
   - Database connection strings
   - SQL query details
   - Stack traces
   - Internal function names
4. **Safe to show**:
   - Validation errors ("Invalid email format")
   - User-friendly status ("Transaction created successfully")
   - Generic failures ("An error occurred. Please try again")

---

## Database Security

### Path Validation

```rust
use std::path::PathBuf;

fn validate_database_path(path: &PathBuf) -> Result<PathBuf, String> {
    // Canonicalize to prevent directory traversal
    let canonical = path.canonicalize()
        .map_err(|_| "Invalid database path".to_string())?;

    // Ensure it's within expected directory
    let data_dir = dirs::data_dir()
        .ok_or("Could not find data directory")?;

    if !canonical.starts_with(&data_dir) {
        return Err("Database path outside allowed directory".to_string());
    }

    Ok(canonical)
}
```

### Connection Pool Security

```rust
// DO: Use connection pooling (already implemented)
use sqlx::SqlitePool;

// DO: Set appropriate pool limits
SqlitePoolOptions::new()
    .max_connections(5)  // Limit concurrent connections
    .connect_with(options)
    .await?
```

### Migration Safety

```rust
// DO: Use SQLx migrations
sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .map_err(|e| {
        eprintln!("Migration failed: {}", e);
        "Database initialization failed".to_string()
    })?;

// DON'T: Run raw SQL from untrusted sources
```

---

## Rate Limiting

### Implementation Approach

```rust
use std::time::{Duration, Instant};
use std::sync::Mutex;

pub struct RateLimiter {
    last_request: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last_request: Mutex::new(Instant::now() - Duration::from_secs(100)),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    pub fn check_and_update(&self) -> Result<(), String> {
        let mut last = self.last_request.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(*last) < self.min_interval {
            return Err("Rate limit exceeded. Please wait.".to_string());
        }

        *last = now;
        Ok(())
    }
}

// Usage in command
static CSV_RATE_LIMITER: Lazy<RateLimiter> = Lazy::new(|| RateLimiter::new(2000));

#[tauri::command]
pub async fn import_csv(...) -> Result<(), String> {
    CSV_RATE_LIMITER.check_and_update()?;
    // ... rest of import logic
}
```

---

## Testing Security

### SQL Injection Tests

```rust
#[tokio::test]
async fn test_sql_injection_attempt() {
    let db = get_test_db_pool().await;

    // Attempt SQL injection in filter
    let malicious_input = "1 OR 1=1; DROP TABLE transactions;--";

    let filter = TransactionFilter {
        account_id: None,
        category_id: None,
        description: Some(malicious_input.to_string()),
        ..Default::default()
    };

    let result = list_transactions_impl(db, Some(filter)).await;

    // Should handle safely without executing injection
    assert!(result.is_ok());

    // Verify transactions table still exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transactions")
        .fetch_one(db)
        .await
        .expect("Table should still exist");
}
```

### Input Validation Tests

```rust
#[tokio::test]
async fn test_csv_size_limit() {
    // Generate CSV larger than limit
    let large_csv = generate_csv_with_rows(20_000);  // Over 10k limit

    let result = import_csv_impl(db, account_id, large_csv, mapping).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Too many rows"));
}

#[tokio::test]
async fn test_invalid_amount_rejected() {
    let result = validate_amount("not_a_number");
    assert!(result.is_err());

    let result = validate_amount("999999999999");  // Out of range
    assert!(result.is_err());
}
```

---

## Security Checklist

Before merging any PR, verify:

### Code Review
- [ ] No SQL string concatenation with user input
- [ ] All user input validated before use
- [ ] Error messages don't expose internal details
- [ ] File uploads have size limits
- [ ] Rate limiting on expensive operations
- [ ] No hard-coded secrets or credentials
- [ ] Database paths validated

### Testing
- [ ] Security tests added for new input points
- [ ] SQL injection tests for new queries
- [ ] Input validation tests for new fields
- [ ] Error message tests (don't expose internals)

### Documentation
- [ ] Security implications documented
- [ ] Input validation rules documented
- [ ] Rate limits documented

---

## Reporting Security Issues

If you discover a security vulnerability:

1. **DO NOT** open a public GitHub issue
2. **DO** email the maintainers directly
3. **DO** provide details on reproduction
4. **DO** suggest a fix if possible

---

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [SQLx Security](https://github.com/launchbadge/sqlx#security)
- [Tauri Security](https://tauri.app/v1/guides/security/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)

---

**Last Updated**: 2025-10-04
**Related Documents**: PR-REVIEW-RESPONSE.md, TESTING.md, CLAUDE.md
