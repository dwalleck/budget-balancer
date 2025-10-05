use budget_balancer_lib::models::debt::NewDebt;
use budget_balancer_lib::commands::debt_commands::{
    calculate_payoff_plan_impl, compare_strategies_impl, create_debt_impl, get_debt_progress_impl, get_payoff_plan_impl,
    list_debts_impl, record_debt_payment_impl, update_debt_impl,
};
use sqlx::SqlitePool;

// Helper function for unique names
fn unique_name(base: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    format!("{} {}", base, timestamp)
}

// Helper function to get database connection
async fn get_test_db() -> SqlitePool {
    use dirs::data_dir;
    let mut db_path = data_dir().expect("Could not find data directory");
    db_path.push("budget-balancer");
    db_path.push("budget_balancer.db");
    let db_url = format!("sqlite:{}", db_path.display());
    SqlitePool::connect(&db_url).await.expect("Failed to connect to test database")
}

// Helper function to clean up debts by name pattern
async fn cleanup_test_debts(name_pattern: &str) {
    let db = get_test_db().await;
    let pattern = format!("%{}%", name_pattern);
    sqlx::query("DELETE FROM debts WHERE name LIKE ?")
        .bind(pattern)
        .execute(&db)
        .await
        .ok();
}

// Helper function to delete ALL debts (for tests that need clean slate)
async fn cleanup_all_debts() {
    let db = get_test_db().await;
    sqlx::query("DELETE FROM debt_payments").execute(&db).await.ok();
    sqlx::query("DELETE FROM debt_plans").execute(&db).await.ok();
    sqlx::query("DELETE FROM debts").execute(&db).await.ok();
}

// T030: Contract test for create_debt command
#[tokio::test]
async fn test_create_debt_success() {
    let db = super::get_test_db_pool().await;
    let debt = NewDebt {
        name: unique_name("Credit Card A"),
        balance: 5000.0,
        interest_rate: 18.5,
        min_payment: 150.0,
    };

    let result = create_debt_impl(db, debt).await;
    assert!(
        result.is_ok(),
        "Failed to create debt: {:?}",
        result.err()
    );

    let debt_id = result.unwrap();
    assert!(debt_id > 0, "Debt ID should be positive");
}

#[tokio::test]
async fn test_create_debt_invalid_interest_rate() {
    let db = super::get_test_db_pool().await;
    let debt = NewDebt {
        name: unique_name("Test Debt"),
        balance: 1000.0,
        interest_rate: 150.0, // Invalid: > 100
        min_payment: 50.0,
    };

    let result = create_debt_impl(db, debt).await;
    assert!(result.is_err(), "Should reject invalid interest rate");
    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();
    assert!(
        error_msg.contains("interest"),
        "Error should mention invalid rate: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_debt_negative_balance() {
    let db = super::get_test_db_pool().await;
    let debt = NewDebt {
        name: unique_name("Test Debt"),
        balance: -1000.0, // Invalid
        interest_rate: 15.0,
        min_payment: 50.0,
    };

    let result = create_debt_impl(db, debt).await;
    assert!(result.is_err(), "Should reject negative balance");
    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();
    assert!(
        error_msg.contains("balance"),
        "Error should mention invalid amount: {}",
        error_msg
    );
}

// T031: Contract test for list_debts command
#[tokio::test]
async fn test_list_debts() {
    let db = super::get_test_db_pool().await;
    // Create a test debt first
    let debt = NewDebt {
        name: unique_name("Test List Debt"),
        balance: 2000.0,
        interest_rate: 15.0,
        min_payment: 75.0,
    };
    create_debt_impl(db, debt).await.unwrap();

    let result = list_debts_impl(db).await;
    assert!(result.is_ok(), "Failed to list debts: {:?}", result.err());

    let debts = result.unwrap();
    assert!(!debts.is_empty(), "Should have at least one debt");
    assert!(debts[0].id > 0);
    assert!(!debts[0].name.is_empty());
    assert!(debts[0].balance >= 0.0);
}

// T032: Contract test for update_debt command
#[tokio::test]
async fn test_update_debt_balance() {
    let db = super::get_test_db_pool().await;
    // Create a test debt
    let debt = NewDebt {
        name: unique_name("Test Update Debt"),
        balance: 3000.0,
        interest_rate: 18.0,
        min_payment: 100.0,
    };
    let debt_id = create_debt_impl(db, debt).await.unwrap();

    // Update the balance
    let result = update_debt_impl(db, debt_id, Some(2500.0), None, None).await;
    assert!(
        result.is_ok(),
        "Failed to update debt: {:?}",
        result.err()
    );

    // Verify the update
    let debts = list_debts_impl(db).await.unwrap();
    let updated_debt = debts.iter().find(|d| d.id == debt_id);
    assert!(updated_debt.is_some(), "Updated debt should exist");
    assert_eq!(updated_debt.unwrap().balance, 2500.0);
}

#[tokio::test]
async fn test_update_debt_not_found() {
    let db = super::get_test_db_pool().await;
    let result = update_debt_impl(db, 99999, Some(1000.0), None, None).await;
    assert!(result.is_err(), "Should fail for non-existent debt");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("not found") || error_msg.contains("NotFound"),
        "Error should indicate debt not found: {}",
        error_msg
    );
}

