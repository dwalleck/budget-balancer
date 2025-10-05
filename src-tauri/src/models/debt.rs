use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Debt {
    pub id: i64,
    pub name: String,
    pub balance: f64,
    pub original_balance: f64,
    pub interest_rate: f64,  // Annual percentage
    pub min_payment: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDebt {
    pub name: String,
    pub balance: f64,
    pub interest_rate: f64,
    pub min_payment: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DebtPayment {
    pub id: i64,
    pub debt_id: i64,
    pub amount: f64,
    pub date: String,
    pub plan_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayoffStrategy {
    Avalanche,
    Snowball,
}

impl std::fmt::Display for PayoffStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayoffStrategy::Avalanche => write!(f, "avalanche"),
            PayoffStrategy::Snowball => write!(f, "snowball"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtPlan {
    pub id: i64,
    pub strategy: String,
    pub monthly_amount: f64,
    pub created_at: String,
    pub updated_at: String,
}
