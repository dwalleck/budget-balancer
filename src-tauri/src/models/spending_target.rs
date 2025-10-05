use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingTarget {
    pub id: i64,
    pub category_id: i64,
    pub amount: f64,
    pub period: String,  // 'monthly', 'quarterly', 'yearly'
    pub start_date: String,
    pub end_date: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSpendingTarget {
    pub category_id: i64,
    pub amount: f64,
    pub period: String,
    pub start_date: String,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetProgress {
    pub category_id: i64,
    pub category_name: String,
    pub target_amount: f64,
    pub actual_amount: f64,
    pub remaining: f64,
    pub percentage_used: f64,
    pub status: String,  // 'under', 'on_track', 'over'
    pub variance: f64,
}
