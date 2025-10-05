use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRule {
    pub id: i64,
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCategoryRule {
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
}
