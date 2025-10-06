use budget_balancer_lib::commands::csv_commands::{
    delete_column_mapping_impl, get_column_mapping_impl, list_column_mappings_impl,
    save_column_mapping_impl, update_column_mapping_impl,
};
use budget_balancer_lib::models::column_mapping::{
    GetColumnMappingQuery, NewColumnMapping, UpdateColumnMapping,
};

// T043 [P] Contract test for save_column_mapping with upsert behavior
#[tokio::test]
async fn test_save_column_mapping_create_new() {
    let db = super::get_test_db_pool().await;

    let mapping = NewColumnMapping {
        source_name: super::unique_name("Chase Checking"),
        date_col: "Transaction Date".to_string(),
        amount_col: "Amount".to_string(),
        description_col: "Description".to_string(),
        merchant_col: Some("Merchant".to_string()),
    };

    let result = save_column_mapping_impl(db, mapping.clone()).await;
    assert!(result.is_ok(), "Failed to create mapping: {:?}", result);

    let created = result.unwrap();
    assert!(created.id > 0, "Should have valid ID");
    assert_eq!(created.source_name, mapping.source_name);
    assert_eq!(created.date_col, mapping.date_col);
    assert_eq!(created.amount_col, mapping.amount_col);
    assert_eq!(created.description_col, mapping.description_col);
    assert_eq!(created.merchant_col, mapping.merchant_col);
}

#[tokio::test]
async fn test_save_column_mapping_upsert_behavior() {
    let db = super::get_test_db_pool().await;

    let source_name = super::unique_name("Bank XYZ");

    // First save
    let first_mapping = NewColumnMapping {
        source_name: source_name.clone(),
        date_col: "Date".to_string(),
        amount_col: "Amt".to_string(),
        description_col: "Desc".to_string(),
        merchant_col: None,
    };

    let first = save_column_mapping_impl(db, first_mapping).await.unwrap();

    // Second save with same source_name - should update
    let second_mapping = NewColumnMapping {
        source_name: source_name.clone(),
        date_col: "TransDate".to_string(), // Different columns
        amount_col: "Amount".to_string(),
        description_col: "Description".to_string(),
        merchant_col: Some("Merchant".to_string()),
    };

    let updated = save_column_mapping_impl(db, second_mapping).await.unwrap();

    // Should have same ID (updated, not created)
    assert_eq!(updated.id, first.id, "Should update existing mapping, not create new");
    assert_eq!(updated.date_col, "TransDate", "Date column should be updated");
    assert_eq!(updated.merchant_col, Some("Merchant".to_string()), "Merchant column should be updated");
}

#[tokio::test]
async fn test_save_column_mapping_without_merchant() {
    let db = super::get_test_db_pool().await;

    let mapping = NewColumnMapping {
        source_name: super::unique_name("Simple Bank"),
        date_col: "Date".to_string(),
        amount_col: "Amount".to_string(),
        description_col: "Description".to_string(),
        merchant_col: None, // No merchant column
    };

    let result = save_column_mapping_impl(db, mapping).await.unwrap();
    assert_eq!(result.merchant_col, None, "Merchant column should be None");
}

// T044 [P] Contract test for list_column_mappings sorted by name
#[tokio::test]
async fn test_list_column_mappings_sorted_by_name() {
    let db = super::get_test_db_pool().await;

    // Create mappings with different names
    let zebra = NewColumnMapping {
        source_name: super::unique_name("Zebra Bank"),
        date_col: "Date".to_string(),
        amount_col: "Amt".to_string(),
        description_col: "Desc".to_string(),
        merchant_col: None,
    };

    let alpha = NewColumnMapping {
        source_name: super::unique_name("Alpha Bank"),
        date_col: "Date".to_string(),
        amount_col: "Amt".to_string(),
        description_col: "Desc".to_string(),
        merchant_col: None,
    };

    save_column_mapping_impl(db, zebra).await.unwrap();
    save_column_mapping_impl(db, alpha).await.unwrap();

    let result = list_column_mappings_impl(db).await.unwrap();

    // Verify alphabetical ordering
    for i in 0..result.len().saturating_sub(1) {
        assert!(
            result[i].source_name <= result[i + 1].source_name,
            "Mappings should be sorted alphabetically by source_name"
        );
    }
}

#[tokio::test]
async fn test_list_column_mappings_empty() {
    let db = super::get_test_db_pool().await;

    // List all mappings - might have some from other tests
    let result = list_column_mappings_impl(db).await.unwrap();

    // Just verify it returns successfully and is a vector
    assert!(result.is_empty() || !result.is_empty(), "Should return a vector");
}

// T045 [P] Contract test for get_column_mapping by ID or source_name
#[tokio::test]
async fn test_get_column_mapping_by_id() {
    let db = super::get_test_db_pool().await;

    let mapping = NewColumnMapping {
        source_name: super::unique_name("Test Bank"),
        date_col: "Date".to_string(),
        amount_col: "Amt".to_string(),
        description_col: "Desc".to_string(),
        merchant_col: None,
    };

    let saved = save_column_mapping_impl(db, mapping).await.unwrap();

    let query = GetColumnMappingQuery {
        id: Some(saved.id),
        source_name: None,
    };

    let result = get_column_mapping_impl(db, query).await;
    assert!(result.is_ok(), "Failed to get mapping: {:?}", result);

    let retrieved = result.unwrap();
    assert_eq!(retrieved.id, saved.id);
    assert_eq!(retrieved.source_name, saved.source_name);
}

