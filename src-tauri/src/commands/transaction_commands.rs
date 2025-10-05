use crate::constants::{DEFAULT_CATEGORY_ID, DEFAULT_OFFSET, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::errors::sanitize_db_error;
use crate::models::transaction::Transaction;
use crate::services::categorizer::Categorizer;
use crate::DbPool;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionFilter {
    pub account_id: Option<i64>,
    pub category_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Business logic functions (used by both commands and tests)

pub async fn list_transactions_impl(
    db: &SqlitePool,
    filter: Option<TransactionFilter>,
) -> Result<Vec<Transaction>, String> {

    let mut query = String::from(
        "SELECT id, account_id, category_id, date, amount, description, merchant, hash, created_at FROM transactions WHERE 1=1"
    );

    let filter = filter.unwrap_or(TransactionFilter {
        account_id: None,
        category_id: None,
        start_date: None,
        end_date: None,
        limit: Some(DEFAULT_PAGE_SIZE),
        offset: Some(DEFAULT_OFFSET),
    });

    // ALWAYS enforce pagination defaults and maximum page size
    // This prevents returning all transactions at once, which could cause performance issues
    let limit = filter
        .limit
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .min(MAX_PAGE_SIZE);
    let offset = filter.offset.unwrap_or(DEFAULT_OFFSET);

    if filter.account_id.is_some() {
        query.push_str(" AND account_id = ?");
    }
    if filter.category_id.is_some() {
        query.push_str(" AND category_id = ?");
    }
    if filter.start_date.is_some() {
        query.push_str(" AND date >= ?");
    }
    if filter.end_date.is_some() {
        query.push_str(" AND date <= ?");
    }

    query.push_str(" ORDER BY date DESC");

    // ALWAYS add LIMIT and OFFSET for pagination
    query.push_str(" LIMIT ? OFFSET ?");

    let mut query_builder = sqlx::query_as::<_, Transaction>(&query);

    if let Some(account_id) = filter.account_id {
        query_builder = query_builder.bind(account_id);
    }
    if let Some(category_id) = filter.category_id {
        query_builder = query_builder.bind(category_id);
    }
    if let Some(start_date) = filter.start_date {
        query_builder = query_builder.bind(start_date);
    }
    if let Some(end_date) = filter.end_date {
        query_builder = query_builder.bind(end_date);
    }

    // Bind limit and offset (always present now)
    query_builder = query_builder.bind(limit).bind(offset);

    query_builder
        .fetch_all(db)
        .await
        .map_err(|e| {
            eprintln!("Database error loading transactions: {}", e);
            "Failed to load transactions".to_string()
        })
}

pub async fn count_transactions_impl(
    db: &SqlitePool,
    filter: Option<TransactionFilter>,
) -> Result<i64, String> {
    let mut query = String::from("SELECT COUNT(*) FROM transactions WHERE 1=1");

    let filter = filter.unwrap_or(TransactionFilter {
        account_id: None,
        category_id: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    });

    // Apply same filters as list_transactions_impl (except limit/offset)
    if filter.account_id.is_some() {
        query.push_str(" AND account_id = ?");
    }
    if filter.category_id.is_some() {
        query.push_str(" AND category_id = ?");
    }
    if filter.start_date.is_some() {
        query.push_str(" AND date >= ?");
    }
    if filter.end_date.is_some() {
        query.push_str(" AND date <= ?");
    }

    let mut query_builder = sqlx::query_as::<_, (i64,)>(&query);

    if let Some(account_id) = filter.account_id {
        query_builder = query_builder.bind(account_id);
    }
    if let Some(category_id) = filter.category_id {
        query_builder = query_builder.bind(category_id);
    }
    if let Some(start_date) = filter.start_date {
        query_builder = query_builder.bind(start_date);
    }
    if let Some(end_date) = filter.end_date {
        query_builder = query_builder.bind(end_date);
    }

    query_builder
        .fetch_one(db)
        .await
        .map(|(count,)| count)
        .map_err(|e| {
            eprintln!("Database error counting transactions: {}", e);
            "Failed to count transactions".to_string()
        })
}

pub async fn update_transaction_category_impl(
    db: &SqlitePool,
    transaction_id: i64,
    category_id: i64,
) -> Result<(), String> {
    sqlx::query("UPDATE transactions SET category_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(category_id)
        .bind(transaction_id)
        .execute(db)
        .await
        .map_err(|e| {
            eprintln!("Database error updating transaction category: {}", e);
            "Failed to update transaction category".to_string()
        })?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct CategorizeResult {
    pub category_id: i64,
    pub matched_rule_id: Option<i64>,
}

pub async fn categorize_transaction_impl(
    db: &SqlitePool,
    transaction_id: i64,
) -> Result<CategorizeResult, String> {
    // Get the transaction
    let transaction = sqlx::query_as::<_, Transaction>(
        "SELECT id, account_id, category_id, date, amount, description, merchant, hash, created_at
         FROM transactions WHERE id = ?"
    )
    .bind(transaction_id)
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load transaction for categorization"))?;

    // Use categorizer to find best category
    let category_id = Categorizer::categorize(
        db,
        transaction.merchant.as_deref(),
        &transaction.description,
    )
    .await
    .map_err(|e| {
        eprintln!("Categorization error: {}", e);
        "Failed to categorize transaction".to_string()
    })?
    .unwrap_or(DEFAULT_CATEGORY_ID); // Default to "Uncategorized"

    // Update the transaction with new category
    sqlx::query("UPDATE transactions SET category_id = ? WHERE id = ?")
        .bind(category_id)
        .bind(transaction_id)
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "update transaction category"))?;

    Ok(CategorizeResult {
        category_id,
        matched_rule_id: None, // TODO: Return actual matched rule ID
    })
}

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub success: bool,
    pub file_path: String,
    pub record_count: usize,
}

pub async fn export_transactions_impl(
    db: &SqlitePool,
    format: String,
    output_path: String,
    filter: Option<TransactionFilter>,
) -> Result<ExportResult, String> {
    // Get transactions using the filter
    let transactions = list_transactions_impl(db, filter).await?;

    match format.as_str() {
        "csv" => {
            // Create CSV content
            let mut csv_content = String::from("Date,Amount,Description,Merchant,Category\n");

            // Get all category names in one query using JOIN
            let transaction_ids: Vec<i64> = transactions.iter().map(|t| t.id).collect();
            if transaction_ids.is_empty() {
                std::fs::write(&output_path, csv_content)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            } else {
                let placeholders = transaction_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let query_str = format!(
                    "SELECT t.id, c.name FROM transactions t
                     JOIN categories c ON t.category_id = c.id
                     WHERE t.id IN ({})",
                    placeholders
                );

                let mut query = sqlx::query_as::<_, (i64, String)>(&query_str);
                for id in &transaction_ids {
                    query = query.bind(id);
                }

                let category_map: std::collections::HashMap<i64, String> = query
                    .fetch_all(db)
                    .await
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .collect();

                for transaction in &transactions {
                    let category_name = category_map
                        .get(&transaction.id)
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string());

                    csv_content.push_str(&format!(
                        "{},{},{},{},{}\n",
                        transaction.date,
                        transaction.amount,
                        transaction.description,
                        transaction.merchant.as_ref().unwrap_or(&String::from("")),
                        category_name
                    ));
                }

                // Write to file
                std::fs::write(&output_path, csv_content)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
        }
        "json" => {
            let json_content = serde_json::to_string_pretty(&transactions)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

            std::fs::write(&output_path, json_content)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }
        _ => return Err(format!("Unsupported format: {}", format)),
    }

    Ok(ExportResult {
        success: true,
        file_path: output_path,
        record_count: transactions.len(),
    })
}

// Tauri command handlers (extract pool from managed state)

#[tauri::command]
pub async fn list_transactions(
    db_pool: tauri::State<'_, DbPool>,
    filter: Option<TransactionFilter>,
) -> Result<Vec<Transaction>, String> {
    list_transactions_impl(&db_pool.0, filter).await
}

#[tauri::command]
pub async fn update_transaction_category(
    db_pool: tauri::State<'_, DbPool>,
    transaction_id: i64,
    category_id: i64,
) -> Result<(), String> {
    update_transaction_category_impl(&db_pool.0, transaction_id, category_id).await
}

#[tauri::command]
pub async fn categorize_transaction(
    db_pool: tauri::State<'_, DbPool>,
    transaction_id: i64,
) -> Result<CategorizeResult, String> {
    categorize_transaction_impl(&db_pool.0, transaction_id).await
}

#[tauri::command]
pub async fn export_transactions(
    db_pool: tauri::State<'_, DbPool>,
    format: String,
    output_path: String,
    filter: Option<TransactionFilter>,
) -> Result<ExportResult, String> {
    export_transactions_impl(&db_pool.0, format, output_path, filter).await
}

#[tauri::command]
pub async fn count_transactions(
    db_pool: tauri::State<'_, DbPool>,
    filter: Option<TransactionFilter>,
) -> Result<i64, String> {
    count_transactions_impl(&db_pool.0, filter).await
}
