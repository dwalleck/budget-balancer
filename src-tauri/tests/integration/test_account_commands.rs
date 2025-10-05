use budget_balancer_lib::commands::account_commands::{
    create_account_impl, delete_account_impl, list_accounts_impl, update_account_impl,
};
use budget_balancer_lib::models::account::{NewAccount, UpdateAccount};
use sqlx::Row;

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

// T022: Contract test for update_account command
#[tokio::test]
async fn test_update_account_name() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    // Create an account first
    let account = NewAccount {
        name: format!("Old Name {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 100.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Update the account name
    let update = UpdateAccount {
        id: account_id,
        name: Some(format!("New Name {}", timestamp)),
        account_type: None,
        balance: None,
    };

    let result = update_account_impl(db, update).await;
    assert!(result.is_ok(), "Failed to update account: {:?}", result);

    // Verify the update
    let accounts = list_accounts_impl(db).await.expect("Failed to list accounts");
    let updated = accounts.iter().find(|a| a.id == account_id).expect("Account not found");
    assert_eq!(updated.name, format!("New Name {}", timestamp));
    assert_eq!(updated.account_type, "checking"); // Unchanged
}

#[tokio::test]
async fn test_update_account_balance() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    // Create an account
    let account = NewAccount {
        name: format!("Balance Test {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 100.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Update the balance
    let update = UpdateAccount {
        id: account_id,
        name: None,
        account_type: None,
        balance: Some(500.0),
    };

    let result = update_account_impl(db, update).await;
    assert!(result.is_ok(), "Failed to update balance: {:?}", result);

    // Verify the update
    let accounts = list_accounts_impl(db).await.expect("Failed to list accounts");
    let updated = accounts.iter().find(|a| a.id == account_id).expect("Account not found");
    assert_eq!(updated.balance, 500.0);
}

#[tokio::test]
async fn test_update_account_type() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    // Create an account
    let account = NewAccount {
        name: format!("Type Test {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Update the account type
    let update = UpdateAccount {
        id: account_id,
        name: None,
        account_type: Some(budget_balancer_lib::models::account::AccountType::Savings),
        balance: None,
    };

    let result = update_account_impl(db, update).await;
    assert!(result.is_ok(), "Failed to update account type: {:?}", result);

    // Verify the update
    let accounts = list_accounts_impl(db).await.expect("Failed to list accounts");
    let updated = accounts.iter().find(|a| a.id == account_id).expect("Account not found");
    assert_eq!(updated.account_type, "savings");
}

#[tokio::test]
async fn test_update_account_nonexistent() {
    let db = super::get_test_db_pool().await;

    let update = UpdateAccount {
        id: 999999, // Non-existent ID
        name: Some("New Name".to_string()),
        account_type: None,
        balance: None,
    };

    let result = update_account_impl(db, update).await;
    assert!(result.is_err(), "Should fail for non-existent account");
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("not found") || error_msg.contains("Account"));
}

// T023: Contract test for delete_account with cascade
#[tokio::test]
async fn test_delete_account_with_no_transactions() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    // Create an account
    let account = NewAccount {
        name: format!("To Delete {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 0.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Delete the account
    let result = delete_account_impl(db, account_id).await;
    assert!(result.is_ok(), "Failed to delete account: {:?}", result);

    let deleted_count = result.unwrap();
    assert_eq!(deleted_count, 0, "Should have deleted 0 transactions");

    // Verify account no longer exists
    let accounts = list_accounts_impl(db).await.expect("Failed to list accounts");
    assert!(!accounts.iter().any(|a| a.id == account_id), "Account should be deleted");
}

#[tokio::test]
async fn test_delete_account_cascade_transactions() {
    let db = super::get_test_db_pool().await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

    // Create an account
    let account = NewAccount {
        name: format!("Cascade Delete {}", timestamp),
        account_type: budget_balancer_lib::models::account::AccountType::Checking,
        initial_balance: 1000.0,
    };
    let account_id = create_account_impl(db, account).await.expect("Failed to create account");

    // Create some transactions for this account
    // Note: This test requires transaction_commands to be implemented
    // For now, we'll insert transactions directly
    let tx_count = sqlx::query(
        "INSERT INTO transactions (account_id, category_id, date, amount, description, hash)
         VALUES (?, 1, '2025-01-01', -50.0, 'Test Transaction 1', ?),
                (?, 1, '2025-01-02', -75.0, 'Test Transaction 2', ?)"
    )
    .bind(account_id)
    .bind(format!("hash1_{}", timestamp))
    .bind(account_id)
    .bind(format!("hash2_{}", timestamp))
    .execute(db)
    .await
    .expect("Failed to insert test transactions");

    assert_eq!(tx_count.rows_affected(), 2, "Should have inserted 2 transactions");

    // Delete the account
    let result = delete_account_impl(db, account_id).await;
    assert!(result.is_ok(), "Failed to delete account: {:?}", result);

    let deleted_count = result.unwrap();
    assert_eq!(deleted_count, 2, "Should have cascaded 2 transactions");

    // Verify account no longer exists
    let accounts = list_accounts_impl(db).await.expect("Failed to list accounts");
    assert!(!accounts.iter().any(|a| a.id == account_id), "Account should be deleted");

    // Verify transactions are also deleted
    let remaining_txs = sqlx::query("SELECT COUNT(*) as count FROM transactions WHERE account_id = ?")
        .bind(account_id)
        .fetch_one(db)
        .await
        .expect("Failed to query transactions");

    let count: i64 = remaining_txs.get("count");
    assert_eq!(count, 0, "Transactions should be cascaded");
}

#[tokio::test]
async fn test_delete_account_nonexistent() {
    let db = super::get_test_db_pool().await;

    let result = delete_account_impl(db, 999999).await;
    assert!(result.is_err(), "Should fail for non-existent account");
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("not found") || error_msg.contains("Account"));
}
