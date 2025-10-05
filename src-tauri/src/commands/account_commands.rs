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
    // Use match statement with static SQL for all 7 combinations
    // This prevents any possibility of SQL injection from future modifications
    let result = match (&update.name, &update.account_type, update.balance) {
        // All three fields
        (Some(name), Some(account_type), Some(balance)) => {
            sqlx::query(
                "UPDATE accounts SET name = ?, type = ?, balance = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(name)
            .bind(account_type.to_string())
            .bind(balance)
            .bind(update.id)
            .execute(db)
            .await
        }
        // Two fields: name + type
        (Some(name), Some(account_type), None) => {
            sqlx::query(
                "UPDATE accounts SET name = ?, type = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(name)
            .bind(account_type.to_string())
            .bind(update.id)
            .execute(db)
            .await
        }
        // Two fields: name + balance
        (Some(name), None, Some(balance)) => {
            sqlx::query(
                "UPDATE accounts SET name = ?, balance = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(name)
            .bind(balance)
            .bind(update.id)
            .execute(db)
            .await
        }
        // Two fields: type + balance
        (None, Some(account_type), Some(balance)) => {
            sqlx::query(
                "UPDATE accounts SET type = ?, balance = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(account_type.to_string())
            .bind(balance)
            .bind(update.id)
            .execute(db)
            .await
        }
        // Single field: name only
        (Some(name), None, None) => {
            sqlx::query(
                "UPDATE accounts SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(name)
            .bind(update.id)
            .execute(db)
            .await
        }
        // Single field: type only
        (None, Some(account_type), None) => {
            sqlx::query(
                "UPDATE accounts SET type = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(account_type.to_string())
            .bind(update.id)
            .execute(db)
            .await
        }
        // Single field: balance only
        (None, None, Some(balance)) => {
            sqlx::query(
                "UPDATE accounts SET balance = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(balance)
            .bind(update.id)
            .execute(db)
            .await
        }
        // No fields provided
        (None, None, None) => {
            return Err("At least one field must be provided for update".to_string());
        }
    }
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
    // Wrap all operations in a transaction to ensure atomicity
    let mut tx = db.begin()
        .await
        .map_err(|e| sanitize_db_error(e, "begin transaction"))?;

    // First, check if account exists
    let exists = sqlx::query("SELECT id FROM accounts WHERE id = ?")
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| sanitize_db_error(e, "check account exists"))?;

    if exists.is_none() {
        return Err(format!("Account with id {} not found", account_id));
    }

    // Count transactions that will be deleted (for reporting)
    let count_result = sqlx::query("SELECT COUNT(*) as count FROM transactions WHERE account_id = ?")
        .bind(account_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| sanitize_db_error(e, "count transactions"))?;

    let transaction_count: i64 = count_result.get("count");

    // Delete the account (CASCADE will delete associated transactions)
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| sanitize_db_error(e, "delete account"))?;

    // Commit the transaction
    tx.commit()
        .await
        .map_err(|e| sanitize_db_error(e, "commit transaction"))?;

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
