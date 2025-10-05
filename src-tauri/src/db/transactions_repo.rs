use crate::models::transaction::{NewTransaction, Transaction};
use rusqlite::{params, Connection, Result};

pub struct TransactionsRepo;

impl TransactionsRepo {
    pub fn create(conn: &Connection, transaction: &NewTransaction) -> Result<i64> {
        conn.execute(
            "INSERT INTO transactions (account_id, category_id, date, amount, description, merchant, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                transaction.account_id,
                transaction.category_id,
                transaction.date,
                transaction.amount,
                transaction.description,
                transaction.merchant,
                transaction.hash,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<Transaction>> {
        let mut stmt = conn.prepare(
            "SELECT id, account_id, category_id, date, amount, description, merchant, hash, created_at
             FROM transactions
             ORDER BY date DESC",
        )?;

        let transactions = stmt
            .query_map([], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    category_id: row.get(2)?,
                    date: row.get(3)?,
                    amount: row.get(4)?,
                    description: row.get(5)?,
                    merchant: row.get(6)?,
                    hash: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(transactions)
    }

    pub fn list_by_account(conn: &Connection, account_id: i64) -> Result<Vec<Transaction>> {
        let mut stmt = conn.prepare(
            "SELECT id, account_id, category_id, date, amount, description, merchant, hash, created_at
             FROM transactions
             WHERE account_id = ?1
             ORDER BY date DESC",
        )?;

        let transactions = stmt
            .query_map([account_id], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    category_id: row.get(2)?,
                    date: row.get(3)?,
                    amount: row.get(4)?,
                    description: row.get(5)?,
                    merchant: row.get(6)?,
                    hash: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(transactions)
    }

    pub fn list_by_date_range(
        conn: &Connection,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<Transaction>> {
        let mut stmt = conn.prepare(
            "SELECT id, account_id, category_id, date, amount, description, merchant, hash, created_at
             FROM transactions
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date DESC",
        )?;

        let transactions = stmt
            .query_map(params![start_date, end_date], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    category_id: row.get(2)?,
                    date: row.get(3)?,
                    amount: row.get(4)?,
                    description: row.get(5)?,
                    merchant: row.get(6)?,
                    hash: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(transactions)
    }

    pub fn update_category(conn: &Connection, id: i64, category_id: i64) -> Result<()> {
        conn.execute(
            "UPDATE transactions SET category_id = ?1 WHERE id = ?2",
            params![category_id, id],
        )?;
        Ok(())
    }

    pub fn exists_by_hash(conn: &Connection, hash: &str) -> Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE hash = ?1",
            [hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
        Ok(())
    }
}
