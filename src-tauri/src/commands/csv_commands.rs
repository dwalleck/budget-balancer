use crate::constants::{MAX_CSV_FILE_SIZE, MAX_CSV_ROWS, MIN_CSV_IMPORT_INTERVAL_MS};
use crate::errors::{sanitize_db_error, CsvImportError};
use crate::models::column_mapping::{
    ColumnMapping as DbColumnMapping, DeleteColumnMappingResponse, GetColumnMappingQuery,
    NewColumnMapping, UpdateColumnMapping,
};
use crate::services::csv_parser::{ColumnMapping, CsvParser};
use crate::services::transaction_importer::TransactionImporter;
use crate::utils::rate_limiter::RateLimiter;
use crate::DbPool;
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::SqlitePool;

// Get rate limiter interval from environment variable or use default
// Set CSV_RATE_LIMIT_MS=50 for fast test execution
// Defaults to 2000ms (MIN_CSV_IMPORT_INTERVAL_MS) in production
fn get_rate_limit_interval() -> u64 {
    std::env::var("CSV_RATE_LIMIT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(MIN_CSV_IMPORT_INTERVAL_MS)
}

// Global rate limiter for CSV imports
static CSV_RATE_LIMITER: Lazy<RateLimiter> = Lazy::new(|| RateLimiter::new(get_rate_limit_interval()));

// Test helper to reset rate limiter between tests
// Note: This is public to allow integration tests to reset the rate limiter
// In production, this function exists but is never called
pub fn reset_rate_limiter() {
    CSV_RATE_LIMITER.reset();
}

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

// Column Mapping Management Functions

pub async fn save_column_mapping_impl(
    db: &SqlitePool,
    mapping: NewColumnMapping,
) -> Result<DbColumnMapping, String> {
    // Check if mapping with same source_name exists (upsert behavior)
    let existing = sqlx::query_as::<_, DbColumnMapping>(
        "SELECT id, source_name, date_col, amount_col, description_col, merchant_col, created_at, updated_at
         FROM column_mappings WHERE source_name = ?"
    )
    .bind(&mapping.source_name)
    .fetch_optional(db)
    .await
    .map_err(|e| sanitize_db_error(e, "check existing mapping"))?;

    if let Some(existing_mapping) = existing {
        // Update existing mapping
        sqlx::query(
            "UPDATE column_mappings
             SET date_col = ?, amount_col = ?, description_col = ?, merchant_col = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?"
        )
        .bind(&mapping.date_col)
        .bind(&mapping.amount_col)
        .bind(&mapping.description_col)
        .bind(&mapping.merchant_col)
        .bind(existing_mapping.id)
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "update column mapping"))?;

        // Fetch and return updated mapping
        sqlx::query_as::<_, DbColumnMapping>(
            "SELECT id, source_name, date_col, amount_col, description_col, merchant_col, created_at, updated_at
             FROM column_mappings WHERE id = ?"
        )
        .bind(existing_mapping.id)
        .fetch_one(db)
        .await
        .map_err(|e| sanitize_db_error(e, "fetch updated mapping"))
    } else {
        // Create new mapping
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
        .map_err(|e| sanitize_db_error(e, "create column mapping"))?;

        let mapping_id = result.last_insert_rowid();

        // Fetch and return created mapping
        sqlx::query_as::<_, DbColumnMapping>(
            "SELECT id, source_name, date_col, amount_col, description_col, merchant_col, created_at, updated_at
             FROM column_mappings WHERE id = ?"
        )
        .bind(mapping_id)
        .fetch_one(db)
        .await
        .map_err(|e| sanitize_db_error(e, "fetch created mapping"))
    }
}

