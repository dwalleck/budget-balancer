// Budget Balancer - Tauri Application
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod constants;
pub mod db;
pub mod models;
pub mod services;
pub mod commands;
pub mod utils;

use sqlx::SqlitePool;
use tauri::Manager;

// Managed state for database pool
pub struct DbPool(pub SqlitePool);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize database with migrations at app startup
            tauri::async_runtime::block_on(async {
                match initialize_database().await {
                    Ok(pool) => {
                        // Store pool in managed state
                        app.manage(DbPool(pool));
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize database: {}", e);
                        Err(e.into())
                    }
                }
            })
        })
        .invoke_handler(tauri::generate_handler![
            commands::csv_commands::get_csv_headers,
            commands::csv_commands::import_csv,
            commands::csv_commands::save_column_mapping,
            commands::transaction_commands::list_transactions,
            commands::transaction_commands::update_transaction_category,
            commands::transaction_commands::categorize_transaction,
            commands::transaction_commands::export_transactions,
            commands::category_commands::list_categories,
            commands::category_commands::create_category,
            commands::account_commands::list_accounts,
            commands::account_commands::create_account,
            commands::debt_commands::create_debt,
            commands::debt_commands::list_debts,
            commands::debt_commands::update_debt,
            commands::debt_commands::calculate_payoff_plan,
            commands::debt_commands::get_payoff_plan,
            commands::debt_commands::record_debt_payment,
            commands::debt_commands::get_debt_progress,
            commands::debt_commands::compare_strategies,
            commands::analytics_commands::get_spending_by_category,
            commands::analytics_commands::get_spending_trends,
            commands::analytics_commands::get_spending_targets_progress,
            commands::analytics_commands::create_spending_target,
            commands::analytics_commands::update_spending_target,
            commands::analytics_commands::get_dashboard_summary,
            commands::analytics_commands::export_analytics_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn initialize_database() -> Result<SqlitePool, String> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    // Get database path in app data directory
    let mut db_path = dirs::data_dir()
        .ok_or_else(|| "Could not find data directory".to_string())?;

    db_path.push("budget-balancer");
    std::fs::create_dir_all(&db_path)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    db_path.push("budget_balancer.db");

    println!("Initializing database at: {}", db_path.display());

    // Create connection options with create_if_missing
    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))
        .map_err(|e| format!("Failed to parse database URL: {}", e))?
        .create_if_missing(true);

    // Create connection pool
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

    println!("Database initialized successfully");
    Ok(pool)
}
