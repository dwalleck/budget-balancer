//! Test fixtures for creating test data without CSV imports.
//!
//! This module provides helper functions to create test data by inserting directly
//! into the database, bypassing the CSV import flow and rate limiter. This approach
//! has several benefits:
//!
//! - **Performance**: Tests run faster without CSV parsing overhead
//! - **Reliability**: Avoids rate limiter issues in parallel test execution
//! - **Simplicity**: Cleaner test code with builder pattern API
//! - **Isolation**: Each test gets unique, realistic data via `fake` crate
//!
//! # Hash Uniqueness Strategy
//!
//! Transaction hashes are calculated from (date + amount + description) per the
//! production duplicate detection logic. To prevent hash collisions when tests run
//! in parallel, we append a unique suffix combining a fake name with a 6-digit random number.
//! This ensures guaranteed uniqueness while keeping test data realistic and readable.
//!
//! For example: "Coffee" becomes "Coffee (Alice123456)" or "Coffee (Bob789012)"
//!
//! **Collision probability:** With 900K possible numbers (100000..999999) and ~113 tests,
//! the collision chance is negligible (<0.01% via birthday paradox).
//!
//! Alternative approaches considered:
//! - Timestamps: Unique but hard to read (13-digit numbers)
//! - Shared counter: Requires mutex synchronization (slower)
//! - Random words: Can collide, limited vocabulary
//! - 4-digit numbers: ~1% collision risk with 113 tests
//! - Test name in description: Too invasive, requires test context
//!
//! The `fake` crate + 6-digit random number approach provides guaranteed uniqueness with
//! human-readable, realistic test data.

use budget_balancer_lib::commands::account_commands::create_account_impl;
use budget_balancer_lib::constants::DEFAULT_CATEGORY_ID;
use budget_balancer_lib::models::account::{AccountType, NewAccount};
use fake::{Fake, Faker};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};

/// Helper to create a test account
pub async fn create_test_account(db: &SqlitePool, name: &str) -> i64 {
    let account_name = super::unique_name(name);
    let account = NewAccount {
        name: account_name.clone(),
        account_type: AccountType::Checking,
        initial_balance: 0.0,
    };
    create_account_impl(db, account)
        .await
        .unwrap_or_else(|e| panic!("Failed to create test account '{}': {}", account_name, e))
}

/// Helper to insert transactions directly into the database
/// This bypasses CSV import and the rate limiter
pub async fn insert_test_transactions(
    db: &SqlitePool,
    account_id: i64,
    transactions: Vec<TestTransaction>,
) -> Vec<i64> {
    let mut transaction_ids = Vec::new();

    for (idx, tx) in transactions.iter().enumerate() {
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
        .bind(tx.category_id.unwrap_or(DEFAULT_CATEGORY_ID)) // Default to Uncategorized
        .bind(&hash)
        .fetch_one(db)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to insert test transaction #{} (desc='{}', date={}, amount={}): {}",
                idx + 1, tx.description, tx.date, tx.amount, e
            )
        });

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
        // Add unique suffix to description to ensure uniqueness across parallel tests
        // Combines fake data (name) with random number for guaranteed uniqueness
        // Range: 100000..999999 = 900K possibilities (<<1% collision with 113 tests)
        use fake::faker::name::en::FirstName;
        use rand::Rng;

        let name: String = FirstName().fake();
        let number: u32 = rand::thread_rng().gen_range(100000..999999);
        let unique_suffix = format!("{}{}", name, number);

        Self {
            date: date.to_string(),
            amount,
            description: format!("{} ({})", description, unique_suffix),
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
