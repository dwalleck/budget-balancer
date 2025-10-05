// Test fixtures for creating test data without CSV imports
// This bypasses the rate limiter and makes tests run faster

use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::models::account::{AccountType, NewAccount};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};

/// Helper to create a test account
pub async fn create_test_account(db: &SqlitePool, name: &str) -> i64 {
    let account = NewAccount {
        name: super::unique_name(name),
        account_type: AccountType::Checking,
        initial_balance: 0.0,
    };
    create_account_impl(db, account)
        .await
        .expect("Failed to create test account")
}

/// Helper to insert transactions directly into the database
/// This bypasses CSV import and the rate limiter
pub async fn insert_test_transactions(
    db: &SqlitePool,
    account_id: i64,
    transactions: Vec<TestTransaction>,
) -> Vec<i64> {
    let mut transaction_ids = Vec::new();

    for tx in transactions {
        // Calculate hash (same logic as CSV import)
        let hash = calculate_transaction_hash(&tx.date, tx.amount, &tx.description);

        let result = sqlx::query(
            "INSERT INTO transactions (account_id, date, amount, description, merchant, category_id, hash)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             RETURNING id"
        )
        .bind(account_id)
        .bind(&tx.date)
        .bind(tx.amount)
        .bind(&tx.description)
        .bind(&tx.merchant)
        .bind(tx.category_id.unwrap_or(10)) // Default to Uncategorized
        .bind(&hash)
        .fetch_one(db)
        .await
        .expect("Failed to insert test transaction");

        let id: i64 = result.get(0);
        transaction_ids.push(id);
    }

    transaction_ids
}

/// Calculate transaction hash (same logic as NewTransaction::calculate_hash)
fn calculate_transaction_hash(date: &str, amount: f64, description: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}", date, amount, description));
    format!("{:x}", hasher.finalize())
}

/// Struct for defining test transactions
#[derive(Clone)]
pub struct TestTransaction {
    pub date: String,
    pub amount: f64,
    pub description: String,
    pub merchant: Option<String>,
    pub category_id: Option<i64>,
}

impl TestTransaction {
    pub fn new(date: &str, amount: f64, description: &str) -> Self {
        // Add microsecond timestamp to description to ensure uniqueness across parallel tests
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();

        Self {
            date: date.to_string(),
            amount,
            description: format!("{} [{}]", description, timestamp),
            merchant: None,
            category_id: None,
        }
    }

    pub fn with_merchant(mut self, merchant: &str) -> Self {
        self.merchant = Some(merchant.to_string());
        self
    }

    pub fn with_category(mut self, category_id: i64) -> Self {
        self.category_id = Some(category_id);
        self
    }
}

/// Quick helper to create a single transaction
pub async fn insert_single_transaction(
    db: &SqlitePool,
    account_id: i64,
    date: &str,
    amount: f64,
    description: &str,
) -> i64 {
    let tx = TestTransaction::new(date, amount, description);
    insert_test_transactions(db, account_id, vec![tx])
        .await
        .into_iter()
        .next()
        .unwrap()
}
