use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::commands::category_commands::create_category_impl;
use budget_balancer_lib::commands::csv_commands::{import_csv_impl, reset_rate_limiter};
use budget_balancer_lib::models::account::NewAccount;
use budget_balancer_lib::models::category::NewCategory;
use budget_balancer_lib::services::csv_parser::ColumnMapping;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_categorize_transaction_with_matching_rule() {
    reset_rate_limiter();
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Categorize Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import a transaction with merchant "Starbucks"
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

    // Sleep to ensure rate limiter window passes before next test
    tokio::time::sleep(tokio::time::Duration::from_millis(super::RATE_LIMITER_DELAY_MS)).await;

    // Get the transaction ID (should be the first one for this account)
    // Note: We need a way to get transactions - this assumes list_transactions exists
    // For now, we'll assume transaction_id = 1 for the test
    // In a real implementation, we'd query the database to get the actual ID

    // TODO: This test needs the list_transactions command to be fully functional
    // For now, we're testing the contract but can't fully verify without querying
}

#[tokio::test]
#[serial]
async fn test_categorize_transaction_no_rule_match() {
    reset_rate_limiter();
    tokio::time::sleep(tokio::time::Duration::from_millis(super::RATE_LIMITER_DELAY_MS)).await;
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("No Match Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Import a transaction with an unknown merchant
    let csv_content = "Date,Amount,Description,Merchant\n2024-01-01,-50.00,Something,Unknown Merchant XYZ";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

    // Sleep to ensure rate limiter window passes before next test
    tokio::time::sleep(tokio::time::Duration::from_millis(super::RATE_LIMITER_DELAY_MS)).await;

    // Test categorization - should assign to "Uncategorized"
    // TODO: Similar to above, needs transaction ID from list_transactions
}

#[tokio::test]
#[serial]
async fn test_categorize_transaction_custom_category() {
    reset_rate_limiter();
    tokio::time::sleep(tokio::time::Duration::from_millis(super::RATE_LIMITER_DELAY_MS)).await;
    let db = super::get_test_db_pool().await;
    // Create test account
    let account = NewAccount {
        name: super::unique_name("Custom Cat Test"),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Create a custom category
    let category = NewCategory {
        name: super::unique_name("Test Category"),
        icon: Some("🎯".to_string()),
    };
    let _category_id = create_category_impl(db, category)
        .await
        .expect("Failed to create category");

    // Import transaction
    let csv_content = "Date,Amount,Description,Merchant\n2024-01-01,-50.00,Test,Test Merchant";
    let mapping = ColumnMapping {
        date: "Date".to_string(),
        amount: "Amount".to_string(),
        description: "Description".to_string(),
        merchant: Some("Merchant".to_string()),
    };

    import_csv_impl(db, account_id, csv_content.to_string(), mapping)
        .await
        .expect("Failed to import CSV");

    // TODO: Test categorization with custom category rule
}
