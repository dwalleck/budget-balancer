use crate::constants::{DEFAULT_CATEGORY_ID, DEFAULT_OFFSET, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::errors::TransactionError;
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

// Helper struct to build SQL WHERE clauses for transaction filters
// This eliminates duplication between list and count operations
struct TransactionFilterBuilder {
    where_clauses: Vec<String>,
    account_id: Option<i64>,
    category_id: Option<i64>,
    start_date: Option<String>,
    end_date: Option<String>,
}

impl TransactionFilterBuilder {
    fn new(filter: &TransactionFilter) -> Self {
        let mut where_clauses = Vec::new();

        if filter.account_id.is_some() {
            where_clauses.push(" AND account_id = ?".to_string());
        }
        if filter.category_id.is_some() {
            where_clauses.push(" AND category_id = ?".to_string());
        }
        if filter.start_date.is_some() {
            where_clauses.push(" AND date >= ?".to_string());
        }
        if filter.end_date.is_some() {
            where_clauses.push(" AND date <= ?".to_string());
        }

        Self {
            where_clauses,
            account_id: filter.account_id,
            category_id: filter.category_id,
            start_date: filter.start_date.clone(),
            end_date: filter.end_date.clone(),
        }
    }

    fn build_where_clause(&self) -> String {
        self.where_clauses.join("")
    }

    fn bind_parameters<'q, O>(
        &'q self,
        mut query: sqlx::query::QueryAs<'q, sqlx::Sqlite, O, sqlx::sqlite::SqliteArguments<'q>>,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, O, sqlx::sqlite::SqliteArguments<'q>>
    where
        O: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        if let Some(account_id) = self.account_id {
            query = query.bind(account_id);
        }
        if let Some(category_id) = self.category_id {
            query = query.bind(category_id);
        }
        if let Some(ref start_date) = self.start_date {
            query = query.bind(start_date);
        }
        if let Some(ref end_date) = self.end_date {
            query = query.bind(end_date);
        }
        query
    }
}

// Business logic functions (used by both commands and tests)

pub async fn list_transactions_impl(
    db: &SqlitePool,
    filter: Option<TransactionFilter>,
) -> Result<Vec<Transaction>, TransactionError> {
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

    // Build WHERE clause using helper to avoid duplication
    let filter_builder = TransactionFilterBuilder::new(&filter);

    let query = format!(
        "SELECT id, account_id, category_id, date, amount, description, merchant, hash, created_at FROM transactions WHERE 1=1{} ORDER BY date DESC LIMIT ? OFFSET ?",
        filter_builder.build_where_clause()
    );

    let query_builder = sqlx::query_as::<_, Transaction>(&query);

    // Bind filter parameters first, then pagination
    let query_builder = filter_builder.bind_parameters(query_builder);
    let query_builder = query_builder.bind(limit).bind(offset);

    query_builder
        .fetch_all(db)
        .await
        .map_err(|e| TransactionError::Database(e.to_string()))
}

pub async fn count_transactions_impl(
    db: &SqlitePool,
    filter: Option<TransactionFilter>,
) -> Result<i64, TransactionError> {
    let filter = filter.unwrap_or(TransactionFilter {
        account_id: None,
        category_id: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    });

    // Build WHERE clause using helper to avoid duplication
    let filter_builder = TransactionFilterBuilder::new(&filter);

    let query = format!(
        "SELECT COUNT(*) FROM transactions WHERE 1=1{}",
        filter_builder.build_where_clause()
    );

    let query_builder = sqlx::query_as::<_, (i64,)>(&query);
    let query_builder = filter_builder.bind_parameters(query_builder);

    query_builder
        .fetch_one(db)
        .await
        .map(|(count,)| count)
        .map_err(|e| TransactionError::Database(e.to_string()))
}

pub async fn update_transaction_category_impl(
    db: &SqlitePool,
    transaction_id: i64,
    category_id: i64,
) -> Result<(), TransactionError> {
    sqlx::query("UPDATE transactions SET category_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(category_id)
        .bind(transaction_id)
        .execute(db)
        .await
        .map_err(|e| TransactionError::Database(e.to_string()))?;

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
) -> Result<CategorizeResult, TransactionError> {
    // Get the transaction
    let transaction = sqlx::query_as::<_, Transaction>(
        "SELECT id, account_id, category_id, date, amount, description, merchant, hash, created_at
         FROM transactions WHERE id = ?"
    )
    .bind(transaction_id)
    .fetch_one(db)
    .await
    .map_err(|e| TransactionError::Database(e.to_string()))?;

    // Use categorizer to find best category
    let category_id = Categorizer::categorize(
        db,
        transaction.merchant.as_deref(),
        &transaction.description,
    )
    .await
    .map_err(|_| TransactionError::CategorizationFailed)?
    .unwrap_or(DEFAULT_CATEGORY_ID); // Default to "Uncategorized"

    // Update the transaction with new category
    sqlx::query("UPDATE transactions SET category_id = ? WHERE id = ?")
        .bind(category_id)
        .bind(transaction_id)
        .execute(db)
        .await
        .map_err(|e| TransactionError::Database(e.to_string()))?;

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
) -> Result<ExportResult, TransactionError> {
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
                    .map_err(|e| TransactionError::Database(format!("Failed to write file: {}", e)))?;
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
                    .map_err(|e| TransactionError::Database(e.to_string()))?
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
                    .map_err(|e| TransactionError::Database(format!("Failed to write file: {}", e)))?;
            }
        }
        "json" => {
            let json_content = serde_json::to_string_pretty(&transactions)
                .map_err(|e| TransactionError::Database(format!("Failed to serialize JSON: {}", e)))?;

            std::fs::write(&output_path, json_content)
                .map_err(|e| TransactionError::Database(format!("Failed to write file: {}", e)))?;
        }
        _ => return Err(TransactionError::Database(format!("Unsupported format: {}", format))),
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
    list_transactions_impl(&db_pool.0, filter)
        .await
        .map_err(|e| e.to_user_message())
}

#[tauri::command]
pub async fn update_transaction_category(
    db_pool: tauri::State<'_, DbPool>,
    transaction_id: i64,
    category_id: i64,
) -> Result<(), String> {
    update_transaction_category_impl(&db_pool.0, transaction_id, category_id)
        .await
        .map_err(|e| e.to_user_message())
}

#[tauri::command]
pub async fn categorize_transaction(
    db_pool: tauri::State<'_, DbPool>,
    transaction_id: i64,
) -> Result<CategorizeResult, String> {
    categorize_transaction_impl(&db_pool.0, transaction_id)
        .await
        .map_err(|e| e.to_user_message())
}

#[tauri::command]
pub async fn export_transactions(
    db_pool: tauri::State<'_, DbPool>,
    format: String,
    output_path: String,
    filter: Option<TransactionFilter>,
) -> Result<ExportResult, String> {
    export_transactions_impl(&db_pool.0, format, output_path, filter)
        .await
        .map_err(|e| e.to_user_message())
}

#[tauri::command]
pub async fn count_transactions(
    db_pool: tauri::State<'_, DbPool>,
    filter: Option<TransactionFilter>,
) -> Result<i64, String> {
    count_transactions_impl(&db_pool.0, filter)
        .await
        .map_err(|e| e.to_user_message())
}
