use budget_balancer_lib::commands::account_commands::{create_account_impl, list_accounts_impl};
use budget_balancer_lib::models::account::NewAccount;

#[tokio::test]
async fn test_create_account_checking() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    let account = NewAccount {
        name: format!("Test Checking {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 1000.0,
    };

    let result = create_account_impl(db, account).await;
    assert!(result.is_ok(), "Failed to create checking account: {:?}", result);

    let account_id = result.unwrap();
    assert!(account_id > 0, "Account ID should be positive");
}

#[tokio::test]
async fn test_create_account_savings() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    let account = NewAccount {
        name: format!("Test Savings {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Savings,
        initial_balance: 5000.0,
    };

    let result = create_account_impl(db, account).await;
    assert!(result.is_ok(), "Failed to create savings account");
}

#[tokio::test]
async fn test_create_account_credit_card() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    let account = NewAccount {
        name: format!("Test Credit Card {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::CreditCard,
        initial_balance: -500.0,
    };

    let result = create_account_impl(db, account).await;
    assert!(result.is_ok(), "Failed to create credit card account");
}

#[tokio::test]
async fn test_list_accounts() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    // Create a test account first
    let account = NewAccount {
        name: format!("List Test Account {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 100.0,
    };

    let _ = create_account_impl(db, account).await.expect("Failed to create account");

    let result = list_accounts_impl(db).await;
    assert!(result.is_ok(), "Failed to list accounts: {:?}", result);

    let accounts = result.unwrap();
    assert!(!accounts.is_empty(), "Should have at least one account");

    // Verify account structure
    let first = &accounts[0];
    assert!(first.id > 0);
    assert!(!first.name.is_empty());
}

#[tokio::test]
async fn test_list_accounts_ordered_by_name() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    // Create accounts in non-alphabetical order
    let account_b = NewAccount {
        name: format!("B Account {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 100.0,
    };
    let account_a = NewAccount {
        name: format!("A Account {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Savings,
        initial_balance: 200.0,
    };

    create_account_impl(db, account_b).await.expect("Failed to create account B");
    create_account_impl(db, account_a).await.expect("Failed to create account A");

    let accounts = list_accounts_impl(db).await.expect("Failed to list accounts");

    // Verify accounts are ordered by name
    for i in 0..accounts.len().saturating_sub(1) {
        assert!(
            accounts[i].name <= accounts[i + 1].name,
            "Accounts should be ordered by name"
        );
    }
}
