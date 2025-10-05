/// Utility functions for interest calculations

/// Calculate monthly interest on a balance given an annual interest rate
pub fn calculate_monthly_interest(balance: f64, annual_rate: f64) -> f64 {
    if balance <= 0.0 || annual_rate < 0.0 {
        return 0.0;
    }
    balance * (annual_rate / 100.0 / 12.0)
}

/// Calculate the total interest paid over a series of payments
pub fn calculate_total_interest(
    initial_balance: f64,
    final_balance: f64,
    total_payments: f64,
) -> f64 {
    if total_payments <= 0.0 {
        return 0.0;
    }
    // Total interest = (initial balance - final balance - total payments)
    // This accounts for the principal reduction
    let principal_paid = initial_balance - final_balance;
    if total_payments > principal_paid {
        total_payments - principal_paid
    } else {
        0.0
    }
}

/// Calculate effective annual rate from monthly interest rate
pub fn calculate_effective_annual_rate(monthly_rate: f64) -> f64 {
    ((1.0 + monthly_rate / 100.0).powi(12) - 1.0) * 100.0
}

/// Calculate new balance after applying monthly interest and payment
pub fn apply_payment_with_interest(
    balance: f64,
    annual_rate: f64,
    payment: f64,
) -> f64 {
    let interest = calculate_monthly_interest(balance, annual_rate);
    let new_balance = balance + interest - payment;
    new_balance.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monthly_interest_calculation() {
        let balance = 1000.0;
        let annual_rate = 18.0; // 18% APR
        let monthly_interest = calculate_monthly_interest(balance, annual_rate);

        // 18% / 12 = 1.5% per month
        // 1000 * 0.015 = 15.0
        assert!((monthly_interest - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_monthly_interest_zero_balance() {
        assert_eq!(calculate_monthly_interest(0.0, 18.0), 0.0);
    }

    #[test]
    fn test_monthly_interest_zero_rate() {
        assert_eq!(calculate_monthly_interest(1000.0, 0.0), 0.0);
    }

    #[test]
    fn test_apply_payment_with_interest() {
        let balance = 1000.0;
        let annual_rate = 18.0;
        let payment = 100.0;

        let new_balance = apply_payment_with_interest(balance, annual_rate, payment);

        // Balance after interest: 1000 + 15 = 1015
        // Balance after payment: 1015 - 100 = 915
        assert!((new_balance - 915.0).abs() < 0.01);
    }

    #[test]
    fn test_apply_payment_exceeding_balance() {
        let balance = 100.0;
        let annual_rate = 15.0;
        let payment = 200.0;

        let new_balance = apply_payment_with_interest(balance, annual_rate, payment);

        // Should not go negative
        assert_eq!(new_balance, 0.0);
    }

    #[test]
    fn test_total_interest_calculation() {
        let initial = 1000.0;
        let final_balance = 0.0;
        let total_payments = 1150.0;

        let total_interest = calculate_total_interest(initial, final_balance, total_payments);

        // Paid 1150 to eliminate 1000 debt = 150 in interest
        assert!((total_interest - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_effective_annual_rate() {
        let monthly_rate = 1.5; // 1.5% per month
        let ear = calculate_effective_annual_rate(monthly_rate);

        // EAR should be slightly higher than 1.5 * 12 = 18% due to compounding
        assert!(ear > 18.0);
        assert!(ear < 20.0);
    }
}
