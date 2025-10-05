use crate::errors::sanitize_db_error;
use crate::models::debt::{Debt, DebtPayment, NewDebt};
use crate::services::avalanche_calculator::AvalancheCalculator;
use crate::services::snowball_calculator::SnowballCalculator;
use crate::DbPool;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoffPlanResponse {
    pub plan_id: i64,
    pub strategy: String,
    pub payoff_date: String,
    pub total_interest: f64,
    pub monthly_breakdown: Vec<MonthlyPaymentResponse>,
    pub debt_summaries: Vec<DebtSummaryResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyPaymentResponse {
    pub month: i32,
    pub date: String,
    pub payments: Vec<DebtPaymentDetailResponse>,
    pub total_paid: f64,
    pub remaining_balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtPaymentDetailResponse {
    pub debt_id: i64,
    pub debt_name: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtSummaryResponse {
    pub debt_id: i64,
    pub debt_name: String,
    pub payoff_month: i32,
    pub total_interest_paid: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPaymentResponse {
    pub payment_id: i64,
    pub updated_balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancePoint {
    pub date: String,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtProgressResponse {
    pub debt: Debt,
    pub payments: Vec<DebtPayment>,
    pub total_paid: f64,
    pub balance_history: Vec<BalancePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyComparison {
    pub strategy: String,
    pub payoff_date: String,
    pub total_interest: f64,
    pub payoff_months: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSavings {
    pub interest_saved: f64,
    pub months_saved: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareStrategiesResponse {
    pub avalanche: StrategyComparison,
    pub snowball: StrategyComparison,
    pub savings: ComparisonSavings,
}

// Business logic functions (used by both commands and tests)

pub async fn create_debt_impl(db: &SqlitePool, debt: NewDebt) -> Result<i64, String> {
    // Validate inputs
    if debt.balance < 0.0 || debt.min_payment < 0.0 {
        return Err("InvalidAmount: balance and min_payment must be non-negative".to_string());
    }
    if debt.interest_rate < 0.0 || debt.interest_rate > 100.0 {
        return Err("InvalidRate: interest_rate must be between 0 and 100".to_string());
    }

    let result = sqlx::query(
        "INSERT INTO debts (name, balance, original_balance, interest_rate, min_payment) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&debt.name)
    .bind(debt.balance)
    .bind(debt.balance)  // original_balance = balance initially
    .bind(debt.interest_rate)
    .bind(debt.min_payment)
    .execute(db)
    .await
    .map_err(|e| sanitize_db_error(e, "create debt"))?;

    Ok(result.last_insert_rowid())
}

// T030: Create debt command
#[tauri::command]
pub async fn create_debt(db_pool: tauri::State<'_, DbPool>, debt: NewDebt) -> Result<i64, String> {
    create_debt_impl(&db_pool.0, debt).await
}

pub async fn list_debts_impl(db: &SqlitePool) -> Result<Vec<Debt>, String> {
    sqlx::query_as::<_, Debt>(
        "SELECT id, name, balance, original_balance, interest_rate, min_payment, created_at, updated_at
         FROM debts ORDER BY balance DESC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load debts"))
}

// T031: List debts command
#[tauri::command]
pub async fn list_debts(db_pool: tauri::State<'_, DbPool>) -> Result<Vec<Debt>, String> {
    list_debts_impl(&db_pool.0).await
}

pub async fn update_debt_impl(
    db: &SqlitePool,
    debt_id: i64,
    balance: Option<f64>,
    interest_rate: Option<f64>,
    min_payment: Option<f64>,
) -> Result<bool, String> {
    // Validate inputs
    if let Some(bal) = balance {
        if bal < 0.0 {
            return Err("InvalidAmount: balance must be non-negative".to_string());
        }
    }
    if let Some(rate) = interest_rate {
        if rate < 0.0 || rate > 100.0 {
            return Err("InvalidRate: interest_rate must be between 0 and 100".to_string());
        }
    }
    if let Some(payment) = min_payment {
        if payment < 0.0 {
            return Err("InvalidAmount: min_payment must be non-negative".to_string());
        }
    }

    // Check if debt exists
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM debts WHERE id = ?")
        .bind(debt_id)
        .fetch_optional(db)
        .await
        .map_err(|e| sanitize_db_error(e, "check debt existence"))?;

    if exists.is_none() {
        return Err("DebtNotFound: debt not found".to_string());
    }

    // Build update query dynamically
    let mut updates = Vec::new();
    let mut query = String::from("UPDATE debts SET ");

    if balance.is_some() {
        updates.push("balance = ?");
    }
    if interest_rate.is_some() {
        updates.push("interest_rate = ?");
    }
    if min_payment.is_some() {
        updates.push("min_payment = ?");
    }
    updates.push("updated_at = CURRENT_TIMESTAMP");

    query.push_str(&updates.join(", "));
    query.push_str(" WHERE id = ?");

    let mut q = sqlx::query(&query);
    if let Some(bal) = balance {
        q = q.bind(bal);
    }
    if let Some(rate) = interest_rate {
        q = q.bind(rate);
    }
    if let Some(payment) = min_payment {
        q = q.bind(payment);
    }
    q = q.bind(debt_id);

    q.execute(db).await.map_err(|e| sanitize_db_error(e, "update debt"))?;

    Ok(true)
}

// T032: Update debt command
#[tauri::command]
pub async fn update_debt(
    db_pool: tauri::State<'_, DbPool>,
    debt_id: i64,
    balance: Option<f64>,
    interest_rate: Option<f64>,
    min_payment: Option<f64>,
) -> Result<bool, String> {
    update_debt_impl(&db_pool.0, debt_id, balance, interest_rate, min_payment).await
}

pub async fn calculate_payoff_plan_impl(
    db: &SqlitePool,
    strategy: String,
    monthly_amount: f64,
) -> Result<PayoffPlanResponse, String> {
    let debts = sqlx::query_as::<_, Debt>(
        "SELECT id, name, balance, original_balance, interest_rate, min_payment, created_at, updated_at
         FROM debts WHERE balance > 0 ORDER BY balance DESC"
    )
    .fetch_all(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load debts for payoff calculation"))?;

    if debts.is_empty() {
        return Err("NoDebts: no debts in database".to_string());
    }

    let plan = match strategy.as_str() {
        "avalanche" => AvalancheCalculator::calculate_payoff_plan(debts, monthly_amount)?,
        "snowball" => SnowballCalculator::calculate_payoff_plan(debts, monthly_amount)?,
        _ => return Err("Invalid strategy: must be 'avalanche' or 'snowball'".to_string()),
    };

    // Save the plan
    let result = sqlx::query(
        "INSERT INTO debt_plans (strategy, monthly_amount) VALUES (?, ?)"
    )
    .bind(&plan.strategy)
    .bind(monthly_amount)
    .execute(db)
    .await
    .map_err(|e| sanitize_db_error(e, "save debt payoff plan"))?;

    let plan_id = result.last_insert_rowid();

    Ok(PayoffPlanResponse {
        plan_id,
        strategy: plan.strategy,
        payoff_date: plan.payoff_date,
        total_interest: plan.total_interest,
        monthly_breakdown: plan.monthly_breakdown.into_iter().map(|m| MonthlyPaymentResponse {
            month: m.month,
            date: m.date,
            payments: m.payments.into_iter().map(|p| DebtPaymentDetailResponse {
                debt_id: p.debt_id,
                debt_name: p.debt_name,
                amount: p.amount,
            }).collect(),
            total_paid: m.total_paid,
            remaining_balance: m.remaining_balance,
        }).collect(),
        debt_summaries: plan.debt_summaries.into_iter().map(|s| DebtSummaryResponse {
            debt_id: s.debt_id,
            debt_name: s.debt_name,
            payoff_month: s.payoff_month,
            total_interest_paid: s.total_interest_paid,
        }).collect(),
    })
}

// T033: Calculate payoff plan command
#[tauri::command]
pub async fn calculate_payoff_plan(
    db_pool: tauri::State<'_, DbPool>,
    strategy: String,
    monthly_amount: f64,
) -> Result<PayoffPlanResponse, String> {
    calculate_payoff_plan_impl(&db_pool.0, strategy, monthly_amount).await
}

pub async fn get_payoff_plan_impl(db: &SqlitePool, plan_id: i64) -> Result<PayoffPlanResponse, String> {
    #[derive(sqlx::FromRow)]
    struct DebtPlan {
        strategy: String,
        monthly_amount: f64,
    }

    let plan = sqlx::query_as::<_, DebtPlan>(
        "SELECT strategy, monthly_amount FROM debt_plans WHERE id = ?"
    )
    .bind(plan_id)
    .fetch_optional(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load debt payoff plan"))?
    .ok_or_else(|| "PlanNotFound: plan not found".to_string())?;

    // Recalculate the plan (plans are not fully stored, just metadata)
    let debts = sqlx::query_as::<_, Debt>(
        "SELECT id, name, balance, original_balance, interest_rate, min_payment, created_at, updated_at
         FROM debts WHERE balance > 0"
    )
    .fetch_all(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load debts for plan recalculation"))?;

    let calc_plan = match plan.strategy.as_str() {
        "avalanche" => AvalancheCalculator::calculate_payoff_plan(debts, plan.monthly_amount)?,
        "snowball" => SnowballCalculator::calculate_payoff_plan(debts, plan.monthly_amount)?,
        _ => return Err("Invalid strategy in stored plan".to_string()),
    };

    Ok(PayoffPlanResponse {
        plan_id,
        strategy: calc_plan.strategy,
        payoff_date: calc_plan.payoff_date,
        total_interest: calc_plan.total_interest,
        monthly_breakdown: calc_plan.monthly_breakdown.into_iter().map(|m| MonthlyPaymentResponse {
            month: m.month,
            date: m.date,
            payments: m.payments.into_iter().map(|p| DebtPaymentDetailResponse {
                debt_id: p.debt_id,
                debt_name: p.debt_name,
                amount: p.amount,
            }).collect(),
            total_paid: m.total_paid,
            remaining_balance: m.remaining_balance,
        }).collect(),
        debt_summaries: calc_plan.debt_summaries.into_iter().map(|s| DebtSummaryResponse {
            debt_id: s.debt_id,
            debt_name: s.debt_name,
            payoff_month: s.payoff_month,
            total_interest_paid: s.total_interest_paid,
        }).collect(),
    })
}

// T034: Get payoff plan command
#[tauri::command]
pub async fn get_payoff_plan(db_pool: tauri::State<'_, DbPool>, plan_id: i64) -> Result<PayoffPlanResponse, String> {
    get_payoff_plan_impl(&db_pool.0, plan_id).await
}

pub async fn record_debt_payment_impl(
    db: &SqlitePool,
    debt_id: i64,
    amount: f64,
    date: String,
    plan_id: Option<i64>,
) -> Result<RecordPaymentResponse, String> {
    if amount <= 0.0 {
        return Err("InvalidAmount: amount must be positive".to_string());
    }

    // Use a transaction to ensure atomicity
    let mut tx = db.begin().await.map_err(|e| sanitize_db_error(e, "begin transaction"))?;

    // Get current debt
    let debt = sqlx::query_as::<_, Debt>(
        "SELECT id, name, balance, original_balance, interest_rate, min_payment, created_at, updated_at
         FROM debts WHERE id = ?"
    )
    .bind(debt_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| sanitize_db_error(e, "load debt for payment"))?
    .ok_or_else(|| "DebtNotFound: debt not found".to_string())?;

    if amount > debt.balance {
        return Err("InvalidAmount: payment exceeds debt balance".to_string());
    }

    // Record payment
    let payment_result = sqlx::query(
        "INSERT INTO debt_payments (debt_id, amount, date, plan_id) VALUES (?, ?, ?, ?)"
    )
    .bind(debt_id)
    .bind(amount)
    .bind(&date)
    .bind(plan_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| sanitize_db_error(e, "record debt payment"))?;

    let payment_id = payment_result.last_insert_rowid();

    // Update debt balance
    let updated_balance = debt.balance - amount;
    sqlx::query("UPDATE debts SET balance = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(updated_balance)
        .bind(debt_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| sanitize_db_error(e, "update debt balance after payment"))?;

    // Commit transaction
    tx.commit().await.map_err(|e| sanitize_db_error(e, "commit payment transaction"))?;

    Ok(RecordPaymentResponse {
        payment_id,
        updated_balance,
    })
}

// T035: Record debt payment command
#[tauri::command]
pub async fn record_debt_payment(
    db_pool: tauri::State<'_, DbPool>,
    debt_id: i64,
    amount: f64,
    date: String,
    plan_id: Option<i64>,
) -> Result<RecordPaymentResponse, String> {
    record_debt_payment_impl(&db_pool.0, debt_id, amount, date, plan_id).await
}

pub async fn get_debt_progress_impl(
    db: &SqlitePool,
    debt_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<DebtProgressResponse, String> {
    let debt = sqlx::query_as::<_, Debt>(
        "SELECT id, name, balance, original_balance, interest_rate, min_payment, created_at, updated_at
         FROM debts WHERE id = ?"
    )
    .bind(debt_id)
    .fetch_optional(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load debt for progress"))?
    .ok_or_else(|| "DebtNotFound: debt not found".to_string())?;

    let payments = if let (Some(start), Some(end)) = (start_date, end_date) {
        sqlx::query_as::<_, DebtPayment>(
            "SELECT id, debt_id, amount, date, plan_id, created_at
             FROM debt_payments
             WHERE debt_id = ? AND date >= ? AND date <= ?
             ORDER BY date DESC"
        )
        .bind(debt_id)
        .bind(start)
        .bind(end)
        .fetch_all(db)
        .await
        .map_err(|e| sanitize_db_error(e, "load debt payment history"))?
    } else {
        sqlx::query_as::<_, DebtPayment>(
            "SELECT id, debt_id, amount, date, plan_id, created_at
             FROM debt_payments
             WHERE debt_id = ?
             ORDER BY date DESC"
        )
        .bind(debt_id)
        .fetch_all(db)
        .await
        .map_err(|e| sanitize_db_error(e, "load debt payment history"))?
    };

    let total_paid: f64 = payments.iter().map(|p| p.amount).sum();

    // Build balance history from payments
    let mut balance_history = Vec::new();
    let mut current_balance = debt.original_balance;

    for payment in &payments {
        current_balance -= payment.amount;
        balance_history.push(BalancePoint {
            date: payment.date.clone(),
            balance: current_balance.max(0.0),
        });
    }

    Ok(DebtProgressResponse {
        debt,
        payments,
        total_paid,
        balance_history,
    })
}

// T036: Get debt progress command
#[tauri::command]
pub async fn get_debt_progress(
    db_pool: tauri::State<'_, DbPool>,
    debt_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<DebtProgressResponse, String> {
    get_debt_progress_impl(&db_pool.0, debt_id, start_date, end_date).await
}

pub async fn compare_strategies_impl(db: &SqlitePool, monthly_amount: f64) -> Result<CompareStrategiesResponse, String> {
    let debts = sqlx::query_as::<_, Debt>(
        "SELECT id, name, balance, original_balance, interest_rate, min_payment, created_at, updated_at
         FROM debts WHERE balance > 0"
    )
    .fetch_all(db)
    .await
    .map_err(|e| sanitize_db_error(e, "load debts for strategy comparison"))?;

    if debts.is_empty() {
        return Err("NoDebts: no debts in database".to_string());
    }

    let avalanche_plan = AvalancheCalculator::calculate_payoff_plan(debts.clone(), monthly_amount)?;
    let snowball_plan = SnowballCalculator::calculate_payoff_plan(debts, monthly_amount)?;

    let interest_saved = snowball_plan.total_interest - avalanche_plan.total_interest;
    let months_saved = (snowball_plan.monthly_breakdown.len() as i32) - (avalanche_plan.monthly_breakdown.len() as i32);

    Ok(CompareStrategiesResponse {
        avalanche: StrategyComparison {
            strategy: "avalanche".to_string(),
            payoff_date: avalanche_plan.payoff_date,
            total_interest: avalanche_plan.total_interest,
            payoff_months: avalanche_plan.monthly_breakdown.len() as i32,
        },
        snowball: StrategyComparison {
            strategy: "snowball".to_string(),
            payoff_date: snowball_plan.payoff_date,
            total_interest: snowball_plan.total_interest,
            payoff_months: snowball_plan.monthly_breakdown.len() as i32,
        },
        savings: ComparisonSavings {
            interest_saved: interest_saved.max(0.0),
            months_saved: months_saved.max(0),
        },
    })
}

// T037: Compare strategies command
#[tauri::command]
pub async fn compare_strategies(db_pool: tauri::State<'_, DbPool>, monthly_amount: f64) -> Result<CompareStrategiesResponse, String> {
    compare_strategies_impl(&db_pool.0, monthly_amount).await
}
