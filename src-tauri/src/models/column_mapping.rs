use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ColumnMapping {
    pub id: i64,
    pub source_name: String,
    pub date_col: String,
    pub amount_col: String,
    pub description_col: String,
    pub merchant_col: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewColumnMapping {
    pub source_name: String,
    pub date_col: String,
    pub amount_col: String,
    pub description_col: String,
    pub merchant_col: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetColumnMappingQuery {
    pub id: Option<i64>,
    pub source_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateColumnMapping {
    pub id: i64,
    pub source_name: Option<String>,
    pub date_col: Option<String>,
    pub amount_col: Option<String>,
    pub description_col: Option<String>,
    pub merchant_col: Option<Option<String>>, // Option<Option> to distinguish between "not updating" and "setting to None"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteColumnMappingResponse {
    pub success: bool,
    pub deleted_mapping_id: i64,
}
