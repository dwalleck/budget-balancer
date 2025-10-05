use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetProgress {
    pub category_id: i64,
    pub category_name: String,
    pub target_amount: f64,
    pub actual_amount: f64,
    pub remaining: f64,
    pub percentage_used: f64,
    pub status: String, // "under", "on_track", "over"
    pub variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetsProgress {
    pub period: DatePeriod,
    pub targets: Vec<TargetProgress>,
    pub overall_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatePeriod {
    pub start_date: String,
    pub end_date: String,
}

pub struct TargetTracker;

impl TargetTracker {
    /// Get progress against spending targets
    pub async fn get_targets_progress(
        db: &SqlitePool,
        start_date: &str,
        end_date: &str,
    ) -> Result<TargetsProgress, String> {
        // Get all active targets for the period
        let targets = sqlx::query_as::<_, (i64, i64, String, f64)>(
            "SELECT id, category_id, (SELECT name FROM categories WHERE id = category_id) as category_name, amount
             FROM spending_targets
             WHERE (start_date <= ? AND (end_date IS NULL OR end_date >= ?))"
        )
        .bind(end_date)
        .bind(start_date)
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

        let mut target_progress_list = Vec::new();
        let mut under_count = 0;
        let mut on_track_count = 0;
        let mut over_count = 0;

        for (_, category_id, category_name, target_amount) in targets {
            // Get actual spending for this category in the period
            let actual_amount = sqlx::query_as::<_, (f64,)>(
                "SELECT CAST(COALESCE(SUM(ABS(amount)), 0) AS REAL)
                 FROM transactions
                 WHERE category_id = ?
                   AND date >= ?
                   AND date <= ?
                   AND amount < 0"
            )
            .bind(category_id)
            .bind(start_date)
            .bind(end_date)
            .fetch_one(db)
            .await
            .map_err(|e| e.to_string())?
            .0;

            let remaining = target_amount - actual_amount;
            let percentage_used = if target_amount > 0.0 {
                (actual_amount / target_amount) * 100.0
            } else {
                0.0
            };
            let variance = actual_amount - target_amount;

            // Determine status
            // under: < 80%, on_track: 80-100%, over: > 100%
            let status = if percentage_used < 80.0 {
                under_count += 1;
                "under".to_string()
            } else if percentage_used <= 100.0 {
                on_track_count += 1;
                "on_track".to_string()
            } else {
                over_count += 1;
                "over".to_string()
            };

            target_progress_list.push(TargetProgress {
                category_id,
                category_name,
                target_amount,
                actual_amount,
                remaining,
                percentage_used,
                status,
                variance,
            });
        }

        // Determine overall status
        let overall_status = if over_count > 0 {
            "over".to_string()
        } else if on_track_count > 0 || under_count > 0 {
            if under_count > on_track_count {
                "under".to_string()
            } else {
                "on_track".to_string()
            }
        } else {
            "under".to_string()
        };

        Ok(TargetsProgress {
            period: DatePeriod {
                start_date: start_date.to_string(),
                end_date: end_date.to_string(),
            },
            targets: target_progress_list,
            overall_status,
        })
    }

    /// Create a spending target
    pub async fn create_target(
        db: &SqlitePool,
        category_id: i64,
        amount: f64,
        period: &str,
        start_date: &str,
        end_date: Option<&str>,
    ) -> Result<i64, String> {
        let result = sqlx::query(
            "INSERT INTO spending_targets (category_id, amount, period, start_date, end_date)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(category_id)
        .bind(amount)
        .bind(period)
        .bind(start_date)
        .bind(end_date)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    }

    /// Update a spending target
    pub async fn update_target(
        db: &SqlitePool,
        target_id: i64,
        amount: Option<f64>,
        end_date: Option<&str>,
    ) -> Result<bool, String> {
        // Check if target exists
        let exists = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM spending_targets WHERE id = ?")
            .bind(target_id)
            .fetch_one(db)
            .await
            .map_err(|e| e.to_string())?
            .0 > 0;

        if !exists {
            return Err("Target not found".to_string());
        }

        // Build update query based on what's being updated
        if let Some(amt) = amount {
            sqlx::query("UPDATE spending_targets SET amount = ? WHERE id = ?")
                .bind(amt)
                .bind(target_id)
                .execute(db)
                .await
                .map_err(|e| e.to_string())?;
        }

        if let Some(date) = end_date {
            sqlx::query("UPDATE spending_targets SET end_date = ? WHERE id = ?")
                .bind(date)
                .bind(target_id)
                .execute(db)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(true)
    }
}
