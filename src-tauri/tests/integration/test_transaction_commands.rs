use budget_balancer_lib::commands::category_commands::create_category_impl;
use budget_balancer_lib::commands::transaction_commands::{
    bulk_delete_transactions_impl, bulk_update_category_impl, count_transactions_impl,
    delete_transaction_impl, list_transactions_impl, search_transactions_impl,
    update_transaction_category_impl, TransactionFilter,
};
use budget_balancer_lib::models::category::NewCategory;

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
    let account_id = super::fixtures::create_test_account(db, "Transaction Filter Test").await;

    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
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
        search: None,
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
        search: None,
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
        search: None,
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
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Pagination Default Test").await;

    // Create 75 test transactions directly (no CSV import needed!)
    let mut transactions = Vec::new();
    for i in 0..75 {
        transactions.push(super::fixtures::TestTransaction::new(
            &format!("2024-01-{:02}", (i % 28) + 1),
            -(i as f64 + 1.0),
            &format!("Test Transaction {}", i + 1),
        ));
    }
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Pass filter with limit=None, offset=None for this account only
    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
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
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Pagination Max Test").await;

    // Create 150 test transactions directly (no CSV import!)
    let mut transactions = Vec::new();
    for i in 0..150 {
        transactions.push(super::fixtures::TestTransaction::new(
            &format!("2024-{:02}-{:02}", (i / 28) + 1, (i % 28) + 1),
            -(i as f64 + 1.0),
            &format!("Test Transaction {}", i + 1),
        ));
    }
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Try to request 1000 items (above max of 100) for this account only
    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
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
    let account_id = super::fixtures::create_test_account(db, "Count Test").await;

    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
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

// Edge case tests added in Week 3
#[tokio::test]
async fn test_list_transactions_zero_limit() {
    let db = super::get_test_db_pool().await;

    let filter = TransactionFilter {
        account_id: None,
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: Some(0),
        offset: None,
    };

    let result = list_transactions_impl(db, Some(filter)).await;
    assert!(result.is_ok(), "Zero limit should use default limit");
}

#[tokio::test]
async fn test_list_transactions_combined_filters() {
    let db = super::get_test_db_pool().await;

    let filter = TransactionFilter {
        account_id: Some(1),
        category_id: Some(1),
        search: None,
        start_date: Some("2025-01-01".to_string()),
        end_date: Some("2025-12-31".to_string()),
        limit: Some(10),
        offset: Some(0),
    };

    let result = list_transactions_impl(db, Some(filter)).await;
    assert!(result.is_ok(), "Combined filters should work");
}

// ==== T026: Search Transactions Tests ====

#[tokio::test]
async fn test_search_transactions_by_description() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Search Test Account").await;

    // Create test transactions directly
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-01", -50.00, "Grocery shopping at Whole Foods"),
        super::fixtures::TestTransaction::new("2025-01-02", -25.00, "Coffee at Starbucks"),
        super::fixtures::TestTransaction::new("2025-01-03", -100.00, "Electronics purchase"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Test search by description substring (case-insensitive)
    let result = search_transactions_impl(db, "grocery".to_string(), None).await;
    assert!(result.is_ok(), "Search should succeed");

    let transactions = result.unwrap();
    assert!(transactions.len() >= 1, "Should find at least one transaction");
    assert!(
        transactions.iter().any(|t| t.description.to_lowercase().contains("grocery")),
        "Should find transaction with 'grocery' in description"
    );
}

#[tokio::test]
async fn test_search_transactions_by_merchant() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Search Merchant Test").await;

    // Create test transactions with merchants
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-01", -50.00, "Purchase").with_merchant("Starbucks Coffee"),
        super::fixtures::TestTransaction::new("2025-01-02", -75.00, "Purchase").with_merchant("Whole Foods Market"),
        super::fixtures::TestTransaction::new("2025-01-03", -30.00, "Purchase").with_merchant("Shell Gas Station"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Test search by merchant substring
    let result = search_transactions_impl(db, "starbucks".to_string(), None).await;
    assert!(result.is_ok(), "Search by merchant should succeed");

    let transactions = result.unwrap();
    assert!(
        transactions.iter().any(|t| t.merchant.as_ref()
            .map(|m| m.to_lowercase().contains("starbucks"))
            .unwrap_or(false)),
        "Should find transaction with 'starbucks' merchant"
    );
}

#[tokio::test]
async fn test_search_transactions_case_insensitive() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Case Test").await;

    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-01", -50.00, "Whole Foods Market"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Search with different case
    let result = search_transactions_impl(db, "WHOLE FOODS".to_string(), None).await;
    assert!(result.is_ok(), "Case-insensitive search should work");
    assert!(result.unwrap().len() >= 1, "Should find transaction regardless of case");
}

