use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySpending {
    pub category_id: i64,
    pub category_name: String,
    pub category_icon: Option<String>,
    pub amount: f64,
    pub percentage: f64,
    pub transaction_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingByCategory {
    pub period: DatePeriod,
    pub categories: Vec<CategorySpending>,
    pub total_spending: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatePeriod {
    pub start_date: String,
    pub end_date: String,
}

pub struct SpendingAggregator;

impl SpendingAggregator {
    /// Calculate total spending per category for a time period
    pub async fn get_spending_by_category(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
        account_id: Option<i64>,
    ) -> Result<SpendingByCategory, String> {
        // Build query based on whether account filter is provided
        let query = if let Some(account_id) = account_id {
            sqlx::query_as::<_, (i64, String, Option<String>, f64, i64)>(
                "SELECT
                    c.id,
                    c.name,
                    c.icon,
                    CAST(COALESCE(SUM(ABS(t.amount)), 0) AS REAL) as total_amount,
                    COUNT(t.id) as transaction_count
                FROM categories c
                LEFT JOIN transactions t ON t.category_id = c.id
                    AND t.date >= ?
                    AND t.date <= ?
                    AND t.amount < 0
                    AND t.account_id = ?
                GROUP BY c.id, c.name, c.icon
                HAVING total_amount > 0
                ORDER BY total_amount DESC"
            )
            .bind(start_date)
            .bind(end_date)
            .bind(account_id)
            .fetch_all(db)
            .await
        } else {
            sqlx::query_as::<_, (i64, String, Option<String>, f64, i64)>(
                "SELECT
                    c.id,
                    c.name,
                    c.icon,
                    CAST(COALESCE(SUM(ABS(t.amount)), 0) AS REAL) as total_amount,
                    COUNT(t.id) as transaction_count
                FROM categories c
                LEFT JOIN transactions t ON t.category_id = c.id
                    AND t.date >= ?
                    AND t.date <= ?
                    AND t.amount < 0
                GROUP BY c.id, c.name, c.icon
                HAVING total_amount > 0
                ORDER BY total_amount DESC"
            )
            .bind(start_date)
            .bind(end_date)
            .fetch_all(db)
            .await
        };

        let rows = query.map_err(|e| e.to_string())?;

        // Calculate total spending
        let total_spending: f64 = rows.iter().map(|(_, _, _, amount, _)| amount).sum();

        // Build category spending list with percentages
        let categories = rows
            .into_iter()
            .map(|(id, name, icon, amount, count)| {
                let percentage = if total_spending > 0.0 {
                    (amount / total_spending) * 100.0
                } else {
                    0.0
                };

                CategorySpending {
                    category_id: id,
                    category_name: name,
                    category_icon: icon,
                    amount,
                    percentage,
                    transaction_count: count,
                }
            })
            .collect();

        Ok(SpendingByCategory {
            period: DatePeriod {
                start_date: start_date.to_string(),
                end_date: end_date.to_string(),
            },
            categories,
            total_spending,
        })
    }

    /// Get top N categories by spending amount
    pub async fn get_top_categories(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
        limit: i64,
    ) -> Result<Vec<CategorySpending>, String> {
        let result = Self::get_spending_by_category(db, start_date, end_date, None).await?;

        Ok(result.categories.into_iter().take(limit as usize).collect())
    }

    /// Calculate total income for a period
    pub async fn get_total_income(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
    ) -> Result<f64, String> {
        let result = sqlx::query_as::<_, (f64,)>(
            "SELECT CAST(COALESCE(SUM(amount), 0) AS REAL)
             FROM transactions
             WHERE date >= ? AND date <= ? AND amount > 0"
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.0)
    }

    /// Calculate total spending for a period
    pub async fn get_total_spending(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
    ) -> Result<f64, String> {
        let result = sqlx::query_as::<_, (f64,)>(
            "SELECT CAST(COALESCE(SUM(ABS(amount)), 0) AS REAL)
             FROM transactions
             WHERE date >= ? AND date <= ? AND amount < 0"
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.0)
    }
}
