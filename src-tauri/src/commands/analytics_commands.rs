use crate::errors::sanitize_db_error;
use crate::services::spending_aggregator::{CategorySpending, SpendingAggregator, SpendingByCategory};
use crate::services::target_tracker::{TargetTracker, TargetsProgress};
use crate::services::trends_calculator::{TrendsCalculator, SpendingTrends};
use crate::DbPool;
use chrono::Datelike;
use serde::Serialize;
use sqlx::SqlitePool;

// Business logic functions (used by both commands and tests)

// T071: get_spending_by_category
pub async fn get_spending_by_category_impl(
    db: &SqlitePool,
    start_date: &str,
    end_date: &str,
    account_id: Option<i64>,
) -> Result<SpendingByCategory, String> {
    SpendingAggregator::get_spending_by_category(db, start_date, end_date, account_id).await
}

#[tauri::command]
pub async fn get_spending_by_category(
    db_pool: tauri::State<'_, DbPool>,
    start_date: String,
    end_date: String,
    account_id: Option<i64>,
) -> Result<SpendingByCategory, String> {
    get_spending_by_category_impl(&db_pool.0, &start_date, &end_date, account_id).await
}

// T072: get_spending_trends
pub async fn get_spending_trends_impl(
    db: &SqlitePool,
    start_date: &str,
    end_date: &str,
    interval: &str,
    category_id: Option<i64>,
) -> Result<SpendingTrends, String> {
    TrendsCalculator::get_spending_trends(db, start_date, end_date, interval, category_id).await
}

#[tauri::command]
pub async fn get_spending_trends(
    db_pool: tauri::State<'_, DbPool>,
    start_date: String,
    end_date: String,
    interval: String,
    category_id: Option<i64>,
) -> Result<SpendingTrends, String> {
    get_spending_trends_impl(&db_pool.0, &start_date, &end_date, &interval, category_id).await
}

// T073: get_spending_targets_progress
pub async fn get_spending_targets_progress_impl(
    db: &SqlitePool,
    period: Option<String>,
    custom_start: Option<String>,
    custom_end: Option<String>,
) -> Result<TargetsProgress, String> {
    // Calculate date range based on period or custom dates
    let (start_date, end_date) = if let (Some(start), Some(end)) = (custom_start, custom_end) {
        (start, end)
    } else {
        let period_str = period.unwrap_or_else(|| "monthly".to_string());
        match period_str.as_str() {
            "monthly" => {
                let now = chrono::Local::now().naive_local();
                let start = now.format("%Y-%m-01").to_string();
                let end = now.format("%Y-%m-%d").to_string();
                (start, end)
            }
            "quarterly" => {
                let now = chrono::Local::now().naive_local();
                let quarter_start_month = ((now.month() - 1) / 3) * 3 + 1;
                let start = format!("{}-{:02}-01", now.year(), quarter_start_month);
                let end = now.format("%Y-%m-%d").to_string();
                (start, end)
            }
            "yearly" => {
                let now = chrono::Local::now().naive_local();
                let start = format!("{}-01-01", now.year());
                let end = now.format("%Y-%m-%d").to_string();
                (start, end)
            }
            _ => return Err(format!("Invalid period: {}", period_str)),
        }
    };

    TargetTracker::get_targets_progress(db, &start_date, &end_date).await
}

#[tauri::command]
pub async fn get_spending_targets_progress(
    db_pool: tauri::State<'_, DbPool>,
    period: Option<String>,
    custom_start: Option<String>,
    custom_end: Option<String>,
) -> Result<TargetsProgress, String> {
    get_spending_targets_progress_impl(&db_pool.0, period, custom_start, custom_end).await
}

// T074: create_spending_target
pub async fn create_spending_target_impl(
    db: &SqlitePool,
    category_id: i64,
    amount: f64,
    period: &str,
    start_date: &str,
    end_date: Option<&str>,
) -> Result<i64, String> {
    TargetTracker::create_target(
        db,
        category_id,
        amount,
        period,
        start_date,
        end_date,
    )
    .await
}

#[tauri::command]
pub async fn create_spending_target(
    db_pool: tauri::State<'_, DbPool>,
    category_id: i64,
    amount: f64,
    period: String,
    start_date: String,
    end_date: Option<String>,
) -> Result<i64, String> {
    create_spending_target_impl(
        &db_pool.0,
        category_id,
        amount,
        &period,
        &start_date,
        end_date.as_deref(),
    )
    .await
}

// T075: update_spending_target
#[derive(Debug, Serialize)]
pub struct UpdateTargetResponse {
    pub success: bool,
}

pub async fn update_spending_target_impl(
    db: &SqlitePool,
    target_id: i64,
    amount: Option<f64>,
    end_date: Option<&str>,
) -> Result<UpdateTargetResponse, String> {
    let success = TargetTracker::update_target(db, target_id, amount, end_date).await?;
    Ok(UpdateTargetResponse { success })
}

#[tauri::command]
pub async fn update_spending_target(
    db_pool: tauri::State<'_, DbPool>,
    target_id: i64,
    amount: Option<f64>,
    end_date: Option<String>,
) -> Result<UpdateTargetResponse, String> {
    update_spending_target_impl(&db_pool.0, target_id, amount, end_date.as_deref()).await
}

// T076: get_dashboard_summary
#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub period: DatePeriod,
    pub total_spending: f64,
    pub total_income: f64,
    pub net: f64,
    pub top_categories: Vec<CategorySpending>,
    pub debt_summary: DebtSummary,
    pub target_summary: TargetSummary,
}

