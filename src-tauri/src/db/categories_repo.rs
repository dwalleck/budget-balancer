use crate::models::category::{Category, NewCategory};
use crate::models::category_rule::{CategoryRule, NewCategoryRule};
use rusqlite::{params, Connection, Result};

pub struct CategoriesRepo;

impl CategoriesRepo {
    pub fn create(conn: &Connection, category: &NewCategory) -> Result<i64> {
        conn.execute(
            "INSERT INTO categories (name, type, icon) VALUES (?1, ?2, ?3)",
            params![category.name, category.category_type, category.icon],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<Category>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, type, parent_id, icon, created_at
             FROM categories
             ORDER BY name",
        )?;

        let categories = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category_type: row.get(2)?,
                    parent_id: row.get(3)?,
                    icon: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(categories)
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Category> {
        conn.query_row(
            "SELECT id, name, type, parent_id, icon, created_at
             FROM categories WHERE id = ?1",
            [id],
            |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category_type: row.get(2)?,
                    parent_id: row.get(3)?,
                    icon: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
    }

    pub fn get_by_name(conn: &Connection, name: &str) -> Result<Category> {
        conn.query_row(
            "SELECT id, name, type, parent_id, icon, created_at
             FROM categories WHERE name = ?1",
            [name],
            |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category_type: row.get(2)?,
                    parent_id: row.get(3)?,
                    icon: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM categories WHERE id = ?1", [id])?;
        Ok(())
    }

    // Category Rules operations
    pub fn create_rule(conn: &Connection, rule: &NewCategoryRule) -> Result<i64> {
        conn.execute(
            "INSERT INTO category_rules (pattern, category_id, priority) VALUES (?1, ?2, ?3)",
            params![rule.pattern, rule.category_id, rule.priority],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_all_rules(conn: &Connection) -> Result<Vec<CategoryRule>> {
        let mut stmt = conn.prepare(
            "SELECT id, pattern, category_id, priority, created_at
             FROM category_rules
             ORDER BY priority DESC, pattern",
        )?;

        let rules = stmt
            .query_map([], |row| {
                Ok(CategoryRule {
                    id: row.get(0)?,
                    pattern: row.get(1)?,
                    category_id: row.get(2)?,
                    priority: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(rules)
    }

    pub fn list_rules_by_category(conn: &Connection, category_id: i64) -> Result<Vec<CategoryRule>> {
        let mut stmt = conn.prepare(
            "SELECT id, pattern, category_id, priority, created_at
             FROM category_rules
             WHERE category_id = ?1
             ORDER BY priority DESC, pattern",
        )?;

        let rules = stmt
            .query_map([category_id], |row| {
                Ok(CategoryRule {
                    id: row.get(0)?,
                    pattern: row.get(1)?,
                    category_id: row.get(2)?,
                    priority: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(rules)
    }

    pub fn delete_rule(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM category_rules WHERE id = ?1", [id])?;
        Ok(())
    }
}
