use crate::constants::{MAX_CSV_FILE_SIZE, MAX_CSV_ROWS, MIN_CSV_IMPORT_INTERVAL_MS};
use crate::models::column_mapping::NewColumnMapping;
use crate::services::csv_parser::{ColumnMapping, CsvParser};
use crate::services::transaction_importer::TransactionImporter;
use crate::utils::rate_limiter::RateLimiter;
use crate::DbPool;
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::SqlitePool;

// Global rate limiter for CSV imports
static CSV_RATE_LIMITER: Lazy<RateLimiter> = Lazy::new(|| RateLimiter::new(MIN_CSV_IMPORT_INTERVAL_MS));

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
    // Check rate limit FIRST (before expensive operations)
    // This ensures rate limiting cannot be bypassed by calling _impl directly
    CSV_RATE_LIMITER.check_and_update()?;

    // Validate file size
    if csv_content.len() > MAX_CSV_FILE_SIZE {
        return Err(format!(
            "File too large. Maximum size is {} MB.",
            MAX_CSV_FILE_SIZE / (1024 * 1024)
        ));
    }

    // Validate row count (approximate by counting newlines)
    let row_count = csv_content.lines().count();
    if row_count > MAX_CSV_ROWS {
        return Err(format!(
            "Too many rows. Maximum is {} rows, found approximately {}.",
            MAX_CSV_ROWS, row_count
        ));
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
        Err(e) => {
            eprintln!("CSV import error: {}", e);  // Log detailed error
            Err("Failed to import CSV file. Please check the file format.".to_string())  // Return safe message
        }
    }
}

// Tauri command handlers (extract pool from managed state)

#[tauri::command]
pub async fn get_csv_headers(csv_content: String) -> Result<Vec<String>, String> {
    // Validate file size
    if csv_content.len() > MAX_CSV_FILE_SIZE {
        return Err(format!(
            "File too large. Maximum size is {} MB.",
            MAX_CSV_FILE_SIZE / (1024 * 1024)
        ));
    }

    CsvParser::get_headers(&csv_content).map_err(|e| {
        eprintln!("CSV header parsing error: {}", e);
        "Failed to parse CSV headers. Please check the file format.".to_string()
    })
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
    // Rate limiting is enforced in import_csv_impl to prevent bypass
    import_csv_impl(&db_pool.0, account_id, csv_content, mapping).await
}