#[derive(Debug, Serialize)]
pub struct DatePeriod {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize)]
pub struct DebtSummary {
    pub total_debt: f64,
    pub total_monthly_payment: f64,
    pub next_payoff_date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TargetSummary {
    pub on_track_count: i64,
    pub over_count: i64,
    pub total_variance: f64,
}

pub async fn get_dashboard_summary_impl(
    db: &SqlitePool,
    period: &str,
) -> Result<DashboardSummary, String> {
    // Calculate date range
    let (start_date, end_date) = match period {
        "current_month" => {
            let now = chrono::Local::now().naive_local();
            let start = now.format("%Y-%m-01").to_string();
            let end = now.format("%Y-%m-%d").to_string();
            (start, end)
        }
        "last_30_days" => {
            let now = chrono::Local::now().naive_local();
            let start = (now - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
            let end = now.format("%Y-%m-%d").to_string();
            (start, end)
        }
        "current_year" => {
            let now = chrono::Local::now().naive_local();
            let start = format!("{}-01-01", now.year());
            let end = now.format("%Y-%m-%d").to_string();
            (start, end)
        }
        _ => return Err(format!("Invalid period: {}", period)),
    };

    // Get spending and income
    let total_spending = SpendingAggregator::get_total_spending(db, &start_date, &end_date).await?;
    let total_income = SpendingAggregator::get_total_income(db, &start_date, &end_date).await?;
    let net = total_income - total_spending;

    // Get top 5 categories
    let top_categories = SpendingAggregator::get_top_categories(db, &start_date, &end_date, 5).await?;

    // Get debt summary
    let total_debt = sqlx::query_as::<_, (f64,)>(
        "SELECT COALESCE(SUM(balance), 0) FROM debts"
    )
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "calculate total debt for dashboard"))?
    .0;

    let total_monthly_payment = sqlx::query_as::<_, (f64,)>(
        "SELECT COALESCE(SUM(min_payment), 0) FROM debts"
    )
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "calculate total debt payments for dashboard"))?
    .0;

    // Get target summary
    let targets = TargetTracker::get_targets_progress(db, &start_date, &end_date).await?;
    let on_track_count = targets.targets.iter().filter(|t| t.status == "on_track").count() as i64;
    let over_count = targets.targets.iter().filter(|t| t.status == "over").count() as i64;
    let total_variance: f64 = targets.targets.iter().map(|t| t.variance).sum();

    Ok(DashboardSummary {
        period: DatePeriod { start_date, end_date },
        total_spending,
        total_income,
        net,
        top_categories,
        debt_summary: DebtSummary {
            total_debt,
            total_monthly_payment,
            next_payoff_date: None, // TODO: Calculate from active plan
        },
        target_summary: TargetSummary {
            on_track_count,
            over_count,
            total_variance,
        },
    })
}

#[tauri::command]
pub async fn get_dashboard_summary(
    db_pool: tauri::State<'_, DbPool>,
    period: String,
) -> Result<DashboardSummary, String> {
    get_dashboard_summary_impl(&db_pool.0, &period).await
}

// T077: export_analytics_report
#[derive(Debug, Serialize)]
pub struct ExportReportResponse {
    pub success: bool,
    pub file_path: String,
    pub file_size: u64,
}

pub async fn export_analytics_report_impl(
    db: &SqlitePool,
    format: &str,
    start_date: &str,
    end_date: &str,
    _include_charts: bool,
    output_path: &str,
) -> Result<ExportReportResponse, String> {
    // Get analytics data
    let spending_data = SpendingAggregator::get_spending_by_category(db, start_date, end_date, None).await?;

    match format {
        "pdf" => {
            // For now, create a text-based report
            // TODO: Implement actual PDF generation
            let content = format!(
                "Budget Balancer Analytics Report\n\
                 Period: {} to {}\n\
                 \n\
                 Total Spending: ${:.2}\n\
                 \n\
                 Categories:\n",
                start_date, end_date, spending_data.total_spending
            );

            let mut full_content = content;
            for cat in spending_data.categories {
                full_content.push_str(&format!(
                    "  - {}: ${:.2} ({:.1}%)\n",
                    cat.category_name, cat.amount, cat.percentage
                ));
            }

            std::fs::write(output_path, full_content)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }
        "xlsx" => {
            // For now, create a CSV-like format
            // TODO: Implement actual XLSX generation
            let mut content = String::from("Category,Amount,Percentage\n");
            for cat in spending_data.categories {
                content.push_str(&format!(
                    "{},{:.2},{:.1}\n",
                    cat.category_name, cat.amount, cat.percentage
                ));
            }

            std::fs::write(output_path, content)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }
        _ => return Err(format!("Unsupported format: {}", format)),
    }

    let metadata = std::fs::metadata(output_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;

    Ok(ExportReportResponse {
        success: true,
        file_path: output_path.to_string(),
        file_size: metadata.len(),
    })
}

#[tauri::command]
pub async fn export_analytics_report(
    db_pool: tauri::State<'_, DbPool>,
    format: String,
    start_date: String,
    end_date: String,
    include_charts: bool,
    output_path: String,
) -> Result<ExportReportResponse, String> {
    export_analytics_report_impl(
        &db_pool.0,
        &format,
        &start_date,
        &end_date,
        include_charts,
        &output_path,
    )
    .await
}
