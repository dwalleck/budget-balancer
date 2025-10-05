use budget_balancer_lib::commands::analytics_commands::get_spending_by_category_impl;

#[tokio::test]
async fn test_get_spending_by_category() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Spending Test").await;

    // Create test transactions directly
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-15", -100.00, "Groceries").with_merchant("Whole Foods"),
        super::fixtures::TestTransaction::new("2025-01-20", -50.00, "Coffee").with_merchant("Starbucks"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Get spending by category
    let result = get_spending_by_category_impl(
        db,
        "2025-01-01",
        "2025-01-31",
        Some(account_id),
    )
    .await;

    assert!(result.is_ok(), "Failed to get spending by category: {:?}", result);

    let response = result.unwrap();
    assert!(response.categories.len() > 0, "Should have at least one category");
    assert!(response.total_spending > 0.0, "Total spending should be greater than 0");

    // Verify percentages sum to ~100
    let total_percentage: f64 = response.categories.iter().map(|c| c.percentage).sum();
    assert!((total_percentage - 100.0).abs() < 1.0, "Percentages should sum to ~100");
}

#[tokio::test]
async fn test_get_spending_by_category_empty_range() {
    let db = super::get_test_db_pool().await;
    let result = get_spending_by_category_impl(
        db,
        "2020-01-01",
        "2020-01-31",
        None,
    )
    .await;

    assert!(result.is_ok(), "Should succeed even with no transactions");

    let response = result.unwrap();
    assert_eq!(response.categories.len(), 0, "Should have no categories");
    assert_eq!(response.total_spending, 0.0, "Total spending should be 0");
}

#[tokio::test]
async fn test_get_spending_by_category_with_account_filter() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Filter Test").await;

    // Create test transaction directly
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-15", -75.00, "Gas").with_merchant("Shell"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Get spending filtered by account
    let result = get_spending_by_category_impl(
        db,
        "2025-01-01",
        "2025-01-31",
        Some(account_id),
    )
    .await;

    assert!(result.is_ok(), "Should succeed with account filter");
}
