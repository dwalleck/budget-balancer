use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryRule {
    pub id: i64,
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
    pub created_at: String,
}

// CategoryRule with joined category name for list responses
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryRuleWithName {
    pub id: i64,
    pub pattern: String,
    pub category_id: i64,
    pub category_name: String,
    pub priority: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCategoryRule {
    pub pattern: String,
    pub category_id: i64,
    pub priority: Option<i32>, // Optional, defaults to 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryRule {
    pub id: i64,
    pub pattern: Option<String>,
    pub category_id: Option<i64>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCategoryRuleResponse {
    pub success: bool,
    pub deleted_rule_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CategoryRuleFilter {
    ByCategoryId(i64),
}