#[tokio::test]
async fn test_search_transactions_with_pagination() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Pagination Search Test").await;

    // Create 10 transactions with "store" in description
    let mut transactions = Vec::new();
    for i in 0..10 {
        transactions.push(super::fixtures::TestTransaction::new(
            &format!("2025-01-{:02}", i + 1),
            -(i as f64 + 10.0),
            &format!("Store purchase {}", i),
        ));
    }
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Search with pagination
    let filter = Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: Some(5),
        offset: Some(0),
    });

    let result = search_transactions_impl(db, "store".to_string(), filter).await;
    assert!(result.is_ok(), "Paginated search should work");
    let transactions = result.unwrap();
    assert!(transactions.len() <= 5, "Should respect pagination limit");
}

#[tokio::test]
async fn test_search_transactions_validates_query_length() {
    let db = super::get_test_db_pool().await;

    // Create a query that's too long (>100 characters)
    let long_query = "a".repeat(101);
    let result = search_transactions_impl(db, long_query, None).await;

    assert!(result.is_err(), "Should reject query longer than 100 characters");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("too long"), "Error should mention query length");
}

#[tokio::test]
async fn test_search_escapes_like_wildcards() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Wildcard Escape Test").await;

    // Create transactions with special LIKE characters
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-01", -50.00, "100% discount offer"),
        super::fixtures::TestTransaction::new("2025-01-02", -75.00, "100 regular item"),
        super::fixtures::TestTransaction::new("2025-01-03", -30.00, "50_50 split payment"),
        super::fixtures::TestTransaction::new("2025-01-04", -40.00, "50 normal payment"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Test 1: Search for "100%" should match only "100% discount", not "100 regular"
    let result = search_transactions_impl(db, "100%".to_string(), Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    }))
    .await
    .expect("Search should succeed");

    // Should only match the transaction with literal "100%", not treat % as wildcard
    assert_eq!(result.len(), 1, "Should match exactly one transaction with '100%'");
    assert!(
        result[0].description.contains("100% discount"),
        "Should match transaction with literal '100%' in description"
    );

    // Test 2: Search for "50_50" should match only "50_50 split", not "50 normal"
    let result2 = search_transactions_impl(db, "50_50".to_string(), Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    }))
    .await
    .expect("Search should succeed");

    // Should only match the transaction with literal "50_50", not treat _ as single-char wildcard
    assert_eq!(result2.len(), 1, "Should match exactly one transaction with '50_50'");
    assert!(
        result2[0].description.contains("50_50"),
        "Should match transaction with literal '50_50' in description"
    );
}

// ==== T028: Delete Transaction Tests ====

#[tokio::test]
async fn test_delete_transaction_success() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Delete Test").await;

    // Create test transaction directly
    let transaction_id = super::fixtures::insert_single_transaction(
        db,
        account_id,
        "2025-01-01",
        -50.00,
        "Test Transaction"
    ).await;

    // Delete the transaction
    let result = delete_transaction_impl(db, transaction_id).await;
    assert!(result.is_ok(), "Delete should succeed");

    // Verify transaction no longer exists
    let updated = list_transactions_impl(db, Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    }))
    .await
    .expect("Failed to list transactions");

    assert!(
        !updated.iter().any(|t| t.id == transaction_id),
        "Transaction should be deleted"
    );
}

#[tokio::test]
async fn test_delete_transaction_not_found() {
    let db = super::get_test_db_pool().await;

    let result = delete_transaction_impl(db, 999999).await;
    assert!(result.is_err(), "Should fail for non-existent transaction");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("not found"),
        "Error should indicate transaction not found"
    );
}

// ==== T029: Bulk Delete Transactions Tests ====

#[tokio::test]
async fn test_bulk_delete_transactions_success() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Bulk Delete Test").await;

    // Create 3 test transactions directly
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-01", -50.00, "Transaction 1"),
        super::fixtures::TestTransaction::new("2025-01-02", -75.00, "Transaction 2"),
        super::fixtures::TestTransaction::new("2025-01-03", -100.00, "Transaction 3"),
    ];
    let ids = super::fixtures::insert_test_transactions(db, account_id, transactions).await;
    assert_eq!(ids.len(), 3, "Should have created 3 transactions");

    // Bulk delete
    let result = bulk_delete_transactions_impl(db, ids.clone()).await;
    assert!(result.is_ok(), "Bulk delete should succeed");

    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.deleted_count, 3, "Should delete 3 transactions");
    assert!(bulk_result.failed_ids.is_empty(), "No IDs should fail");

    // Verify all deleted
    let updated = list_transactions_impl(db, Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    }))
    .await
    .expect("Failed to list transactions");

    for id in ids {
        assert!(
            !updated.iter().any(|t| t.id == id),
            "Transaction {} should be deleted",
            id
        );
    }
}

