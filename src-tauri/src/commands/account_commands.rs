use crate::models::account::{Account, NewAccount};
use crate::DbPool;
use sqlx::SqlitePool;

// Business logic functions (used by both commands and tests)

pub async fn list_accounts_impl(db: &SqlitePool) -> Result<Vec<Account>, String> {
    sqlx::query_as::<_, Account>(
        "SELECT id, name, type, balance, created_at, updated_at FROM accounts ORDER BY name"
    )
    .fetch_all(db)
    .await
    .map_err(|e| {
        eprintln!("Database error loading accounts: {}", e);
        "Failed to load accounts".to_string()
    })
}

pub async fn create_account_impl(
    db: &SqlitePool,
    account: NewAccount,
) -> Result<i64, String> {
    let result = sqlx::query(
        "INSERT INTO accounts (name, type, balance) VALUES (?, ?, ?)"
    )
    .bind(&account.name)
    .bind(account.account_type.to_string())
    .bind(account.initial_balance)
    .execute(db)
    .await
    .map_err(|e| {
        eprintln!("Database error creating account: {}", e);
        "Failed to create account".to_string()
    })?;

    Ok(result.last_insert_rowid())
}

// Tauri command handlers (extract pool from managed state)

#[tauri::command]
pub async fn list_accounts(db_pool: tauri::State<'_, DbPool>) -> Result<Vec<Account>, String> {
    list_accounts_impl(&db_pool.0).await
}

#[tauri::command]
pub async fn create_account(
    db_pool: tauri::State<'_, DbPool>,
    account: NewAccount,
) -> Result<i64, String> {
    create_account_impl(&db_pool.0, account).await
}
