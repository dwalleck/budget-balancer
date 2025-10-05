use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub date: String,
    pub amount: f64,
    pub transaction_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingTrends {
    pub data_points: Vec<TrendPoint>,
    pub total_spending: f64,
    pub average_per_interval: f64,
}

pub struct TrendsCalculator;

impl TrendsCalculator {
    /// Get spending trends over time with specified interval
    pub async fn get_spending_trends(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
        interval: &str,
        category_id: Option<i64>,
    ) -> Result<SpendingTrends, String> {
        let data_points = match interval {
            "daily" => Self::get_daily_trends(db, start_date, end_date, category_id).await?,
            "weekly" => Self::get_weekly_trends(db, start_date, end_date, category_id).await?,
            "monthly" => Self::get_monthly_trends(db, start_date, end_date, category_id).await?,
            _ => return Err(format!("Invalid interval: {}", interval)),
        };

        let total_spending: f64 = data_points.iter().map(|p| p.amount).sum();
        let average_per_interval = if !data_points.is_empty() {
            total_spending / data_points.len() as f64
        } else {
            0.0
        };

        Ok(SpendingTrends {
            data_points,
            total_spending,
            average_per_interval,
        })
    }

    async fn get_daily_trends(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
        category_id: Option<i64>,
    ) -> Result<Vec<TrendPoint>, String> {
        let query = if let Some(cat_id) = category_id {
            sqlx::query_as::<_, (String, f64, i64)>(
                "SELECT
                    date,
                    CAST(COALESCE(SUM(ABS(amount)), 0) AS REAL) as total,
                    COUNT(*) as count
                FROM transactions
                WHERE date >= ? AND date <= ? AND amount < 0 AND category_id = ?
                GROUP BY date
                ORDER BY date"
            )
            .bind(start_date)
            .bind(end_date)
            .bind(cat_id)
            .fetch_all(db)
            .await
        } else {
            sqlx::query_as::<_, (String, f64, i64)>(
                "SELECT
                    date,
                    CAST(COALESCE(SUM(ABS(amount)), 0) AS REAL) as total,
                    COUNT(*) as count
                FROM transactions
                WHERE date >= ? AND date <= ? AND amount < 0
                GROUP BY date
                ORDER BY date"
            )
            .bind(start_date)
            .bind(end_date)
            .fetch_all(db)
            .await
        };

        let rows = query.map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(|(date, amount, count)| TrendPoint {
                date,
                amount,
                transaction_count: count,
            })
            .collect())
    }

    async fn get_weekly_trends(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
        category_id: Option<i64>,
    ) -> Result<Vec<TrendPoint>, String> {
        // Get daily data and aggregate by week
        let daily_trends = Self::get_daily_trends(db, start_date, end_date, category_id).await?;

        let mut weekly_data: std::collections::HashMap<String, (f64, i64)> =
            std::collections::HashMap::new();

        for point in daily_trends {
            if let Ok(date) = NaiveDate::parse_from_str(&point.date, "%Y-%m-%d") {
                // Get the start of the week (Monday)
                let week_start = date - chrono::Duration::days(date.weekday().num_days_from_monday() as i64);
                let week_key = week_start.format("%Y-%m-%d").to_string();

                let entry = weekly_data.entry(week_key).or_insert((0.0, 0));
                entry.0 += point.amount;
                entry.1 += point.transaction_count;
            }
        }

        let mut result: Vec<TrendPoint> = weekly_data
            .into_iter()
            .map(|(date, (amount, count))| TrendPoint {
                date,
                amount,
                transaction_count: count,
            })
            .collect();

        result.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(result)
    }

    async fn get_monthly_trends(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
        category_id: Option<i64>,
    ) -> Result<Vec<TrendPoint>, String> {
        // Parse start and end dates
        let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid start_date: {}", e))?;
        let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid end_date: {}", e))?;

        // Generate all months in the range
        let mut months = Vec::new();
        let mut current = NaiveDate::from_ymd_opt(start.year(), start.month(), 1)
            .ok_or("Invalid start date")?;
        let end_month = NaiveDate::from_ymd_opt(end.year(), end.month(), 1)
            .ok_or("Invalid end date")?;

        while current <= end_month {
            months.push(current.format("%Y-%m-01").to_string());
            current = if current.month() == 12 {
                NaiveDate::from_ymd_opt(current.year() + 1, 1, 1)
                    .ok_or("Date calculation error")?
            } else {
                NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1)
                    .ok_or("Date calculation error")?
            };
        }

        // Get spending data for each month
        let mut result = Vec::new();
        for month_start in months {
            let query = if let Some(cat_id) = category_id {
                sqlx::query_as::<_, (f64, i64)>(
                    "SELECT
                        CAST(COALESCE(SUM(ABS(amount)), 0) AS REAL) as total,
                        COUNT(*) as count
                    FROM transactions
                    WHERE strftime('%Y-%m', date) = strftime('%Y-%m', ?)
                        AND amount < 0
                        AND category_id = ?"
                )
                .bind(&month_start)
                .bind(cat_id)
                .fetch_one(db)
                .await
            } else {
                sqlx::query_as::<_, (f64, i64)>(
                    "SELECT
                        CAST(COALESCE(SUM(ABS(amount)), 0) AS REAL) as total,
                        COUNT(*) as count
                    FROM transactions
                    WHERE strftime('%Y-%m', date) = strftime('%Y-%m', ?)
                        AND amount < 0"
                )
                .bind(&month_start)
                .fetch_one(db)
                .await
            };

            let (amount, count) = query.map_err(|e| e.to_string())?;

            result.push(TrendPoint {
                date: month_start,
                amount,
                transaction_count: count,
            });
        }

        Ok(result)
    }
}
