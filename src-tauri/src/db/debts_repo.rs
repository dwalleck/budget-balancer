use crate::models::debt::{Debt, DebtPayment, DebtPlan, NewDebt};
use rusqlite::{params, Connection, Result};

pub struct DebtsRepo;

impl DebtsRepo {
    pub fn create(conn: &Connection, debt: &NewDebt) -> Result<i64> {
        conn.execute(
            "INSERT INTO debts (name, balance, original_balance, interest_rate, min_payment)
             VALUES (?1, ?2, ?2, ?3, ?4)",
            params![debt.name, debt.balance, debt.interest_rate, debt.min_payment],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<Debt>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, balance, original_balance, interest_rate, min_payment, created_at, updated_at
             FROM debts
             ORDER BY balance DESC",
        )?;

        let debts = stmt
            .query_map([], |row| {
                Ok(Debt {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    balance: row.get(2)?,
                    original_balance: row.get(3)?,
                    interest_rate: row.get(4)?,
                    min_payment: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(debts)
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Debt> {
        conn.query_row(
            "SELECT id, name, balance, original_balance, interest_rate, min_payment, created_at, updated_at
             FROM debts WHERE id = ?1",
            [id],
            |row| {
                Ok(Debt {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    balance: row.get(2)?,
                    original_balance: row.get(3)?,
                    interest_rate: row.get(4)?,
                    min_payment: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
    }

    pub fn update(
        conn: &Connection,
        id: i64,
        balance: Option<f64>,
        interest_rate: Option<f64>,
        min_payment: Option<f64>,
    ) -> Result<()> {
        if let Some(bal) = balance {
            conn.execute(
                "UPDATE debts SET balance = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![bal, id],
            )?;
        }
        if let Some(rate) = interest_rate {
            conn.execute(
                "UPDATE debts SET interest_rate = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![rate, id],
            )?;
        }
        if let Some(payment) = min_payment {
            conn.execute(
                "UPDATE debts SET min_payment = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![payment, id],
            )?;
        }
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM debts WHERE id = ?1", [id])?;
        Ok(())
    }

    // Debt Payment operations
    pub fn create_payment(
        conn: &Connection,
        debt_id: i64,
        amount: f64,
        date: &str,
        plan_id: Option<i64>,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO debt_payments (debt_id, amount, date, plan_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![debt_id, amount, date, plan_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_payments_by_debt(conn: &Connection, debt_id: i64) -> Result<Vec<DebtPayment>> {
        let mut stmt = conn.prepare(
            "SELECT id, debt_id, amount, date, plan_id, created_at
             FROM debt_payments
             WHERE debt_id = ?1
             ORDER BY date DESC",
        )?;

        let payments = stmt
            .query_map([debt_id], |row| {
                Ok(DebtPayment {
                    id: row.get(0)?,
                    debt_id: row.get(1)?,
                    amount: row.get(2)?,
                    date: row.get(3)?,
                    plan_id: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(payments)
    }

    pub fn list_payments_by_date_range(
        conn: &Connection,
        debt_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<DebtPayment>> {
        let mut stmt = conn.prepare(
            "SELECT id, debt_id, amount, date, plan_id, created_at
             FROM debt_payments
             WHERE debt_id = ?1 AND date >= ?2 AND date <= ?3
             ORDER BY date DESC",
        )?;

        let payments = stmt
            .query_map(params![debt_id, start_date, end_date], |row| {
                Ok(DebtPayment {
                    id: row.get(0)?,
                    debt_id: row.get(1)?,
                    amount: row.get(2)?,
                    date: row.get(3)?,
                    plan_id: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(payments)
    }

    // Debt Plan operations
    pub fn create_plan(conn: &Connection, strategy: &str, monthly_amount: f64) -> Result<i64> {
        conn.execute(
            "INSERT INTO debt_plans (strategy, monthly_amount)
             VALUES (?1, ?2)",
            params![strategy, monthly_amount],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_plan_by_id(conn: &Connection, id: i64) -> Result<DebtPlan> {
        conn.query_row(
            "SELECT id, strategy, monthly_amount, created_at, updated_at
             FROM debt_plans WHERE id = ?1",
            [id],
            |row| {
                Ok(DebtPlan {
                    id: row.get(0)?,
                    strategy: row.get(1)?,
                    monthly_amount: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
    }

    pub fn list_all_plans(conn: &Connection) -> Result<Vec<DebtPlan>> {
        let mut stmt = conn.prepare(
            "SELECT id, strategy, monthly_amount, created_at, updated_at
             FROM debt_plans
             ORDER BY created_at DESC",
        )?;

        let plans = stmt
            .query_map([], |row| {
                Ok(DebtPlan {
                    id: row.get(0)?,
                    strategy: row.get(1)?,
                    monthly_amount: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(plans)
    }
}
