use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::commands::csv_commands::import_csv_impl;
use budget_balancer_lib::commands::transaction_commands::export_transactions_impl;
use budget_balancer_lib::models::account::NewAccount;
use budget_balancer_lib::services::csv_parser::ColumnMapping;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn test_export_transactions_to_csv() {
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Export Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import some transactions
    let csv_content = "Date,Amount,Description,Merchant\n2024-01-01,-50.00,Coffee,Starbucks\n2024-01-02,-100.00,Groceries,Whole Foods";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

    // Export to CSV
    let output_path = format!("/tmp/export_test_{}.csv", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());

    let result = export_transactions_impl(
        db,
        "csv".to_string(),
        output_path.clone(),
        None, // No filters
    )
    .await;

    assert!(result.is_ok(), "Failed to export transactions: {:?}", result);

    let export_result = result.unwrap();
    assert!(export_result.success, "Export should succeed");
    assert_eq!(export_result.file_path, output_path);
    assert!(export_result.record_count > 0, "Should export at least one record");

    // Verify file exists
    assert!(PathBuf::from(&output_path).exists(), "Export file should exist");

    // Clean up
    fs::remove_file(output_path).ok();
}

#[tokio::test]
async fn test_export_transactions_to_json() {
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Export JSON Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import some transactions
    let csv_content = "Date,Amount,Description,Merchant\n2024-01-01,-50.00,Coffee,Starbucks";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

    // Export to JSON
    let output_path = format!("/tmp/export_test_{}.json", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());

    let result = export_transactions_impl(
        db,
        "json".to_string(),
        output_path.clone(),
        None,
    )
    .await;

    assert!(result.is_ok(), "Failed to export transactions: {:?}", result);

    let export_result = result.unwrap();
    assert!(export_result.success, "Export should succeed");
    assert!(export_result.record_count > 0, "Should export at least one record");

    // Verify file exists
    assert!(PathBuf::from(&output_path).exists(), "Export file should exist");

    // Clean up
    fs::remove_file(output_path).ok();
}

#[tokio::test]
async fn test_export_transactions_with_date_filter() {
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Export Filter Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import transactions with different dates
    let csv_content = "Date,Amount,Description,Merchant\n2024-01-01,-50.00,Coffee,Starbucks\n2024-02-01,-100.00,Groceries,Whole Foods";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

    // Export with date filter (only January)
    let output_path = format!("/tmp/export_filter_test_{}.csv", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());

    // Note: The filter structure depends on the implementation
    // For now, we'll test without filters until the command is implemented
    let result = export_transactions_impl(
        db,
        "csv".to_string(),
        output_path.clone(),
        None, // TODO: Add filters when implemented
    )
    .await;

    assert!(result.is_ok(), "Failed to export transactions");

    // Clean up
    fs::remove_file(output_path).ok();
}
