use crate::models::transaction::NewTransaction;

#[derive(Debug)]
pub enum DuplicateError {
    DatabaseError(String),
}

impl std::fmt::Display for DuplicateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DuplicateError::DatabaseError(e) => write!(f, "Database Error: {}", e),
        }
    }
}

impl std::error::Error for DuplicateError {}

pub struct DuplicateDetector;

impl DuplicateDetector {
    pub async fn is_duplicate(
        db: &sqlx::Pool<sqlx::Sqlite>,
        date: &str,
        amount: f64,
        description: &str,
    ) -> Result<bool, DuplicateError> {
        let hash = NewTransaction::calculate_hash(date, amount, description);

        let result: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM transactions WHERE hash = ?"
        )
        .bind(&hash)
        .fetch_one(db)
        .await
        .map_err(|e| DuplicateError::DatabaseError(e.to_string()))?;

        Ok(result.unwrap_or(0) > 0)
    }

    pub async fn filter_duplicates(
        db: &sqlx::Pool<sqlx::Sqlite>,
        transactions: Vec<(String, f64, String)>, // (date, amount, description)
    ) -> Result<Vec<bool>, DuplicateError> {
        let mut results = Vec::new();

        for (date, amount, description) in transactions {
            let is_dup = Self::is_duplicate(db, &date, amount, &description).await?;
            results.push(is_dup);
        }

        Ok(results)
    }
}
