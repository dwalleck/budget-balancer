use budget_balancer_lib::commands::category_commands::{
    create_category_impl, delete_category_impl, list_categories_impl, update_category_impl,
};
use budget_balancer_lib::commands::transaction_commands::update_transaction_category_impl;
use budget_balancer_lib::models::category::{CategoryFilter, NewCategory, UpdateCategory};
use sqlx::Row;

// T034 [P] Contract test for create_category command
#[tokio::test]
async fn test_create_category() {
    let db = super::get_test_db_pool().await;
    let category = NewCategory {
        name: super::unique_name("Test Category"),
        icon: Some("💰".to_string()),
    };

    let result = create_category_impl(db, category).await;
    assert!(result.is_ok(), "Failed to create category: {:?}", result);

    let category_id = result.unwrap();
    assert!(category_id > 0, "Category ID should be positive");
}

#[tokio::test]
async fn test_create_category_without_icon() {
    let db = super::get_test_db_pool().await;
    let category = NewCategory {
        name: super::unique_name("No Icon Category"),
        icon: None,
    };

    let result = create_category_impl(db, category).await;
    assert!(result.is_ok(), "Failed to create category without icon");
}

#[tokio::test]
async fn test_create_category_duplicate_name() {
    let db = super::get_test_db_pool().await;
    let name = super::unique_name("Duplicate Category");

    let category1 = NewCategory {
        name: name.clone(),
        icon: Some("🎯".to_string()),
    };

    let result1 = create_category_impl(db, category1).await;
    assert!(result1.is_ok(), "First category creation should succeed");

    let category2 = NewCategory {
        name,
        icon: Some("🎨".to_string()),
    };

    let result2 = create_category_impl(db, category2).await;
    assert!(result2.is_err(), "Duplicate category name should fail");
    let error_msg = result2.unwrap_err();
    let error_msg_lower = error_msg.to_lowercase();
    assert!(
        error_msg_lower.contains("already exists") || error_msg_lower.contains("duplicate"),
        "Error should mention duplicate category, got: {}", error_msg
    );
}

