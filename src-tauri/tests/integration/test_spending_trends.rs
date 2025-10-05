use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::commands::analytics_commands::get_spending_trends_impl;
use budget_balancer_lib::commands::csv_commands::import_csv_impl;
use budget_balancer_lib::models::account::NewAccount;
use budget_balancer_lib::services::csv_parser::ColumnMapping;

#[tokio::test]
async fn test_get_spending_trends_monthly() {
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Trends Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import transactions across multiple months
    let csv_content = "Date,Amount,Description,Merchant\n2025-01-15,-100.00,Groceries,Whole Foods\n2025-02-20,-150.00,Groceries,Whole Foods\n2025-03-10,-120.00,Groceries,Whole Foods";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

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
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Category Trends Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import transactions
    let csv_content = "Date,Amount,Description,Merchant\n2025-01-15,-50.00,Coffee,Starbucks\n2025-02-20,-60.00,Coffee,Starbucks";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

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
