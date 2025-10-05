use budget_balancer_lib::commands::analytics_commands::{create_spending_target_impl, update_spending_target_impl};
use budget_balancer_lib::commands::category_commands::create_category_impl;
use budget_balancer_lib::models::category::NewCategory;

#[tokio::test]
async fn test_update_spending_target_amount() {
    let db = super::get_test_db_pool().await;
    // Create a category
    let category = NewCategory {
        name: super::unique_name("Update Target Category"),
        icon: Some("🎯".to_string()),
    };
    let category_id = create_category_impl(db, category)
        .await
        .expect("Failed to create category");

    // Create spending target
    let target_id = create_spending_target_impl(
        db,
        category_id,
        500.0,
        "monthly",
        "2025-01-01",
        None,
    )
    .await
    .expect("Failed to create target");

    // Update target amount
    let result = update_spending_target_impl(
        db,
        target_id,
        Some(600.0),
        None,
    )
    .await;

    assert!(result.is_ok(), "Failed to update spending target: {:?}", result);

    let response = result.unwrap();
    assert!(response.success, "Update should succeed");
}

#[tokio::test]
async fn test_update_spending_target_end_date() {
    let db = super::get_test_db_pool().await;
    // Create a category
    let category = NewCategory {
        name: super::unique_name("Update End Date Category"),
        icon: Some("📆".to_string()),
    };
    let category_id = create_category_impl(db, category)
        .await
        .expect("Failed to create category");

    // Create spending target
    let target_id = create_spending_target_impl(
        db,
        category_id,
        500.0,
        "monthly",
        "2025-01-01",
        None,
    )
    .await
    .expect("Failed to create target");

    // Update end date
    let result = update_spending_target_impl(
        db,
        target_id,
        None,
        Some("2025-06-30"),
    )
    .await;

    assert!(result.is_ok(), "Should update end date");
}

#[tokio::test]
async fn test_update_nonexistent_target() {
    let db = super::get_test_db_pool().await;
    let result = update_spending_target_impl(
        db,
        99999, // Non-existent ID
        Some(700.0),
        None,
    )
    .await;

    assert!(result.is_err(), "Should fail for non-existent target");
}