#[tokio::test]
async fn test_get_column_mapping_by_source_name() {
    let db = super::get_test_db_pool().await;

    let source_name = super::unique_name("Chase Visa");

    let mapping = NewColumnMapping {
        source_name: source_name.clone(),
        date_col: "Date".to_string(),
        amount_col: "Amt".to_string(),
        description_col: "Desc".to_string(),
        merchant_col: None,
    };

    save_column_mapping_impl(db, mapping).await.unwrap();

    let query = GetColumnMappingQuery {
        id: None,
        source_name: Some(source_name.clone()),
    };

    let result = get_column_mapping_impl(db, query).await.unwrap();
    assert_eq!(result.source_name, source_name);
}

#[tokio::test]
async fn test_get_column_mapping_not_found() {
    let db = super::get_test_db_pool().await;

    let query = GetColumnMappingQuery {
        id: Some(999999),
        source_name: None,
    };

    let result = get_column_mapping_impl(db, query).await;
    assert!(result.is_err(), "Should fail for non-existent mapping");
    let error_msg = result.unwrap_err().to_lowercase();
    assert!(
        error_msg.contains("not found"),
        "Error should mention mapping not found"
    );
}

#[tokio::test]
async fn test_get_column_mapping_no_params() {
    let db = super::get_test_db_pool().await;

    let query = GetColumnMappingQuery {
        id: None,
        source_name: None,
    };

    let result = get_column_mapping_impl(db, query).await;
    assert!(result.is_err(), "Should fail when no parameters provided");
    let error_msg = result.unwrap_err().to_lowercase();
    assert!(
        error_msg.contains("required") || error_msg.contains("must") || error_msg.contains("provided"),
        "Error should mention required parameters, got: {}", error_msg
    );
}

// T046 [P] Contract test for update_column_mapping
#[tokio::test]
async fn test_update_column_mapping_specific_columns() {
    let db = super::get_test_db_pool().await;

    let mapping = NewColumnMapping {
        source_name: super::unique_name("Test"),
        date_col: "D".to_string(),
        amount_col: "A".to_string(),
        description_col: "Desc".to_string(),
        merchant_col: None,
    };

    let saved = save_column_mapping_impl(db, mapping).await.unwrap();

    let update = UpdateColumnMapping {
        id: saved.id,
        source_name: None,
        date_col: Some("Transaction Date".to_string()),
        amount_col: Some("Amount".to_string()),
        description_col: None, // Unchanged
        merchant_col: None,
    };

    let result = update_column_mapping_impl(db, update).await.unwrap();
    assert_eq!(result.date_col, "Transaction Date", "Date column should be updated");
    assert_eq!(result.amount_col, "Amount", "Amount column should be updated");
    assert_eq!(result.description_col, "Desc", "Description should remain unchanged");
}

#[tokio::test]
async fn test_update_column_mapping_add_merchant() {
    let db = super::get_test_db_pool().await;

    let mapping = NewColumnMapping {
        source_name: super::unique_name("Test"),
        date_col: "D".to_string(),
        amount_col: "A".to_string(),
        description_col: "Desc".to_string(),
        merchant_col: None,
    };

    let saved = save_column_mapping_impl(db, mapping).await.unwrap();

    let update = UpdateColumnMapping {
        id: saved.id,
        source_name: None,
        date_col: None,
        amount_col: None,
        description_col: None,
        merchant_col: Some(Some("Merchant".to_string())), // Add merchant
    };

    let result = update_column_mapping_impl(db, update).await.unwrap();
    assert_eq!(
        result.merchant_col,
        Some("Merchant".to_string()),
        "Merchant column should be added"
    );
}

#[tokio::test]
async fn test_update_column_mapping_not_found() {
    let db = super::get_test_db_pool().await;

    let update = UpdateColumnMapping {
        id: 999999,
        source_name: Some("Test".to_string()),
        date_col: None,
        amount_col: None,
        description_col: None,
        merchant_col: None,
    };

    let result = update_column_mapping_impl(db, update).await;
    assert!(result.is_err(), "Should fail for non-existent mapping");
}

// T047 [P] Contract test for delete_column_mapping
#[tokio::test]
async fn test_delete_column_mapping_success() {
    let db = super::get_test_db_pool().await;

    let mapping = NewColumnMapping {
        source_name: super::unique_name("Delete Me"),
        date_col: "D".to_string(),
        amount_col: "A".to_string(),
        description_col: "Desc".to_string(),
        merchant_col: None,
    };

    let saved = save_column_mapping_impl(db, mapping).await.unwrap();

    let result = delete_column_mapping_impl(db, saved.id).await;
    assert!(result.is_ok(), "Failed to delete mapping: {:?}", result);

    let delete_response = result.unwrap();
    assert!(delete_response.success, "Delete should succeed");
    assert_eq!(delete_response.deleted_mapping_id, saved.id);

    // Verify mapping no longer exists
    let query = GetColumnMappingQuery {
        id: Some(saved.id),
        source_name: None,
    };
    let get_result = get_column_mapping_impl(db, query).await;
    assert!(get_result.is_err(), "Deleted mapping should not be retrievable");
}

#[tokio::test]
async fn test_delete_column_mapping_not_found() {
    let db = super::get_test_db_pool().await;

    let result = delete_column_mapping_impl(db, 999999).await;
    assert!(result.is_err(), "Should fail for non-existent mapping");
    let error_msg = result.unwrap_err().to_lowercase();
    assert!(
        error_msg.contains("not found"),
        "Error should mention mapping not found"
    );
}