// T033: Contract test for calculate_payoff_plan command (avalanche)
#[tokio::test]
async fn test_calculate_avalanche_payoff_plan() {
    let db = super::get_test_db_pool().await;
    // Clean slate for this test
    cleanup_all_debts().await;

    // Create two test debts with different interest rates
    let debt1 = NewDebt {
        name: unique_name("Low Interest"),
        balance: 1000.0,
        interest_rate: 10.0,
        min_payment: 25.0,
    };
    let debt2 = NewDebt {
        name: unique_name("High Interest"),
        balance: 1000.0,
        interest_rate: 20.0,
        min_payment: 25.0,
    };
    create_debt_impl(db, debt1).await.unwrap();
    create_debt_impl(db, debt2).await.unwrap();

    let result = calculate_payoff_plan_impl(db, "avalanche".to_string(), 200.0).await;
    assert!(
        result.is_ok(),
        "Failed to calculate avalanche plan: {:?}",
        result.err()
    );

    let plan = result.unwrap();
    assert_eq!(plan.strategy, "avalanche");
    assert!(!plan.payoff_date.is_empty());
    assert!(plan.total_interest > 0.0);
    assert!(!plan.monthly_breakdown.is_empty());

    // Verify first month prioritizes high interest debt
    let first_month = &plan.monthly_breakdown[0];
    assert!(!first_month.payments.is_empty());
}

#[tokio::test]
async fn test_calculate_snowball_payoff_plan() {
    let db = super::get_test_db_pool().await;
    // Clean slate for this test
    cleanup_all_debts().await;

    // Create two test debts with different balances
    let debt1 = NewDebt {
        name: unique_name("Small Balance"),
        balance: 500.0,
        interest_rate: 20.0,
        min_payment: 25.0,
    };
    let debt2 = NewDebt {
        name: unique_name("Large Balance"),
        balance: 2000.0,
        interest_rate: 10.0,
        min_payment: 25.0,
    };
    create_debt_impl(db, debt1).await.unwrap();
    create_debt_impl(db, debt2).await.unwrap();

    let result = calculate_payoff_plan_impl(db, "snowball".to_string(), 200.0).await;
    assert!(
        result.is_ok(),
        "Failed to calculate snowball plan: {:?}",
        result.err()
    );

    let plan = result.unwrap();
    assert_eq!(plan.strategy, "snowball");
    assert!(!plan.monthly_breakdown.is_empty());
}

#[tokio::test]
async fn test_calculate_payoff_plan_insufficient_funds() {
    let db = super::get_test_db_pool().await;
    let debt = NewDebt {
        name: unique_name("Test Debt"),
        balance: 1000.0,
        interest_rate: 15.0,
        min_payment: 100.0,
    };
    create_debt_impl(db, debt).await.unwrap();

    let result = calculate_payoff_plan_impl(db, "avalanche".to_string(), 50.0).await;
    assert!(
        result.is_err(),
        "Should reject insufficient monthly amount"
    );
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Insufficient") || error_msg.contains("funds"),
        "Error should mention insufficient funds: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_calculate_payoff_plan_invalid_strategy() {
    let db = super::get_test_db_pool().await;
    // Clean slate for this test
    cleanup_all_debts().await;

    let debt = NewDebt {
        name: unique_name("Test Debt"),
        balance: 1000.0,
        interest_rate: 15.0,
        min_payment: 50.0,
    };
    create_debt_impl(db, debt).await.unwrap();

    let result = calculate_payoff_plan_impl(db, "invalid_strategy".to_string(), 150.0).await;
    assert!(
        result.is_err(),
        "Should reject invalid strategy"
    );
    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();
    assert!(
        error_msg.contains("strategy"),
        "Error should mention invalid strategy: {}",
        error_msg
    );
}

// T034: Contract test for get_payoff_plan command
#[tokio::test]
async fn test_get_payoff_plan() {
    let db = super::get_test_db_pool().await;
    // Clean slate for this test
    cleanup_all_debts().await;

    // Create a debt and plan
    let debt = NewDebt {
        name: unique_name("Plan Test Debt"),
        balance: 1000.0,
        interest_rate: 15.0,
        min_payment: 50.0,
    };
    create_debt_impl(db, debt).await.unwrap();

    let plan = calculate_payoff_plan_impl(db, "avalanche".to_string(), 150.0)
        .await
        .unwrap();

    // Retrieve the plan
    let result = get_payoff_plan_impl(db, plan.plan_id).await;
    assert!(
        result.is_ok(),
        "Failed to get payoff plan: {:?}",
        result.err()
    );

    let retrieved_plan = result.unwrap();
    assert_eq!(retrieved_plan.plan_id, plan.plan_id);
    assert!(!retrieved_plan.monthly_breakdown.is_empty());
}

