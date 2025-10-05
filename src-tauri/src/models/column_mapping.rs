use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub id: i64,
    pub source_name: String,
    pub date_col: String,
    pub amount_col: String,
    pub description_col: String,
    pub merchant_col: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewColumnMapping {
    pub source_name: String,
    pub date_col: String,
    pub amount_col: String,
    pub description_col: String,
    pub merchant_col: Option<String>,
}
