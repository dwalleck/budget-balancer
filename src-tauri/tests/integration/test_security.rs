// Security tests for input validation, rate limiting, and SQL injection protection

use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::commands::csv_commands::{get_csv_headers, import_csv_impl, reset_rate_limiter};
use budget_balancer_lib::commands::transaction_commands::{
    list_transactions_impl, search_transactions_impl, TransactionFilter,
};
use budget_balancer_lib::constants::BYTES_PER_MB;
use budget_balancer_lib::models::account::NewAccount;
use budget_balancer_lib::services::csv_parser::ColumnMapping;
use serial_test::serial;

// ==== CSV File Size Validation Tests ====

#[tokio::test]
async fn test_csv_file_size_limit_enforced() {
    // Generate a CSV larger than 10MB
    let huge_csv = "a".repeat(11 * BYTES_PER_MB); // 11MB

    let result = get_csv_headers(huge_csv).await;

    assert!(result.is_err(), "Should reject file larger than 10MB");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("too large") || error_msg.contains("Too large"),
        "Error should mention file size"
    );
}

#[tokio::test]
#[serial]
async fn test_csv_file_size_just_under_limit() {
    reset_rate_limiter();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Ensure reset takes effect
    let db = super::get_test_db_pool().await;

    // Create test account
    let account = NewAccount {
        name: super::unique_name("Size Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.unwrap();

    // Generate a CSV with 1000 rows (well under 10,000 row limit)
    // and moderate file size (under 10MB limit)
    let mut csv = "Date,Amount,Description\n".to_string();
    for i in 0..1000 {
        csv.push_str(&format!("2024-01-01,-{}.00,Test transaction {}\n", i % 100, i));
    }

    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: None,
    };

    let result = import_csv_impl(db, account_id, csv, mapping).await;

    // Should succeed (file is well under 10MB limit)
    assert!(result.is_ok(), "Should successfully process file under size limit: {:?}", result.err());
}

// ==== CSV Row Count Validation Tests ====

#[tokio::test]
#[serial]
async fn test_csv_row_count_limit_enforced() {
    reset_rate_limiter();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Ensure reset takes effect
    let db = super::get_test_db_pool().await;

    // Create test account
    let account = NewAccount {
        name: super::unique_name("Row Count Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.unwrap();

    // Generate CSV with more than 10,000 rows
    let mut huge_csv = "Date,Amount,Description\n".to_string();
    for i in 0..11_000 {
        huge_csv.push_str(&format!("2024-01-01,-{}.00,Test transaction {}\n", i % 100, i));
    }

    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: None,
    };

    let result = import_csv_impl(db, account_id, huge_csv, mapping).await;

    assert!(result.is_err(), "Should reject CSV with more than 10,000 rows");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Too many rows") || error_msg.contains("too many"),
        "Error should mention row limit, got: {}",
        error_msg
    );
}

// ==== Rate Limiting Tests ====

#[tokio::test]
#[serial]
async fn test_csv_import_rate_limiting() {
    // Note: Tests use a 50ms rate limit for fast execution (set via cfg(test))
    // This test verifies rate limiting works, even with the shorter interval
    reset_rate_limiter();
    tokio::time::sleep(tokio::time::Duration::from_millis(60)).await; // Ensure reset takes effect

    let db = super::get_test_db_pool().await;

    // Create test account
    let account = NewAccount {
        name: super::unique_name("Rate Limit Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.unwrap();

    let csv_content = "Date,Amount,Description\n2024-01-01,-50.00,Test";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: None,
    };

    // First import should succeed
    let result1 = import_csv_impl(db, account_id, csv_content.to_string(), mapping.clone()).await;
    assert!(result1.is_ok(), "First import should succeed");

    // Immediate second import should be rate limited (within 50ms window)
    let result2 = import_csv_impl(db, account_id, csv_content.to_string(), mapping.clone()).await;
    assert!(result2.is_err(), "Second import should be rate limited");

    let error = result2.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Rate limit") || error_msg.contains("wait"),
        "Rate limit error should mention rate limiting, got: {}",
        error_msg
    );

    // After waiting for the rate limit period, import should succeed again
    tokio::time::sleep(tokio::time::Duration::from_millis(60)).await;
    // Use different content to avoid duplicate detection
    let csv_content3 = "Date,Amount,Description\n2024-01-02,-60.00,Test3";
    let result3 = import_csv_impl(db, account_id, csv_content3.to_string(), mapping).await;
    assert!(
        result3.is_ok(),
        "Third import after waiting should succeed, got error: {:?}",
        result3.err()
    );
}

// ==== SQL Injection Protection Tests ====

#[tokio::test]
async fn test_sql_injection_in_account_filter() {
    let db = super::get_test_db_pool().await;

    // Attempt SQL injection via account_id filter
    // If vulnerable, this could try to return all transactions or drop tables
    let malicious_input = TransactionFilter {
        account_id: None,
        category_id: None,
        search: None,
        start_date: Some("2024-01-01' OR '1'='1".to_string()), // SQL injection attempt
        end_date: None,
        limit: Some(10),
        offset: Some(0),
    };

    let result = list_transactions_impl(db, Some(malicious_input)).await;

    // Should handle safely - either no results or error, but should not execute injection
    assert!(
        result.is_ok(),
        "Parameterized queries should handle injection attempts safely"
    );

    // Verify database integrity - transactions table should still exist
    let count_result: Result<(i64,), _> =
        sqlx::query_as("SELECT COUNT(*) FROM transactions")
            .fetch_one(db)
            .await;

    assert!(
        count_result.is_ok(),
        "Database should not be corrupted by injection attempt"
    );
}

#[tokio::test]
async fn test_sql_injection_attempts_various_inputs() {
    let db = super::get_test_db_pool().await;

    let malicious_inputs = vec![
        "1 OR 1=1",
        "1'; DROP TABLE transactions;--",
        "' OR ''='",
        "1 UNION SELECT * FROM sqlite_master--",
        "'; DELETE FROM transactions WHERE '1'='1",
        "1' AND '1'='1",
    ];

    for input in malicious_inputs {
        let filter = TransactionFilter {
            account_id: None,
            category_id: None,
        search: None,
            start_date: Some(input.to_string()),
            end_date: None,
            limit: Some(10),
            offset: Some(0),
        };

        let result = list_transactions_impl(db, Some(filter)).await;

        // Should handle safely without executing injection
        // Result can be Ok (no matches) or Err (invalid format), both are fine
        // What matters is that the database isn't compromised

        // Verify transactions table still exists after each attempt
        let integrity_check: Result<(i64,), _> =
            sqlx::query_as("SELECT COUNT(*) FROM transactions")
                .fetch_one(db)
                .await;

        assert!(
            integrity_check.is_ok(),
            "Database integrity compromised by input: {}",
            input
        );
    }
}

#[tokio::test]
#[serial]
async fn test_sql_injection_in_search_query() {
    reset_rate_limiter();
    let db = super::get_test_db_pool().await;

    // Create test account and transaction
    let account = NewAccount {
        name: super::unique_name("SQL Injection Search Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.unwrap();

    let csv_content = "Date,Amount,Description\n2025-01-01,-50.00,Normal Transaction";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: None,
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .unwrap();

    // Attempt SQL injection via search query
    let malicious_queries = vec![
        "' OR '1'='1",
        "'; DROP TABLE transactions;--",
        "' UNION SELECT * FROM sqlite_master--",
        "1' AND '1'='1",
        "%' OR 1=1--",
        "' OR 'x'='x",
    ];

    for malicious_query in malicious_queries {
        let result = search_transactions_impl(db, malicious_query.to_string(), None).await;

        // Should handle safely - parameterized LIKE queries should treat special chars as literals
        assert!(
            result.is_ok(),
            "Parameterized LIKE query should handle injection attempt safely: {}",
            malicious_query
        );

        // Verify database integrity after each attempt
        let integrity_check: Result<(i64,), _> =
            sqlx::query_as("SELECT COUNT(*) FROM transactions")
                .fetch_one(db)
                .await;

        assert!(
            integrity_check.is_ok(),
            "Database integrity compromised by search query: {}",
            malicious_query
        );
    }

    // Verify the transactions table structure is intact
    let table_check: Result<Vec<(String,)>, _> =
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name='transactions'")
            .fetch_all(db)
            .await;

    assert!(
        table_check.is_ok() && !table_check.unwrap().is_empty(),
        "Transactions table should still exist after injection attempts"
    );
}

// ==== Error Message Security Tests ====

#[tokio::test]
async fn test_errors_dont_expose_database_paths() {
    let db = super::get_test_db_pool().await;

    // Trigger various errors and check messages don't expose internals
    let filter = TransactionFilter {
        account_id: Some(999999),
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: Some(10),
        offset: Some(0),
    };

    let result = list_transactions_impl(db, Some(filter)).await;

    if result.is_err() {
        let error = result.unwrap_err();
    let error_msg = error.to_string();

        // Should NOT contain sensitive information
        assert!(
            !error_msg.contains("/home/"),
            "Error should not expose file paths"
        );
        assert!(
            !error_msg.contains(".db"),
            "Error should not expose database files"
        );
        assert!(
            !error_msg.contains("sqlite"),
            "Error should not expose database type"
        );
        assert!(
            !error_msg.contains("panic"),
            "Error should not expose panic details"
        );
        assert!(
            !error_msg.contains("unwrap"),
            "Error should not expose internal calls"
        );
    }
}

#[tokio::test]
#[serial]
async fn test_csv_error_messages_are_safe() {
    reset_rate_limiter();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Ensure reset takes effect
    let db = super::get_test_db_pool().await;

    let account = NewAccount {
        name: super::unique_name("Error Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.unwrap();

    // Invalid CSV content
    let invalid_csv = "Not a valid CSV at all!@#$%";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: None,
    };

    let result = import_csv_impl(db, account_id, invalid_csv.to_string(), mapping).await;

    if result.is_err() {
        let error = result.unwrap_err();
        let error_msg = error.to_string();

        // Should be a generic, user-friendly message
        assert!(
            error_msg.contains("Failed") || error_msg.contains("format") || error_msg.contains("check") ||
            error_msg.contains("parse") || error_msg.contains("Error") || error_msg.contains("Missing"),
            "Error should be user-friendly, got: {}", error_msg
        );

        // Should NOT expose internals
        assert!(!error_msg.contains("src/"), "Should not expose source paths");
        assert!(!error_msg.contains("panic"), "Should not expose panic info");
        assert!(!error_msg.contains("unwrap"), "Should not expose internal details");
    }
}

// ==== Input Validation Tests ====

#[tokio::test]
async fn test_page_size_limit_enforced() {
    let db = super::get_test_db_pool().await;

    // Request more than max page size (100)
    let filter = TransactionFilter {
        account_id: None,
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: Some(1000), // Way over limit
        offset: Some(0),
    };

    let result = list_transactions_impl(db, Some(filter)).await;

    assert!(result.is_ok(), "Should clamp to max page size, not error");

    // Result should have at most 100 items (MAX_PAGE_SIZE)
    let transactions = result.unwrap();
    assert!(
        transactions.len() <= 100,
        "Should enforce max page size of 100"
    );
}

// Additional error sanitization tests added in Week 3
#[tokio::test]
async fn test_debt_error_messages_sanitized() {
    use budget_balancer_lib::commands::debt_commands::update_debt_impl;

    let db = super::get_test_db_pool().await;

    let result = update_debt_impl(db, 99999, Some(-100.0), None, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Should not contain internal details
    assert!(!error_msg.contains("src/"));
    assert!(!error_msg.contains("unwrap"));
}

#[tokio::test]
#[serial]
async fn test_csv_error_user_friendly() {
    reset_rate_limiter();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let db = super::get_test_db_pool().await;

    let huge_file = "x".repeat(11 * BYTES_PER_MB);
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    let result = import_csv_impl(db, 1, huge_file, mapping).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = error.to_user_message();

    assert!(error_msg.contains("large") || error_msg.contains("size"));
}