pub async fn list_column_mappings_impl(
    db: &SqlitePool,
) -> Result<Vec<DbColumnMapping>, String> {
    sqlx::query_as::<_, DbColumnMapping>(
        "SELECT id, source_name, date_col, amount_col, description_col, merchant_col, created_at, updated_at
         FROM column_mappings
         ORDER BY source_name ASC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load column mappings"))
}

pub async fn get_column_mapping_impl(
    db: &SqlitePool,
    query: GetColumnMappingQuery,
) -> Result<DbColumnMapping, String> {
    // Validate that at least one parameter is provided
    if query.id.is_none() && query.source_name.is_none() {
        return Err("Either id or source_name must be provided".to_string());
    }

    // If both provided, id takes precedence
    if let Some(id) = query.id {
        let mapping = sqlx::query_as::<_, DbColumnMapping>(
            "SELECT id, source_name, date_col, amount_col, description_col, merchant_col, created_at, updated_at
             FROM column_mappings WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| sanitize_db_error(e, "fetch column mapping"))?;

        mapping.ok_or_else(|| format!("Column mapping with id {} not found", id))
    } else if let Some(source_name) = query.source_name {
        let mapping = sqlx::query_as::<_, DbColumnMapping>(
            "SELECT id, source_name, date_col, amount_col, description_col, merchant_col, created_at, updated_at
             FROM column_mappings WHERE source_name = ?"
        )
        .bind(&source_name)
        .fetch_optional(db)
        .await
        .map_err(|e| sanitize_db_error(e, "fetch column mapping"))?;

        mapping.ok_or_else(|| format!("Column mapping with source_name '{}' not found", source_name))
    } else {
        unreachable!("Should have been caught by earlier validation")
    }
}

pub async fn update_column_mapping_impl(
    db: &SqlitePool,
    update: UpdateColumnMapping,
) -> Result<DbColumnMapping, String> {
    // First, verify the mapping exists
    let existing = sqlx::query_as::<_, DbColumnMapping>(
        "SELECT id, source_name, date_col, amount_col, description_col, merchant_col, created_at, updated_at
         FROM column_mappings WHERE id = ?"
    )
    .bind(update.id)
    .fetch_optional(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch column mapping"))?;

    if existing.is_none() {
        return Err(format!("Column mapping with id {} not found", update.id));
    }

    // Build update query based on provided fields
    let mut updates = Vec::new();
    let mut has_updates = false;

    if update.source_name.is_some() {
        updates.push("source_name = ?");
        has_updates = true;
    }
    if update.date_col.is_some() {
        updates.push("date_col = ?");
        has_updates = true;
    }
    if update.amount_col.is_some() {
        updates.push("amount_col = ?");
        has_updates = true;
    }
    if update.description_col.is_some() {
        updates.push("description_col = ?");
        has_updates = true;
    }
    if update.merchant_col.is_some() {
        updates.push("merchant_col = ?");
        has_updates = true;
    }

    if !has_updates {
        return Err("At least one field must be provided for update".to_string());
    }

    updates.push("updated_at = CURRENT_TIMESTAMP");

    let query_str = format!(
        "UPDATE column_mappings SET {} WHERE id = ?",
        updates.join(", ")
    );

    let mut query = sqlx::query(&query_str);

    // Bind parameters in order
    if let Some(ref source_name) = update.source_name {
        query = query.bind(source_name);
    }
    if let Some(ref date_col) = update.date_col {
        query = query.bind(date_col);
    }
    if let Some(ref amount_col) = update.amount_col {
        query = query.bind(amount_col);
    }
    if let Some(ref description_col) = update.description_col {
        query = query.bind(description_col);
    }
    if let Some(ref merchant_col) = update.merchant_col {
        query = query.bind(merchant_col);
    }
    query = query.bind(update.id);

    query
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "update column mapping"))?;

    // Fetch and return updated mapping
    sqlx::query_as::<_, DbColumnMapping>(
        "SELECT id, source_name, date_col, amount_col, description_col, merchant_col, created_at, updated_at
         FROM column_mappings WHERE id = ?"
    )
    .bind(update.id)
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch updated mapping"))
}

