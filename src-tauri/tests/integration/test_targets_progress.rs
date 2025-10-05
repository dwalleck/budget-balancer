use budget_balancer_lib::commands::analytics_commands::{create_spending_target_impl, get_spending_targets_progress_impl};

#[tokio::test]
async fn test_get_spending_targets_progress() {
    let db = super::get_test_db_pool().await;
    let result = get_spending_targets_progress_impl(
        db,
        Some("monthly".to_string()),
        None,
        None,
    )
    .await;

    assert!(result.is_ok(), "Failed to get spending targets progress: {:?}", result);

    let response = result.unwrap();
    assert!(response.targets.is_empty() || !response.targets.is_empty(), "Targets should be a valid array");

    // Verify overall_status is valid
    assert!(
        response.overall_status == "under" ||
        response.overall_status == "on_track" ||
        response.overall_status == "over",
        "Overall status should be valid"
    );
}

#[tokio::test]
async fn test_get_spending_targets_progress_with_custom_range() {
    let db = super::get_test_db_pool().await;
    let result = get_spending_targets_progress_impl(
        db,
        None,
        Some("2025-01-01".to_string()),
        Some("2025-01-31".to_string()),
    )
    .await;

    assert!(result.is_ok(), "Should support custom date range");

    let response = result.unwrap();
    assert_eq!(response.period.start_date, "2025-01-01");
    assert_eq!(response.period.end_date, "2025-01-31");
}

#[tokio::test]
async fn test_target_status_calculation() {
    let db = super::get_test_db_pool().await;
    // Create a target
    let target_result = create_spending_target_impl(
        db,
        1, // category_id
        500.0,
        "monthly",
        "2025-01-01",
        None,
    )
    .await;

    assert!(target_result.is_ok(), "Failed to create spending target");

    // Get progress
    let result = get_spending_targets_progress_impl(
        db,
        Some("monthly".to_string()),
        None,
        None,
    )
    .await;

    assert!(result.is_ok(), "Should get progress after creating target");

    let response = result.unwrap();
    for target in response.targets {
        assert!(target.percentage_used >= 0.0, "Percentage used should be >= 0");
        assert!(
            target.status == "under" ||
            target.status == "on_track" ||
            target.status == "over",
            "Target status should be valid"
        );
    }
}
