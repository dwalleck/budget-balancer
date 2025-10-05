use crate::models::column_mapping::NewColumnMapping;
use crate::services::csv_parser::{ColumnMapping, CsvParser};
use crate::services::transaction_importer::TransactionImporter;
use crate::DbPool;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub total: usize,
    pub imported: usize,
    pub duplicates: usize,
    pub errors: usize,
    pub message: String,
}

// Business logic functions (used by both commands and tests)

pub async fn save_column_mapping_impl(
    db: &SqlitePool,
    mapping: NewColumnMapping,
) -> Result<i64, String> {
    let result = sqlx::query(
        "INSERT INTO column_mappings (source_name, date_col, amount_col, description_col, merchant_col)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&mapping.source_name)
    .bind(&mapping.date_col)
    .bind(&mapping.amount_col)
    .bind(&mapping.description_col)
    .bind(&mapping.merchant_col)
    .execute(db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            format!("A mapping with the name '{}' already exists", mapping.source_name)
        } else {
            e.to_string()
        }
    })?;

    Ok(result.last_insert_rowid())
}

pub async fn import_csv_impl(
    db: &SqlitePool,
    account_id: i64,
    csv_content: String,
    mapping: ColumnMapping,
) -> Result<ImportResult, String> {
    match TransactionImporter::import(db, account_id, &csv_content, &mapping).await {
        Ok(stats) => Ok(ImportResult {
            success: true,
            total: stats.total,
            imported: stats.imported,
            duplicates: stats.duplicates,
            errors: stats.errors,
            message: format!(
                "Imported {} of {} transactions ({} duplicates skipped, {} errors)",
                stats.imported, stats.total, stats.duplicates, stats.errors
            ),
        }),
        Err(e) => Err(e.to_string()),
    }
}

// Tauri command handlers (extract pool from managed state)

#[tauri::command]
pub async fn get_csv_headers(csv_content: String) -> Result<Vec<String>, String> {
    CsvParser::get_headers(&csv_content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_column_mapping(
    db_pool: tauri::State<'_, DbPool>,
    mapping: NewColumnMapping,
) -> Result<i64, String> {
    save_column_mapping_impl(&db_pool.0, mapping).await
}

#[tauri::command]
pub async fn import_csv(
    db_pool: tauri::State<'_, DbPool>,
    account_id: i64,
    csv_content: String,
    mapping: ColumnMapping,
) -> Result<ImportResult, String> {
    import_csv_impl(&db_pool.0, account_id, csv_content, mapping).await
}