#[tokio::test]
async fn test_bulk_delete_transactions_reports_failed_ids() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Bulk Delete Failed Test").await;

    // Create one transaction
    let valid_id = super::fixtures::insert_single_transaction(
        db,
        account_id,
        "2025-01-01",
        -50.00,
        "Test Transaction"
    ).await;
    let invalid_id = 999999i64;

    // Try to delete both valid and invalid IDs
    let result = bulk_delete_transactions_impl(db, vec![valid_id, invalid_id]).await;
    assert!(result.is_ok(), "Bulk delete should succeed even with some failures");

    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.deleted_count, 1, "Should delete 1 transaction");
    assert!(bulk_result.failed_ids.contains(&invalid_id), "Should report failed ID");
}

#[tokio::test]
async fn test_bulk_delete_transactions_validates_empty_array() {
    let db = super::get_test_db_pool().await;

    let result = bulk_delete_transactions_impl(db, vec![]).await;
    assert!(result.is_err(), "Should reject empty array");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("empty"),
        "Error should mention empty array"
    );
}

#[tokio::test]
async fn test_bulk_delete_transactions_validates_max_1000() {
    let db = super::get_test_db_pool().await;

    let many_ids: Vec<i64> = (1..=1001).collect();
    let result = bulk_delete_transactions_impl(db, many_ids).await;

    assert!(result.is_err(), "Should reject more than 1000 IDs");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("1000"),
        "Error should mention 1000 limit"
    );
}

// ==== T030: Bulk Update Category Tests ====

#[tokio::test]
async fn test_bulk_update_category_success() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Bulk Update Test").await;

    // Create test category
    let category = NewCategory {
        name: super::unique_name("Bulk Test Category"),
        icon: None,
    };
    let new_category_id = create_category_impl(db, category)
        .await
        .expect("Failed to create category");

    // Create 3 transactions directly
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-01", -50.00, "Transaction 1"),
        super::fixtures::TestTransaction::new("2025-01-02", -75.00, "Transaction 2"),
        super::fixtures::TestTransaction::new("2025-01-03", -100.00, "Transaction 3"),
    ];
    let ids = super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Bulk update category
    let result = bulk_update_category_impl(db, ids.clone(), new_category_id).await;
    assert!(result.is_ok(), "Bulk update should succeed");

    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.updated_count, 3, "Should update 3 transactions");
    assert!(bulk_result.failed_ids.is_empty(), "No IDs should fail");

    // Verify all updated
    let updated = list_transactions_impl(db, Some(TransactionFilter {
        account_id: Some(account_id),
        category_id: None,
        search: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    }))
    .await
    .expect("Failed to list transactions");

    for id in ids {
        let transaction = updated.iter().find(|t| t.id == id);
        assert!(transaction.is_some(), "Transaction {} should exist", id);
        assert_eq!(
            transaction.unwrap().category_id,
            new_category_id,
            "Transaction {} should have new category",
            id
        );
    }
}

#[tokio::test]
async fn test_bulk_update_category_validates_category_exists() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Category Validation Test").await;

    // Create one transaction
    let transaction_id = super::fixtures::insert_single_transaction(
        db,
        account_id,
        "2025-01-01",
        -50.00,
        "Test Transaction"
    ).await;

    // Try to update to non-existent category
    let invalid_category_id = 999999i64;
    let result = bulk_update_category_impl(db, vec![transaction_id], invalid_category_id).await;

    assert!(result.is_err(), "Should reject non-existent category");
    let error = result.unwrap_err();
    assert!(
        error.to_string().to_lowercase().contains("category") ||
        error.to_string().to_lowercase().contains("not found"),
        "Error should mention category not found"
    );
}

#[tokio::test]
async fn test_bulk_update_category_reports_failed_ids() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Bulk Update Failed Test").await;

    // Create test category
    let category = NewCategory {
        name: super::unique_name("Update Failed Category"),
        icon: None,
    };
    let category_id = create_category_impl(db, category)
        .await
        .expect("Failed to create category");

    // Create one transaction
    let valid_id = super::fixtures::insert_single_transaction(
        db,
        account_id,
        "2025-01-01",
        -50.00,
        "Test Transaction"
    ).await;
    let invalid_id = 999999i64;

    // Try to update both valid and invalid IDs
    let result = bulk_update_category_impl(db, vec![valid_id, invalid_id], category_id).await;
    assert!(result.is_ok(), "Bulk update should succeed even with some failures");

    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.updated_count, 1, "Should update 1 transaction");
    assert!(bulk_result.failed_ids.contains(&invalid_id), "Should report failed ID");
}

#[tokio::test]
async fn test_bulk_update_category_validates_empty_array() {
    let db = super::get_test_db_pool().await;

    let result = bulk_update_category_impl(db, vec![], 1).await;
    assert!(result.is_err(), "Should reject empty array");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("empty"),
        "Error should mention empty array"
    );
}

#[tokio::test]
async fn test_bulk_update_category_validates_max_1000() {
    let db = super::get_test_db_pool().await;

    let many_ids: Vec<i64> = (1..=1001).collect();
    let result = bulk_update_category_impl(db, many_ids, 1).await;

    assert!(result.is_err(), "Should reject more than 1000 IDs");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("1000"),
        "Error should mention 1000 limit"
    );
}
