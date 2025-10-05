use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::commands::analytics_commands::get_spending_by_category_impl;
use budget_balancer_lib::commands::csv_commands::import_csv_impl;
use budget_balancer_lib::models::account::NewAccount;
use budget_balancer_lib::services::csv_parser::ColumnMapping;

#[tokio::test]
async fn test_get_spending_by_category() {
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Spending Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import transactions
    let csv_content = "Date,Amount,Description,Merchant\n2025-01-15,-100.00,Groceries,Whole Foods\n2025-01-20,-50.00,Coffee,Starbucks";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

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
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Filter Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import transaction
    let csv_content = "Date,Amount,Description,Merchant\n2025-01-15,-75.00,Gas,Shell";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

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
