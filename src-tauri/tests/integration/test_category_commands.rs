use budget_balancer_lib::commands::category_commands::{create_category_impl, list_categories_impl};
use budget_balancer_lib::models::category::NewCategory;

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
    assert!(result.is_ok(), "Failed to create category without description");
}

#[tokio::test]
async fn test_list_categories() {
    let db = super::get_test_db_pool().await;
    // Create a test category first
    let category = NewCategory {
        name: super::unique_name("List Test Category"),
        icon: Some("📊".to_string()),
    };

    let _ = create_category_impl(db, category).await.expect("Failed to create category");

    let result = list_categories_impl(db).await;
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
    let categories = list_categories_impl(db).await.expect("Failed to list categories");

    // Should have seeded categories like Food, Transportation, etc.
    let category_names: Vec<String> = categories.iter().map(|c| c.name.clone()).collect();

    assert!(
        category_names.iter().any(|n| n.contains("Food") || n.contains("Groceries")),
        "Should have food-related category from seed data"
    );
}

#[tokio::test]
async fn test_list_categories_ordered_by_name() {
    let db = super::get_test_db_pool().await;
    let categories = list_categories_impl(db).await.expect("Failed to list categories");

    // Verify categories are ordered by name
    for i in 0..categories.len().saturating_sub(1) {
        assert!(
            categories[i].name <= categories[i + 1].name,
            "Categories should be ordered by name"
        );
    }
}
