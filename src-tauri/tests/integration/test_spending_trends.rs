use budget_balancer_lib::commands::analytics_commands::get_spending_trends_impl;

#[tokio::test]
async fn test_get_spending_trends_monthly() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Trends Test").await;

    // Create transactions across multiple months
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-15", -100.00, "Groceries").with_merchant("Whole Foods"),
        super::fixtures::TestTransaction::new("2025-02-20", -150.00, "Groceries").with_merchant("Whole Foods"),
        super::fixtures::TestTransaction::new("2025-03-10", -120.00, "Groceries").with_merchant("Whole Foods"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Get monthly trends
    let result = get_spending_trends_impl(
        db,
        "2025-01-01",
        "2025-12-31",
        "monthly",
        None,
    )
    .await;

    assert!(result.is_ok(), "Failed to get spending trends: {:?}", result);

    let response = result.unwrap();
    assert_eq!(response.data_points.len(), 12, "Should have 12 monthly data points");
    assert!(response.average_per_interval >= 0.0, "Average should be >= 0");
}

#[tokio::test]
async fn test_get_spending_trends_for_category() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Category Trends Test").await;

    // Create transactions directly
    let transactions = vec![
        super::fixtures::TestTransaction::new("2025-01-15", -50.00, "Coffee").with_merchant("Starbucks"),
        super::fixtures::TestTransaction::new("2025-02-20", -60.00, "Coffee").with_merchant("Starbucks"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Get trends for specific category (assumes category ID 1 exists)
    let result = get_spending_trends_impl(
        db,
        "2025-01-01",
        "2025-12-31",
        "monthly",
        Some(1),
    )
    .await;

    assert!(result.is_ok(), "Should get trends for specific category");
}

#[tokio::test]
async fn test_get_spending_trends_weekly() {
    let db = super::get_test_db_pool().await;
    let result = get_spending_trends_impl(
        db,
        "2025-01-01",
        "2025-01-31",
        "weekly",
        None,
    )
    .await;

    assert!(result.is_ok(), "Should support weekly interval");

    let response = result.unwrap();
    assert!(response.data_points.len() >= 4, "Should have at least 4 weekly data points for January");
}