pub async fn delete_column_mapping_impl(
    db: &SqlitePool,
    mapping_id: i64,
) -> Result<DeleteColumnMappingResponse, String> {
    // Verify the mapping exists
    let existing = sqlx::query("SELECT id FROM column_mappings WHERE id = ?")
        .bind(mapping_id)
        .fetch_optional(db)
        .await
        .map_err(|e| sanitize_db_error(e, "check mapping exists"))?;

    if existing.is_none() {
        return Err(format!("Column mapping with id {} not found", mapping_id));
    }

    // Delete the mapping (does not affect existing transactions)
    sqlx::query("DELETE FROM column_mappings WHERE id = ?")
        .bind(mapping_id)
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "delete column mapping"))?;

    Ok(DeleteColumnMappingResponse {
        success: true,
        deleted_mapping_id: mapping_id,
    })
}

pub async fn import_csv_impl(
    db: &SqlitePool,
    account_id: i64,
    csv_content: String,
    mapping: ColumnMapping,
) -> Result<ImportResult, CsvImportError> {
    // Check rate limit FIRST (before expensive operations)
    // This ensures rate limiting cannot be bypassed by calling _impl directly
    CSV_RATE_LIMITER.check_and_update()
        .map_err(|err| CsvImportError::RateLimitExceeded(err.seconds()))?;

    // Validate file size
    if csv_content.len() > MAX_CSV_FILE_SIZE {
        return Err(CsvImportError::FileTooLarge {
            size: csv_content.len(),
            max: MAX_CSV_FILE_SIZE,
        });
    }

    // Validate row count (approximate by counting newlines)
    let row_count = csv_content.lines().count();
    if row_count > MAX_CSV_ROWS {
        return Err(CsvImportError::TooManyRows {
            count: row_count,
            max: MAX_CSV_ROWS,
        });
    }

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
        Err(e) => Err(CsvImportError::Database(e.to_string())),
    }
}

// Tauri command handlers (extract pool from managed state)

#[tauri::command]
pub async fn get_csv_headers(csv_content: String) -> Result<Vec<String>, String> {
    // Validate file size
    if csv_content.len() > MAX_CSV_FILE_SIZE {
        return Err(CsvImportError::FileTooLarge {
            size: csv_content.len(),
            max: MAX_CSV_FILE_SIZE,
        }.to_user_message());
    }

    CsvParser::get_headers(&csv_content).map_err(|e| {
        CsvImportError::ParseError(e.to_string()).to_user_message()
    })
}

#[tauri::command]
pub async fn save_column_mapping(
    db_pool: tauri::State<'_, DbPool>,
    mapping: NewColumnMapping,
) -> Result<DbColumnMapping, String> {
    save_column_mapping_impl(&db_pool.0, mapping).await
}

#[tauri::command]
pub async fn list_column_mappings(
    db_pool: tauri::State<'_, DbPool>,
) -> Result<Vec<DbColumnMapping>, String> {
    list_column_mappings_impl(&db_pool.0).await
}

#[tauri::command]
pub async fn get_column_mapping(
    db_pool: tauri::State<'_, DbPool>,
    query: GetColumnMappingQuery,
) -> Result<DbColumnMapping, String> {
    get_column_mapping_impl(&db_pool.0, query).await
}

#[tauri::command]
pub async fn update_column_mapping(
    db_pool: tauri::State<'_, DbPool>,
    update: UpdateColumnMapping,
) -> Result<DbColumnMapping, String> {
    update_column_mapping_impl(&db_pool.0, update).await
}

#[tauri::command]
pub async fn delete_column_mapping(
    db_pool: tauri::State<'_, DbPool>,
    mapping_id: i64,
) -> Result<DeleteColumnMappingResponse, String> {
    delete_column_mapping_impl(&db_pool.0, mapping_id).await
}

#[tauri::command]
pub async fn import_csv(
    db_pool: tauri::State<'_, DbPool>,
    account_id: i64,
    csv_content: String,
    mapping: ColumnMapping,
) -> Result<ImportResult, String> {
    // Rate limiting is enforced in import_csv_impl to prevent bypass
    import_csv_impl(&db_pool.0, account_id, csv_content, mapping)
        .await
        .map_err(|e| e.to_user_message())
}
