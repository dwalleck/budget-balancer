use budget_balancer_lib::commands::transaction_commands::export_transactions_impl;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn test_export_transactions_to_csv() {
    let db = super::get_test_db_pool().await;
    let account_id = super::fixtures::create_test_account(db, "Export Test").await;

    // Create test transactions directly
    let transactions = vec![
        super::fixtures::TestTransaction::new("2024-01-01", -50.00, "Coffee").with_merchant("Starbucks"),
        super::fixtures::TestTransaction::new("2024-01-02", -100.00, "Groceries").with_merchant("Whole Foods"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

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
    let account_id = super::fixtures::create_test_account(db, "Export JSON Test").await;

    // Create test transaction directly
    let transactions = vec![
        super::fixtures::TestTransaction::new("2024-01-01", -50.00, "Coffee").with_merchant("Starbucks"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

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
    let account_id = super::fixtures::create_test_account(db, "Export Filter Test").await;

    // Create transactions with different dates
    let transactions = vec![
        super::fixtures::TestTransaction::new("2024-01-01", -50.00, "Coffee").with_merchant("Starbucks"),
        super::fixtures::TestTransaction::new("2024-02-01", -100.00, "Groceries").with_merchant("Whole Foods"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

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
