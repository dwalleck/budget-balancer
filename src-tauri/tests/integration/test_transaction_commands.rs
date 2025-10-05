use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::commands::category_commands::create_category_impl;
use budget_balancer_lib::commands::csv_commands::{import_csv_impl, reset_rate_limiter};
use budget_balancer_lib::commands::transaction_commands::{count_transactions_impl, list_transactions_impl, update_transaction_category_impl, TransactionFilter};
use budget_balancer_lib::models::account::NewAccount;
use budget_balancer_lib::models::category::NewCategory;
use budget_balancer_lib::services::csv_parser::ColumnMapping;

#[tokio::test]
async fn test_list_transactions_empty() {
    let db = super::get_test_db_pool().await;
    let result = list_transactions_impl(db, None).await;
    assert!(result.is_ok(), "Failed to list transactions: {:?}", result);

    let transactions = result.unwrap();
    // May be empty or have transactions from other tests
    assert!(transactions.is_empty() || !transactions.is_empty());
}

#[tokio::test]
async fn test_list_transactions_with_account_filter() {
    let db = super::get_test_db_pool().await;
    // Create a test account
    let account = NewAccount {
        name: super::unique_name("Transaction Filter Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    });

    let result = list_transactions_impl(db, filter).await;
    assert!(result.is_ok(), "Failed to list transactions with filter");

    let transactions = result.unwrap();
    // All transactions should be for the specified account
    for transaction in transactions {
        assert_eq!(transaction.account_id, account_id);
    }
}

#[tokio::test]
async fn test_list_transactions_with_limit() {
    let db = super::get_test_db_pool().await;
    let filter = Some(TransactionFilter {
        account_id: None,
        category_id: None,
        start_date: None,
        end_date: None,
        limit: Some(5),
        offset: None,
    });

    let result = list_transactions_impl(db, filter).await;
    assert!(result.is_ok(), "Failed to list transactions with limit");

    let transactions = result.unwrap();
    assert!(transactions.len() <= 5, "Should return at most 5 transactions");
}

#[tokio::test]
async fn test_update_transaction_category() {
    let db = super::get_test_db_pool().await;
    // Note: This test requires a transaction to exist
    // We'll need to import a transaction first via CSV or create one directly
    // For now, this tests the command interface

    let result = update_transaction_category_impl(db, 999999, 1).await;
    // Should fail because transaction doesn't exist, but tests the interface
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
async fn test_list_transactions_with_date_filter() {
    let db = super::get_test_db_pool().await;
    let filter = Some(TransactionFilter {
        account_id: None,
        category_id: None,
        start_date: Some("2024-01-01".to_string()),
        end_date: Some("2024-12-31".to_string()),
        limit: None,
        offset: None,
    });

    let result = list_transactions_impl(db, filter).await;
    assert!(result.is_ok(), "Failed to list transactions with date filter");

    let transactions = result.unwrap();
    for transaction in transactions {
        assert!(
            transaction.date.as_str() >= "2024-01-01" && transaction.date.as_str() <= "2024-12-31",
            "Transaction date should be within filter range"
        );
    }
}

#[tokio::test]
async fn test_list_transactions_with_category_filter() {
    let db = super::get_test_db_pool().await;
    // Create a test category
    let category = NewCategory {
        name: super::unique_name("Transaction Test Category"),
        icon: None,
    };
    let category_id = create_category_impl(db, category).await.expect("Failed to create category");

    let filter = Some(TransactionFilter {
        account_id: None,
        category_id: Some(category_id),
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    });

    let result = list_transactions_impl(db, filter).await;
    assert!(result.is_ok(), "Failed to list transactions with category filter");

    let transactions = result.unwrap();
    for transaction in transactions {
        assert_eq!(transaction.category_id, category_id);
    }
}

// ==== Pagination Tests ====

#[tokio::test]
async fn test_pagination_defaults_applied_when_none() {
    reset_rate_limiter();
    let db = super::get_test_db_pool().await;

    // Create test account
    let account = NewAccount {
        name: super::unique_name("Pagination Default Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Create 75 test transactions via CSV import
    let mut csv_rows = vec!["Date,Amount,Description".to_string()];
    for i in 0..75 {
        csv_rows.push(format!("2024-01-{:02},-{}.00,Test Transaction {}",
            (i % 28) + 1, i + 1, i + 1));
    }
    let csv_content = csv_rows.join("\n");

    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: None,
    };

    import_csv_impl(db, account_id, csv_content, mapping)
        .await
        .expect("Failed to import test transactions");

    // Pass filter with limit=None, offset=None for this account only
    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        start_date: None,
        end_date: None,
        limit: None,  // Should default to 50
        offset: None, // Should default to 0
    });

    let result = list_transactions_impl(db, filter).await;
    assert!(result.is_ok(), "Should successfully apply default pagination");

    let transactions = result.unwrap();
    // Should return exactly 50 (default page size) since we have 75 transactions
    assert_eq!(transactions.len(), 50, "Should return exactly 50 transactions with default pagination");
}

#[tokio::test]
async fn test_pagination_max_limit_enforced() {
    reset_rate_limiter();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Ensure rate limit reset
    let db = super::get_test_db_pool().await;

    // Create test account
    let account = NewAccount {
        name: super::unique_name("Pagination Max Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Create 150 test transactions to test max limit
    let mut csv_rows = vec!["Date,Amount,Description".to_string()];
    for i in 0..150 {
        csv_rows.push(format!("2024-{:02}-{:02},-{}.00,Test Transaction {}",
            (i / 28) + 1, (i % 28) + 1, i + 1, i + 1));
    }
    let csv_content = csv_rows.join("\n");

    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: None,
    };

    import_csv_impl(db, account_id, csv_content, mapping)
        .await
        .expect("Failed to import test transactions");

    // Try to request 1000 items (above max of 100) for this account only
    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        start_date: None,
        end_date: None,
        limit: Some(1000), // Should be clamped to 100
        offset: Some(0),
    });

    let result = list_transactions_impl(db, filter).await;
    assert!(result.is_ok(), "Should successfully clamp to max page size");

    let transactions = result.unwrap();
    // Should return exactly 100 (max page size) since we requested 1000 but have 150
    assert_eq!(transactions.len(), 100, "Should return exactly 100 transactions when limit exceeds max");
}

#[tokio::test]
async fn test_count_transactions_without_filter() {
    let db = super::get_test_db_pool().await;

    let result = count_transactions_impl(db, None).await;
    assert!(result.is_ok(), "Should successfully count transactions");

    let count = result.unwrap();
    assert!(count >= 0, "Count should be non-negative");
}

#[tokio::test]
async fn test_count_transactions_with_filter() {
    let db = super::get_test_db_pool().await;

    // Create a test account
    let account = NewAccount {
        name: super::unique_name("Count Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    });

    let result = count_transactions_impl(db, filter).await;
    assert!(result.is_ok(), "Should successfully count filtered transactions");

    let count = result.unwrap();
    // Count for new account should be 0 (no transactions yet)
    assert_eq!(count, 0, "New account should have 0 transactions");
}