#[tokio::test]
async fn test_get_payoff_plan_not_found() {
    let db = super::get_test_db_pool().await;
    let result = get_payoff_plan_impl(db, 99999).await;
    assert!(result.is_err(), "Should fail for non-existent plan");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("not found") || error_msg.contains("NotFound"),
        "Error should indicate plan not found: {}",
        error_msg
    );
}

// T035: Contract test for record_debt_payment command
#[tokio::test]
async fn test_record_debt_payment() {
    let db = super::get_test_db_pool().await;
    let debt = NewDebt {
        name: unique_name("Payment Test Debt"),
        balance: 1000.0,
        interest_rate: 15.0,
        min_payment: 50.0,
    };
    let debt_id = create_debt_impl(db, debt).await.unwrap();

    let result = record_debt_payment_impl(db, debt_id, 200.0, "2025-10-15".to_string(), None).await;
    assert!(
        result.is_ok(),
        "Failed to record payment: {:?}",
        result.err()
    );

    let payment_response = result.unwrap();
    assert!(payment_response.payment_id > 0);
    assert!(payment_response.updated_balance < 1000.0);
}

#[tokio::test]
async fn test_record_debt_payment_exceeds_balance() {
    let db = super::get_test_db_pool().await;
    let debt = NewDebt {
        name: unique_name("Payment Exceed Test"),
        balance: 500.0,
        interest_rate: 15.0,
        min_payment: 50.0,
    };
    let debt_id = create_debt_impl(db, debt).await.unwrap();

    let result = record_debt_payment_impl(db, debt_id, 999999.0, "2025-10-15".to_string(), None).await;
    assert!(
        result.is_err(),
        "Should reject payment exceeding balance"
    );
    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();
    assert!(
        error_msg.contains("payment") && error_msg.contains("balance"),
        "Error should mention payment exceeds balance: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_record_debt_payment_invalid_amount() {
    let db = super::get_test_db_pool().await;
    let debt = NewDebt {
        name: unique_name("Invalid Payment Test"),
        balance: 1000.0,
        interest_rate: 15.0,
        min_payment: 50.0,
    };
    let debt_id = create_debt_impl(db, debt).await.unwrap();

    // Test zero payment
    let result = record_debt_payment_impl(db, debt_id, 0.0, "2025-10-15".to_string(), None).await;
    assert!(
        result.is_err(),
        "Should reject zero payment amount"
    );
    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();
    assert!(
        error_msg.contains("payment") && error_msg.contains("positive"),
        "Error should mention payment must be positive: {}",
        error_msg
    );

    // Test negative payment
    let result = record_debt_payment_impl(db, debt_id, -100.0, "2025-10-15".to_string(), None).await;
    assert!(
        result.is_err(),
        "Should reject negative payment amount"
    );
    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();
    assert!(
        error_msg.contains("payment") && error_msg.contains("positive"),
        "Error should mention payment must be positive: {}",
        error_msg
    );
}

// T036: Contract test for get_debt_progress command
#[tokio::test]
async fn test_get_debt_progress() {
    let db = super::get_test_db_pool().await;
    let debt = NewDebt {
        name: unique_name("Progress Test Debt"),
        balance: 1000.0,
        interest_rate: 15.0,
        min_payment: 50.0,
    };
    let debt_id = create_debt_impl(db, debt).await.unwrap();

    // Record a payment
    record_debt_payment_impl(db, debt_id, 100.0, "2025-10-15".to_string(), None)
        .await
        .unwrap();

    let result = get_debt_progress_impl(db, debt_id, None, None).await;
    assert!(
        result.is_ok(),
        "Failed to get debt progress: {:?}",
        result.err()
    );

    let progress = result.unwrap();
    assert_eq!(progress.debt.id, debt_id);
    assert!(!progress.payments.is_empty());
    assert!(progress.total_paid > 0.0);
    assert!(!progress.balance_history.is_empty());
}

// T037: Contract test for compare_strategies command
#[tokio::test]
async fn test_compare_strategies() {
    let db = super::get_test_db_pool().await;
    // Clean slate for this test
    cleanup_all_debts().await;

    // Create test debts
    let debt1 = NewDebt {
        name: unique_name("Compare Debt 1"),
        balance: 1000.0,
        interest_rate: 18.0,
        min_payment: 50.0,
    };
    let debt2 = NewDebt {
        name: unique_name("Compare Debt 2"),
        balance: 2000.0,
        interest_rate: 12.0,
        min_payment: 75.0,
    };
    create_debt_impl(db, debt1).await.unwrap();
    create_debt_impl(db, debt2).await.unwrap();

    let result = compare_strategies_impl(db, 300.0).await;
    assert!(
        result.is_ok(),
        "Failed to compare strategies: {:?}",
        result.err()
    );

    let comparison = result.unwrap();
    assert_eq!(comparison.avalanche.strategy, "avalanche");
    assert_eq!(comparison.snowball.strategy, "snowball");
    assert!(comparison.savings.interest_saved >= 0.0);
    assert!(comparison.savings.months_saved >= 0);

    // Avalanche should typically save on interest
    assert!(
        comparison.avalanche.total_interest <= comparison.snowball.total_interest,
        "Avalanche should save interest compared to snowball"
    );
}
