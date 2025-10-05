#[derive(Debug)]
pub enum CategorizerError {
    DatabaseError(String),
}

impl std::fmt::Display for CategorizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CategorizerError::DatabaseError(e) => write!(f, "Database Error: {}", e),
        }
    }
}

impl std::error::Error for CategorizerError {}

pub struct Categorizer;

impl Categorizer {
    /// Finds the best matching category for a transaction based on merchant/description
    /// Returns the category_id, or None if no match found
    pub async fn categorize(
        db: &sqlx::Pool<sqlx::Sqlite>,
        merchant: Option<&str>,
        description: &str,
    ) -> Result<Option<i64>, CategorizerError> {

        // Get all category rules ordered by priority (highest first)
        let rules: Vec<(i64, String, i64)> = sqlx::query_as(
            "SELECT id, pattern, category_id FROM category_rules ORDER BY priority DESC"
        )
        .fetch_all(db)
        .await
        .map_err(|e| CategorizerError::DatabaseError(e.to_string()))?;

        // Try to match against merchant first, then description
        let text_to_match = merchant.unwrap_or(description).to_lowercase();

        for (_rule_id, pattern, category_id) in rules {
            if text_to_match.contains(&pattern.to_lowercase()) {
                return Ok(Some(category_id));
            }
        }

        // No match found - return uncategorized category by querying for it
        let uncategorized_id: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM categories WHERE name = 'Uncategorized' LIMIT 1"
        )
        .fetch_optional(db)
        .await
        .map_err(|e| CategorizerError::DatabaseError(e.to_string()))?;

        Ok(uncategorized_id.map(|r| r.0))
    }
}
