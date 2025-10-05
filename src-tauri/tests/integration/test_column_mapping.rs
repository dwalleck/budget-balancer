use budget_balancer_lib::commands::csv_commands::save_column_mapping_impl;
use budget_balancer_lib::models::column_mapping::NewColumnMapping;

#[tokio::test]
async fn test_save_column_mapping() {
    let db = super::get_test_db_pool().await;
    let mapping = NewColumnMapping {
        source_name: super::unique_name("Test Bank"),
        date_col: "Date".to_string(),
        amount_col: "Amount".to_string(),
        description_col: "Description".to_string(),
        merchant_col: Some("Merchant".to_string()),
    };

    let result = save_column_mapping_impl(db, mapping).await;
    assert!(result.is_ok(), "Failed to save column mapping: {:?}", result);

    let mapping_id = result.unwrap();
    assert!(mapping_id > 0, "Mapping ID should be greater than 0");
}

#[tokio::test]
async fn test_save_column_mapping_duplicate_name() {
    let db = super::get_test_db_pool().await;
    let mapping_name = super::unique_name("Duplicate Test Bank");

    let mapping1 = NewColumnMapping {
        source_name: mapping_name.clone(),
        date_col: "Date".to_string(),
        amount_col: "Amount".to_string(),
        description_col: "Description".to_string(),
        merchant_col: None,
    };

    // First save should succeed
    let result1 = save_column_mapping_impl(db, mapping1).await;
    assert!(result1.is_ok(), "First save should succeed");

    // Second save with same name should fail
    let mapping2 = NewColumnMapping {
        source_name: mapping_name.clone(),
        date_col: "Date2".to_string(),
        amount_col: "Amount2".to_string(),
        description_col: "Description2".to_string(),
        merchant_col: None,
    };

    let result2 = save_column_mapping_impl(db, mapping2).await;
    assert!(result2.is_err(), "Duplicate source_name should fail");
}

#[tokio::test]
async fn test_save_column_mapping_without_merchant() {
    let db = super::get_test_db_pool().await;
    let mapping = NewColumnMapping {
        source_name: super::unique_name("Bank Without Merchant"),
        date_col: "Date".to_string(),
        amount_col: "Amount".to_string(),
        description_col: "Description".to_string(),
        merchant_col: None,
    };

    let result = save_column_mapping_impl(db, mapping).await;
    assert!(result.is_ok(), "Should save mapping without merchant column");
}
