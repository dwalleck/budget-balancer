use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::commands::analytics_commands::get_dashboard_summary_impl;
use budget_balancer_lib::commands::csv_commands::import_csv_impl;
use budget_balancer_lib::models::account::NewAccount;
use budget_balancer_lib::services::csv_parser::ColumnMapping;

#[tokio::test]
async fn test_get_dashboard_summary_current_month() {
    let db = super::get_test_db_pool().await;
    let result = get_dashboard_summary_impl(db, "current_month").await;

    assert!(result.is_ok(), "Failed to get dashboard summary: {:?}", result);

    let response = result.unwrap();
    assert!(response.total_spending >= 0.0, "Total spending should be >= 0");
    assert!(response.total_income >= 0.0, "Total income should be >= 0");
    assert!(response.top_categories.len() <= 5, "Should have at most 5 top categories");
    assert!(response.debt_summary.total_debt >= 0.0, "Total debt should be >= 0");
}

#[tokio::test]
async fn test_get_dashboard_summary_last_30_days() {
    let db = super::get_test_db_pool().await;
    let result = get_dashboard_summary_impl(db, "last_30_days").await;

    assert!(result.is_ok(), "Should get dashboard for last 30 days");

    let response = result.unwrap();
    // Net = income - spending
    assert_eq!(
        response.net,
        response.total_income - response.total_spending,
        "Net should equal income - spending"
    );
}

#[tokio::test]
async fn test_get_dashboard_summary_current_year() {
    let db = super::get_test_db_pool().await;
    let result = get_dashboard_summary_impl(db, "current_year").await;

    assert!(result.is_ok(), "Should get dashboard for current year");
}

#[tokio::test]
async fn test_dashboard_with_data() {
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Dashboard Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import transactions
    let csv_content = "Date,Amount,Description,Merchant\n2025-01-15,-100.00,Groceries,Whole Foods\n2025-01-20,500.00,Paycheck,Employer";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

    // Get dashboard
    let result = get_dashboard_summary_impl(db, "current_month").await;

    assert!(result.is_ok(), "Dashboard should work with data");

    let response = result.unwrap();
    assert!(response.total_spending > 0.0 || response.total_income > 0.0, "Should have some financial activity");
}
