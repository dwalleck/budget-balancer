use super::csv_parser::{CsvParser, ColumnMapping};
use super::duplicate_detector::DuplicateDetector;
use super::categorizer::Categorizer;
use crate::constants::{DEFAULT_CATEGORY_ID, MAX_TRANSACTION_AMOUNT};
use crate::models::transaction::NewTransaction;

#[derive(Debug)]
pub enum ImportError {
    CsvError(String),
    DuplicateError(String),
    CategorizerError(String),
    ValidationError(String),
    DatabaseError(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::CsvError(e) => write!(f, "CSV Error: {}", e),
            ImportError::DuplicateError(e) => write!(f, "Duplicate Detection Error: {}", e),
            ImportError::CategorizerError(e) => write!(f, "Categorization Error: {}", e),
            ImportError::ValidationError(e) => write!(f, "Validation Error: {}", e),
            ImportError::DatabaseError(e) => write!(f, "Database Error: {}", e),
        }
    }
}

impl std::error::Error for ImportError {}

pub struct ImportStats {
    pub total: usize,
    pub imported: usize,
    pub duplicates: usize,
    pub errors: usize,
}

pub struct TransactionImporter;

impl TransactionImporter {
    pub async fn import(
        db: &sqlx::Pool<sqlx::Sqlite>,
        account_id: i64,
        csv_content: &str,
        mapping: &ColumnMapping,
    ) -> Result<ImportStats, ImportError> {
        // Parse CSV
        let transactions = CsvParser::parse(csv_content, mapping)
            .map_err(|e| ImportError::CsvError(e.to_string()))?;

        let total = transactions.len();
        let mut imported = 0;
        let mut duplicates = 0;
        let mut errors = 0;

        for transaction in transactions {
            // Validate transaction amount
            if transaction.amount.abs() > MAX_TRANSACTION_AMOUNT {
                return Err(ImportError::ValidationError(
                    format!("Transaction amount ${:.2} exceeds maximum allowed amount of ${:.2}",
                        transaction.amount.abs(), MAX_TRANSACTION_AMOUNT)
                ));
            }

            // Check for duplicates
            let is_duplicate = DuplicateDetector::is_duplicate(
                db,
                &transaction.date,
                transaction.amount,
                &transaction.description,
            )
            .await
            .map_err(|e| ImportError::DuplicateError(e.to_string()))?;

            if is_duplicate {
                duplicates += 1;
                continue;
            }

            // Categorize
            let category_id = Categorizer::categorize(
                db,
                transaction.merchant.as_deref(),
                &transaction.description,
            )
            .await
            .map_err(|e| ImportError::CategorizerError(e.to_string()))?
            .unwrap_or(DEFAULT_CATEGORY_ID); // Default to uncategorized

            // Calculate hash
            let hash = NewTransaction::calculate_hash(
                &transaction.date,
                transaction.amount,
                &transaction.description,
            );

            // Insert transaction
            let result = sqlx::query(
                r#"
                INSERT INTO transactions (account_id, category_id, date, amount, description, merchant, hash)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(account_id)
            .bind(category_id)
            .bind(&transaction.date)
            .bind(transaction.amount)
            .bind(&transaction.description)
            .bind(&transaction.merchant)
            .bind(&hash)
            .execute(db)
            .await;

            match result {
                Ok(_) => imported += 1,
                Err(_) => errors += 1,
            }
        }

        Ok(ImportStats {
            total,
            imported,
            duplicates,
            errors,
        })
    }
}
