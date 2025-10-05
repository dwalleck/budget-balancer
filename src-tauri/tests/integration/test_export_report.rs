use budget_balancer_lib::commands::analytics_commands::export_analytics_report_impl;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn test_export_analytics_report_pdf() {
    let db = super::get_test_db_pool().await;
    let output_path = format!(
        "/tmp/analytics_report_{}.pdf",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let result = export_analytics_report_impl(
        db,
        "pdf",
        "2025-01-01",
        "2025-12-31",
        true,
        &output_path,
    )
    .await;

    assert!(result.is_ok(), "Failed to export analytics report: {:?}", result);

    let response = result.unwrap();
    assert!(response.success, "Export should succeed");
    assert_eq!(response.file_path, output_path);
    assert!(response.file_size > 0, "File size should be greater than 0");

    // Verify file exists
    assert!(PathBuf::from(&output_path).exists(), "Export file should exist");

    // Clean up
    fs::remove_file(output_path).ok();
}

#[tokio::test]
async fn test_export_analytics_report_xlsx() {
    let db = super::get_test_db_pool().await;
    let output_path = format!(
        "/tmp/analytics_report_{}.xlsx",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let result = export_analytics_report_impl(
        db,
        "xlsx",
        "2025-01-01",
        "2025-12-31",
        false,
        &output_path,
    )
    .await;

    assert!(result.is_ok(), "Should export to XLSX");

    let response = result.unwrap();
    assert!(response.success, "XLSX export should succeed");

    // Clean up
    fs::remove_file(output_path).ok();
}

#[tokio::test]
async fn test_export_analytics_report_with_charts() {
    let db = super::get_test_db_pool().await;
    let output_path = format!(
        "/tmp/analytics_with_charts_{}.pdf",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let result = export_analytics_report_impl(
        db,
        "pdf",
        "2025-01-01",
        "2025-03-31",
        true, // Include charts
        &output_path,
    )
    .await;

    assert!(result.is_ok(), "Should export with charts");

    // Clean up
    fs::remove_file(output_path).ok();
}
