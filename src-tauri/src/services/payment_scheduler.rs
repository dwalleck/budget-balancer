use crate::models::debt::Debt;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledPayment {
    pub debt_id: i64,
    pub debt_name: String,
    pub amount: f64,
    pub due_date: String,
    pub is_minimum: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSchedule {
    pub month: String,         // YYYY-MM format
    pub total_amount: f64,
    pub payments: Vec<ScheduledPayment>,
}

pub struct PaymentScheduler;

impl PaymentScheduler {
    /// Generate a payment schedule for the current month based on debts
    pub fn generate_monthly_schedule(debts: Vec<Debt>) -> Vec<ScheduledPayment> {
        let today = chrono::Local::now().date_naive();
        let year = today.year();
        let month = today.month();

        // Calculate due date (e.g., 15th of the month)
        let due_day = 15u32;
        let due_date = NaiveDate::from_ymd_opt(year, month, due_day.min(28))
            .unwrap_or(today)
            .format("%Y-%m-%d")
            .to_string();

        debts
            .into_iter()
            .filter(|d| d.balance > 0.0)
            .map(|d| ScheduledPayment {
                debt_id: d.id,
                debt_name: d.name,
                amount: d.min_payment,
                due_date: due_date.clone(),
                is_minimum: true,
            })
            .collect()
    }

    /// Generate payment schedules for the next N months
    pub fn generate_future_schedules(debts: Vec<Debt>, months_ahead: u32) -> Vec<PaymentSchedule> {
        let today = chrono::Local::now().date_naive();
        let mut schedules = Vec::new();

        for month_offset in 0..months_ahead {
            let target_date = if month_offset == 0 {
                today
            } else {
                // Add months
                let new_month = today.month() + month_offset;
                let year_offset = (new_month - 1) / 12;
                let month = ((new_month - 1) % 12) + 1;
                let year = today.year() + year_offset as i32;

                NaiveDate::from_ymd_opt(year, month, 1)
                    .unwrap_or(today)
            };

            let year = target_date.year();
            let month = target_date.month();
            let month_str = format!("{:04}-{:02}", year, month);

            let due_day = 15u32;
            let due_date = NaiveDate::from_ymd_opt(year, month, due_day.min(28))
                .unwrap_or(target_date)
                .format("%Y-%m-%d")
                .to_string();

            let payments: Vec<ScheduledPayment> = debts
                .iter()
                .filter(|d| d.balance > 0.0)
                .map(|d| ScheduledPayment {
                    debt_id: d.id,
                    debt_name: d.name.clone(),
                    amount: d.min_payment,
                    due_date: due_date.clone(),
                    is_minimum: true,
                })
                .collect();

            let total_amount: f64 = payments.iter().map(|p| p.amount).sum();

            schedules.push(PaymentSchedule {
                month: month_str,
                total_amount,
                payments,
            });
        }

        schedules
    }

    /// Calculate the next due date for a debt payment
    pub fn get_next_due_date() -> String {
        let today = chrono::Local::now().date_naive();
        let year = today.year();
        let month = today.month();
        let day = today.day();

        let due_day = 15u32;

        // If we've passed the due date this month, return next month's due date
        let (target_year, target_month) = if day > due_day {
            if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            }
        } else {
            (year, month)
        };

        NaiveDate::from_ymd_opt(target_year, target_month, due_day.min(28))
            .unwrap_or(today)
            .format("%Y-%m-%d")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_monthly_schedule() {
        let debts = vec![
            Debt {
                id: 1,
                name: "Credit Card A".to_string(),
                balance: 1000.0,
                original_balance: 1000.0,
                interest_rate: 18.0,
                min_payment: 50.0,
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
            },
            Debt {
                id: 2,
                name: "Credit Card B".to_string(),
                balance: 2000.0,
                original_balance: 2000.0,
                interest_rate: 15.0,
                min_payment: 75.0,
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
            },
        ];

        let schedule = PaymentScheduler::generate_monthly_schedule(debts);

        assert_eq!(schedule.len(), 2);
        assert_eq!(schedule[0].debt_id, 1);
        assert_eq!(schedule[0].amount, 50.0);
        assert!(schedule[0].is_minimum);
        assert_eq!(schedule[1].debt_id, 2);
        assert_eq!(schedule[1].amount, 75.0);
    }

    #[test]
    fn test_generate_future_schedules() {
        let debts = vec![Debt {
            id: 1,
            name: "Card".to_string(),
            balance: 1000.0,
            original_balance: 1000.0,
            interest_rate: 18.0,
            min_payment: 50.0,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
        }];

        let schedules = PaymentScheduler::generate_future_schedules(debts, 3);

        assert_eq!(schedules.len(), 3);
        assert_eq!(schedules[0].payments.len(), 1);
        assert_eq!(schedules[0].total_amount, 50.0);
    }

    #[test]
    fn test_next_due_date_format() {
        let due_date = PaymentScheduler::get_next_due_date();

        // Should be in YYYY-MM-DD format
        assert_eq!(due_date.len(), 10);
        assert_eq!(&due_date[4..5], "-");
        assert_eq!(&due_date[7..8], "-");
    }

    #[test]
    fn test_exclude_zero_balance_debts() {
        let debts = vec![
            Debt {
                id: 1,
                name: "Active Card".to_string(),
                balance: 1000.0,
                original_balance: 1000.0,
                interest_rate: 18.0,
                min_payment: 50.0,
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
            },
            Debt {
                id: 2,
                name: "Paid Off Card".to_string(),
                balance: 0.0,
                original_balance: 1000.0,
                interest_rate: 15.0,
                min_payment: 0.0,
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
            },
        ];

        let schedule = PaymentScheduler::generate_monthly_schedule(debts);

        // Should only include the debt with positive balance
        assert_eq!(schedule.len(), 1);
        assert_eq!(schedule[0].debt_id, 1);
    }
}
