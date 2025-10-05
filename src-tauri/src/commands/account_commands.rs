use crate::errors::sanitize_db_error;
use crate::models::account::{Account, NewAccount, UpdateAccount};
use crate::DbPool;
use sqlx::{Row, SqlitePool};

// Business logic functions (used by both commands and tests)

pub async fn list_accounts_impl(db: &SqlitePool) -> Result<Vec<Account>, String> {
    sqlx::query_as::<_, Account>(
        "SELECT id, name, type, balance, created_at, updated_at FROM accounts ORDER BY name"
    )
    .fetch_all(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load accounts"))
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
    .map_err(|e| sanitize_db_error(e, "create account"))?;

    Ok(result.last_insert_rowid())
}

pub async fn update_account_impl(
    db: &SqlitePool,
    update: UpdateAccount,
) -> Result<Account, String> {
    // Build dynamic UPDATE query based on which fields are provided
    let mut query_parts = Vec::new();

    if update.name.is_some() {
        query_parts.push("name = ?");
    }
    if update.account_type.is_some() {
        query_parts.push("type = ?");
    }
    if update.balance.is_some() {
        query_parts.push("balance = ?");
    }

    if query_parts.is_empty() {
        return Err("At least one field must be provided for update".to_string());
    }

    // Add updated_at timestamp
    query_parts.push("updated_at = CURRENT_TIMESTAMP");

    let sql = format!(
        "UPDATE accounts SET {} WHERE id = ?",
        query_parts.join(", ")
    );

    // Bind parameters in order
    let mut query = sqlx::query(&sql);
    if let Some(ref name) = update.name {
        query = query.bind(name);
    }
    if let Some(ref account_type) = update.account_type {
        query = query.bind(account_type.to_string());
    }
    if let Some(balance) = update.balance {
        query = query.bind(balance);
    }
    query = query.bind(update.id);

    let result = query
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "update account"))?;

    if result.rows_affected() == 0 {
        return Err(format!("Account with id {} not found", update.id));
    }

    // Fetch and return the updated account
    sqlx::query_as::<_, Account>(
        "SELECT id, name, type, balance, created_at, updated_at FROM accounts WHERE id = ?"
    )
    .bind(update.id)
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch updated account"))
}

pub async fn delete_account_impl(
    db: &SqlitePool,
    account_id: i64,
) -> Result<i64, String> {
    // First, check if account exists
    let exists = sqlx::query("SELECT id FROM accounts WHERE id = ?")
        .bind(account_id)
        .fetch_optional(db)
        .await
        .map_err(|e| sanitize_db_error(e, "check account exists"))?;

    if exists.is_none() {
        return Err(format!("Account with id {} not found", account_id));
    }

    // Count transactions that will be deleted (for reporting)
    let count_result = sqlx::query("SELECT COUNT(*) as count FROM transactions WHERE account_id = ?")
        .bind(account_id)
        .fetch_one(db)
        .await
        .map_err(|e| sanitize_db_error(e, "count transactions"))?;

    let transaction_count: i64 = count_result.get("count");

    // Delete the account (CASCADE will delete associated transactions)
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "delete account"))?;

    Ok(transaction_count)
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

#[tauri::command]
pub async fn update_account(
    db_pool: tauri::State<'_, DbPool>,
    update: UpdateAccount,
) -> Result<Account, String> {
    update_account_impl(&db_pool.0, update).await
}

#[tauri::command]
pub async fn delete_account(
    db_pool: tauri::State<'_, DbPool>,
    account_id: i64,
) -> Result<i64, String> {
    delete_account_impl(&db_pool.0, account_id).await
}
