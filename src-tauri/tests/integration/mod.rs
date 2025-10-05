// Integration tests for Budget Balancer Tauri commands
// These tests verify the contract/interface of each command

pub mod fixtures;
mod test_account_commands;
mod test_categorize;
mod test_category_commands;
mod test_column_mapping;
mod test_create_target;
mod test_dashboard;
mod test_debt_commands;
mod test_export_report;
mod test_export_transactions;
mod test_import_csv;
mod test_security;
mod test_spending_by_category;
mod test_spending_trends;
mod test_targets_progress;
mod test_transaction_commands;
mod test_update_target;

use sqlx::SqlitePool;
use std::sync::OnceLock;

// Static database pool shared across all tests
static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

// Get or initialize the shared database pool
pub async fn get_test_db_pool() -> &'static SqlitePool {
    if let Some(pool) = DB_POOL.get() {
        return pool;
    }

    // Initialize database
    let pool = initialize_test_database().await.expect("Failed to initialize test database");
    DB_POOL.get_or_init(|| pool)
}

async fn initialize_test_database() -> Result<SqlitePool, String> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    let mut db_path = dirs::data_dir()
        .ok_or_else(|| "Could not find data directory".to_string())?;

    db_path.push("budget-balancer");
    std::fs::create_dir_all(&db_path)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    db_path.push("budget_balancer.db");

    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))
        .map_err(|e| format!("Failed to parse database URL: {}", e))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Failed to run migrations: {}", e))?;

    Ok(pool)
}

// Helper function to generate unique test names
pub fn unique_name(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    format!("{} {}", prefix, timestamp)
}
