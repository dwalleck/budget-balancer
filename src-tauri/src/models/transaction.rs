use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub category_id: i64,
    pub date: String,           // ISO 8601 format
    pub amount: f64,
    pub description: String,
    pub merchant: Option<String>,
    pub hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub account_id: i64,
    pub category_id: i64,
    pub date: String,
    pub amount: f64,
    pub description: String,
    pub merchant: Option<String>,
    pub hash: String,
}

impl NewTransaction {
    pub fn calculate_hash(date: &str, amount: f64, description: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}", date, amount, description));
        format!("{:x}", hasher.finalize())
    }
}
