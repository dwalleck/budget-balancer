use crate::constants::{MAX_PAYOFF_YEARS, MONTHS_PER_YEAR, PERCENT_TO_DECIMAL_DIVISOR};
use crate::errors::DebtError;
use crate::models::debt::Debt;
use crate::services::avalanche_calculator::{DebtPaymentDetail, DebtSummary, MonthlyPayment, PayoffPlan};

#[derive(Debug, Clone)]
struct DebtState {
    id: i64,
    name: String,
    balance: f64,
    interest_rate: f64,
    min_payment: f64,
    total_interest_paid: f64,
    payoff_month: Option<i32>,
}

pub struct SnowballCalculator;

impl SnowballCalculator {
    pub fn calculate_payoff_plan(debts: Vec<Debt>, monthly_amount: f64) -> Result<PayoffPlan, DebtError> {
        if debts.is_empty() {
            return Err(DebtError::NoDebts);
        }

        // Validate monthly amount covers minimum payments
        let total_min_payments: f64 = debts.iter().map(|d| d.min_payment).sum();
        if monthly_amount < total_min_payments {
            return Err(DebtError::InsufficientFunds {
                monthly: monthly_amount,
                min_payments: total_min_payments,
            });
        }

        // Initialize debt states sorted by balance (lowest first - snowball strategy)
        let mut debt_states: Vec<DebtState> = debts
            .iter()
            .map(|d| DebtState {
                id: d.id,
                name: d.name.clone(),
                balance: d.balance,
                interest_rate: d.interest_rate,
                min_payment: d.min_payment,
                total_interest_paid: 0.0,
                payoff_month: None,
            })
            .collect();

        let mut monthly_breakdown = Vec::new();
        let mut month: i32 = 1;
        let start_date = chrono::Local::now().date_naive();

        // Simulate month-by-month payments until all debts paid off
        while debt_states.iter().any(|d| d.balance > 0.01) {
            let current_date = start_date + chrono::Duration::days(((month - 1) * 30) as i64);

            // Sort by balance (lowest first) at the beginning of each month
            debt_states.sort_by(|a, b| {
                if a.balance < 0.01 && b.balance < 0.01 {
                    std::cmp::Ordering::Equal
                } else if a.balance < 0.01 {
                    std::cmp::Ordering::Greater
                } else if b.balance < 0.01 {
                    std::cmp::Ordering::Less
                } else {
                    a.balance.partial_cmp(&b.balance).unwrap_or(std::cmp::Ordering::Equal)
                }
            });

            // Apply interest to all debts
            for debt in &mut debt_states {
                if debt.balance > 0.01 {
                    let monthly_interest = debt.balance * (debt.interest_rate / PERCENT_TO_DECIMAL_DIVISOR / MONTHS_PER_YEAR);
                    debt.balance += monthly_interest;
                    debt.total_interest_paid += monthly_interest;
                }
            }

            let mut remaining_amount = monthly_amount;
            let mut payments = Vec::new();

            // Pay minimums on all debts first
            for debt in &mut debt_states {
                if debt.balance > 0.01 {
                    let payment = debt.min_payment.min(debt.balance);
                    debt.balance -= payment;
                    remaining_amount -= payment;
                    payments.push(DebtPaymentDetail {
                        debt_id: debt.id,
                        debt_name: debt.name.clone(),
                        amount: payment,
                    });

                    if debt.balance < 0.01 && debt.payoff_month.is_none() {
                        debt.payoff_month = Some(month);
                    }
                }
            }

            // Allocate extra payment to lowest balance debt with balance remaining
            if remaining_amount > 0.01 {
                if let Some(target_debt) = debt_states.iter_mut().find(|d| d.balance > 0.01) {
                    let extra_payment = remaining_amount.min(target_debt.balance);
                    target_debt.balance -= extra_payment;

                    // Add to existing payment or create new one
                    if let Some(payment_detail) = payments.iter_mut().find(|p| p.debt_id == target_debt.id) {
                        payment_detail.amount += extra_payment;
                    } else {
                        payments.push(DebtPaymentDetail {
                            debt_id: target_debt.id,
                            debt_name: target_debt.name.clone(),
                            amount: extra_payment,
                        });
                    }

                    if target_debt.balance < 0.01 && target_debt.payoff_month.is_none() {
                        target_debt.payoff_month = Some(month);
                    }
                }
            }

            let total_paid: f64 = payments.iter().map(|p| p.amount).sum();
            let remaining_balance: f64 = debt_states.iter().map(|d| d.balance).sum();

            monthly_breakdown.push(MonthlyPayment {
                month,
                date: current_date.format("%Y-%m-%d").to_string(),
                payments,
                total_paid,
                remaining_balance,
            });

            month += 1;

            // Safety check: prevent infinite loops
            if month > (MAX_PAYOFF_YEARS * MONTHS_PER_YEAR as i32) {
                return Err(DebtError::PayoffExceeded(MAX_PAYOFF_YEARS));
            }
        }

        let total_interest: f64 = debt_states.iter().map(|d| d.total_interest_paid).sum();
        let payoff_date = monthly_breakdown.last().map(|m| m.date.clone()).unwrap_or_default();

        let debt_summaries: Vec<DebtSummary> = debt_states
            .iter()
            .map(|d| DebtSummary {
                debt_id: d.id,
                debt_name: d.name.clone(),
                payoff_month: d.payoff_month.unwrap_or(0),
                total_interest_paid: d.total_interest_paid,
            })
            .collect();

        Ok(PayoffPlan {
            strategy: "snowball".to_string(),
            payoff_date,
            total_interest,
            monthly_breakdown,
            debt_summaries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowball_calculation_prioritizes_low_balance() {
        let debts = vec![
            Debt {
                id: 1,
                name: "Small Balance Card".to_string(),
                balance: 500.0,
                original_balance: 500.0,
                interest_rate: 20.0,
                min_payment: 25.0,
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
            },
            Debt {
                id: 2,
                name: "Large Balance Card".to_string(),
                balance: 2000.0,
                original_balance: 2000.0,
                interest_rate: 10.0,
                min_payment: 25.0,
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
            },
        ];

        let plan = SnowballCalculator::calculate_payoff_plan(debts, 200.0).unwrap();

        assert_eq!(plan.strategy, "snowball");
        assert!(plan.total_interest > 0.0);
        assert!(!plan.monthly_breakdown.is_empty());

        // First month should have extra payment going to smallest balance debt (id: 1)
        let first_month = &plan.monthly_breakdown[0];
        let small_balance_payment = first_month.payments.iter().find(|p| p.debt_id == 1).unwrap();
        let large_balance_payment = first_month.payments.iter().find(|p| p.debt_id == 2).unwrap();

        // Small balance debt should get more than minimum
        assert!(small_balance_payment.amount > 25.0);
        // Large balance debt should get only minimum
        assert_eq!(large_balance_payment.amount, 25.0);

        // Verify total_paid and remaining_balance are calculated correctly
        assert_eq!(first_month.total_paid, 200.0);
        assert!(first_month.remaining_balance > 0.0);
        assert!(first_month.remaining_balance < 2500.0);
    }

    #[test]
    fn test_insufficient_monthly_amount_returns_error() {
        let debts = vec![Debt {
            id: 1,
            name: "Card".to_string(),
            balance: 1000.0,
            original_balance: 1000.0,
            interest_rate: 15.0,
            min_payment: 50.0,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
        }];

        let result = SnowballCalculator::calculate_payoff_plan(debts, 25.0);
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("Insufficient funds"));
    }
}
