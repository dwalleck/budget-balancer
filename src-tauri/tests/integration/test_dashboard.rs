use budget_balancer_lib::commands::analytics_commands::get_dashboard_summary_impl;

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
    let account_id = super::fixtures::create_test_account(db, "Dashboard Test").await;

    // Create test transactions with relative dates (2 and 4 days ago)
    // This ensures tests work regardless of current date or month
    let transactions = vec![
        super::fixtures::TestTransaction::new(&super::days_ago(4), -100.00, "Groceries").with_merchant("Whole Foods"),
        super::fixtures::TestTransaction::new(&super::days_ago(2), 500.00, "Paycheck").with_merchant("Employer"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    // Get dashboard
    let result = get_dashboard_summary_impl(db, "current_month").await;

    assert!(result.is_ok(), "Dashboard should work with data");

    let response = result.unwrap();
    println!("Dashboard response: total_spending={}, total_income={}", response.total_spending, response.total_income);
    assert!(response.total_spending > 0.0 || response.total_income > 0.0, "Should have some financial activity. Got spending={}, income={}", response.total_spending, response.total_income);
}