// T035 [P] Contract test for create_category (custom) - verify type is always 'custom'
#[tokio::test]
async fn test_create_category_always_custom_type() {
    let db = super::get_test_db_pool().await;
    let category = NewCategory {
        name: super::unique_name("Custom Category"),
        icon: Some("🎯".to_string()),
    };

    let category_id = create_category_impl(db, category).await.unwrap();

    // Fetch the created category and verify type
    let categories = list_categories_impl(db, None).await.unwrap();
    let created = categories.iter().find(|c| c.id == category_id).unwrap();

    assert_eq!(created.r#type, "custom", "User-created categories must have type 'custom'");
}

// T036 [P] Contract test for list_categories with type filter
#[tokio::test]
async fn test_list_categories() {
    let db = super::get_test_db_pool().await;

    // Create a test category first
    let category = NewCategory {
        name: super::unique_name("List Test Category"),
        icon: Some("📊".to_string()),
    };
    let _ = create_category_impl(db, category).await.expect("Failed to create category");

    let result = list_categories_impl(db, None).await;
    assert!(result.is_ok(), "Failed to list categories: {:?}", result);

    let categories = result.unwrap();
    assert!(!categories.is_empty(), "Should have at least one category (seeded or created)");

    // Verify category structure
    let first = &categories[0];
    assert!(first.id > 0);
    assert!(!first.name.is_empty());
}

#[tokio::test]
async fn test_list_categories_includes_seeded_categories() {
    let db = super::get_test_db_pool().await;
    let categories = list_categories_impl(db, None).await.expect("Failed to list categories");

    // Should have seeded predefined categories
    let category_names: Vec<String> = categories.iter().map(|c| c.name.clone()).collect();

    assert!(
        category_names.iter().any(|n| n.contains("Groceries")),
        "Should have Groceries category from seed data"
    );
}

#[tokio::test]
async fn test_list_categories_ordered_by_name() {
    let db = super::get_test_db_pool().await;
    let categories = list_categories_impl(db, None).await.expect("Failed to list categories");

    // Verify categories are ordered by name
    for i in 0..categories.len().saturating_sub(1) {
        assert!(
            categories[i].name <= categories[i + 1].name,
            "Categories should be ordered by name"
        );
    }
}

#[tokio::test]
async fn test_list_categories_filter_by_predefined() {
    let db = super::get_test_db_pool().await;

    let categories = list_categories_impl(db, Some(CategoryFilter::Predefined)).await.unwrap();

    assert!(!categories.is_empty(), "Should have predefined categories");
    assert!(
        categories.iter().all(|c| c.r#type == "predefined"),
        "All categories should be predefined type"
    );
}

#[tokio::test]
async fn test_list_categories_filter_by_custom() {
    let db = super::get_test_db_pool().await;

    // Create a custom category
    let category = NewCategory {
        name: super::unique_name("Custom Filter Test"),
        icon: Some("🔧".to_string()),
    };
    let _ = create_category_impl(db, category).await.unwrap();

    let categories = list_categories_impl(db, Some(CategoryFilter::Custom)).await.unwrap();

    assert!(!categories.is_empty(), "Should have at least one custom category");
    assert!(
        categories.iter().all(|c| c.r#type == "custom"),
        "All categories should be custom type"
    );
}

// T037 [P] Contract test for update_category (custom only)
#[tokio::test]
async fn test_update_category_name() {
    let db = super::get_test_db_pool().await;

    let old_name = super::unique_name("Old Name");
    let new_name = super::unique_name("New Name");

    let category = NewCategory {
        name: old_name,
        icon: Some("🎯".to_string()),
    };
    let category_id = create_category_impl(db, category.clone()).await.unwrap();

    let update = UpdateCategory {
        id: category_id,
        name: Some(new_name.clone()),
        icon: None,
    };

    let result = update_category_impl(db, update).await;
    assert!(result.is_ok(), "Failed to update category: {:?}", result);

    let updated = result.unwrap();
    assert_eq!(updated.name, new_name, "Name should be updated");
    assert_eq!(updated.icon, category.icon, "Icon should remain unchanged");
}

#[tokio::test]
async fn test_update_category_icon_only() {
    let db = super::get_test_db_pool().await;

    let category = NewCategory {
        name: super::unique_name("Icon Test"),
        icon: Some("🎯".to_string()),
    };
    let category_id = create_category_impl(db, category.clone()).await.unwrap();

    let update = UpdateCategory {
        id: category_id,
        name: None,
        icon: Some("🎨".to_string()),
    };

    let result = update_category_impl(db, update).await;
    assert!(result.is_ok(), "Failed to update category icon");

    let updated = result.unwrap();
    assert_eq!(updated.name, category.name, "Name should remain unchanged");
    assert_eq!(updated.icon, Some("🎨".to_string()), "Icon should be updated");
}

#[tokio::test]
async fn test_update_category_reject_predefined() {
    let db = super::get_test_db_pool().await;

    // Get a predefined category
    let categories = list_categories_impl(db, Some(CategoryFilter::Predefined)).await.unwrap();
    let predefined_id = categories[0].id;

    let update = UpdateCategory {
        id: predefined_id,
        name: Some("Modified Predefined".to_string()),
        icon: None,
    };

    let result = update_category_impl(db, update).await;
    assert!(result.is_err(), "Should reject update of predefined category");
    let error_msg = result.unwrap_err().to_lowercase();
    assert!(
        error_msg.contains("predefined") || error_msg.contains("modify"),
        "Error should mention predefined protection"
    );
}

#[tokio::test]
async fn test_update_category_not_found() {
    let db = super::get_test_db_pool().await;

    let update = UpdateCategory {
        id: 999999,
        name: Some("Non-existent".to_string()),
        icon: None,
    };

    let result = update_category_impl(db, update).await;
    assert!(result.is_err(), "Should fail for non-existent category");
    assert!(
        result.unwrap_err().to_lowercase().contains("not found"),
        "Error should mention category not found"
    );
}

// T038 [P] Contract test for delete_category with reassignment to Uncategorized
#[tokio::test]
async fn test_delete_category_with_transaction_reassignment() {
    let db = super::get_test_db_pool().await;

    // Create a custom category
    let category = NewCategory {
        name: super::unique_name("To Delete"),
        icon: Some("🗑️".to_string()),
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    // Get a transaction and assign it to this category
    let account_id = super::fixtures::create_test_account(db, "Delete Category Test").await;
    let transactions = vec![
        super::fixtures::TestTransaction::new(&super::days_ago(1), -50.00, "Test Transaction")
            .with_merchant("Test Merchant"),
    ];
    super::fixtures::insert_test_transactions(db, account_id, transactions).await;

    let all_transactions = sqlx::query("SELECT id FROM transactions WHERE account_id = ? LIMIT 1")
        .bind(account_id)
        .fetch_one(db)
        .await
        .unwrap();
    let transaction_id: i64 = all_transactions.get("id");

    update_transaction_category_impl(db, transaction_id, category_id).await.unwrap();

    // Delete the category
    let result = delete_category_impl(db, category_id).await;
    assert!(result.is_ok(), "Failed to delete category: {:?}", result);

    let delete_response = result.unwrap();
    assert!(delete_response.success, "Delete should succeed");
    assert_eq!(
        delete_response.reassigned_transactions_count, 1,
        "Should reassign 1 transaction"
    );

    // Verify transaction was reassigned to Uncategorized
    let uncategorized_id = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM categories WHERE name = 'Uncategorized'"
    )
    .fetch_one(db)
    .await
    .unwrap()
    .0;

    let updated_transaction = sqlx::query("SELECT category_id FROM transactions WHERE id = ?")
        .bind(transaction_id)
        .fetch_one(db)
        .await
        .unwrap();
    let new_category_id: i64 = updated_transaction.get("category_id");

    assert_eq!(
        new_category_id, uncategorized_id,
        "Transaction should be reassigned to Uncategorized"
    );
}

#[tokio::test]
async fn test_delete_category_no_transactions() {
    let db = super::get_test_db_pool().await;

    let category = NewCategory {
        name: super::unique_name("Empty Category"),
        icon: Some("📭".to_string()),
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    let result = delete_category_impl(db, category_id).await;
    assert!(result.is_ok(), "Failed to delete empty category");

    let delete_response = result.unwrap();
    assert!(delete_response.success, "Delete should succeed");
    assert_eq!(
        delete_response.reassigned_transactions_count, 0,
        "Should reassign 0 transactions"
    );
}

#[tokio::test]
async fn test_delete_category_reject_predefined() {
    let db = super::get_test_db_pool().await;

    // Get a predefined category
    let categories = list_categories_impl(db, Some(CategoryFilter::Predefined)).await.unwrap();
    let predefined_id = categories[0].id;

    let result = delete_category_impl(db, predefined_id).await;
    assert!(result.is_err(), "Should reject deletion of predefined category");
    let error_msg = result.unwrap_err().to_lowercase();
    assert!(
        error_msg.contains("predefined") || error_msg.contains("delete"),
        "Error should mention predefined protection"
    );
}

#[tokio::test]
async fn test_delete_category_not_found() {
    let db = super::get_test_db_pool().await;

    let result = delete_category_impl(db, 999999).await;
    assert!(result.is_err(), "Should fail for non-existent category");
    assert!(
        result.unwrap_err().to_lowercase().contains("not found"),
        "Error should mention category not found"
    );
}
