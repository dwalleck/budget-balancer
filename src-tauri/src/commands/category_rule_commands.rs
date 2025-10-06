use crate::errors::sanitize_db_error;
use crate::models::category_rule::{
    CategoryRule, CategoryRuleFilter, CategoryRuleWithName, DeleteCategoryRuleResponse,
    NewCategoryRule, UpdateCategoryRule,
};
use crate::DbPool;
use sqlx::SqlitePool;

// Business logic functions (used by both commands and tests)

pub async fn create_category_rule_impl(
    db: &SqlitePool,
    rule: NewCategoryRule,
) -> Result<CategoryRule, String> {
    // Normalize pattern to lowercase
    let normalized_pattern = rule.pattern.to_lowercase();
    let priority = rule.priority.unwrap_or(0);

    // Verify category exists
    let category_exists = sqlx::query("SELECT id FROM categories WHERE id = ?")
        .bind(rule.category_id)
        .fetch_optional(db)
        .await
        .map_err(|e| sanitize_db_error(e, "check category exists"))?;

    if category_exists.is_none() {
        return Err(format!("Category with id {} not found", rule.category_id));
    }

    // Insert the rule
    let result = sqlx::query(
        "INSERT INTO category_rules (pattern, category_id, priority) VALUES (?, ?, ?)"
    )
    .bind(&normalized_pattern)
    .bind(rule.category_id)
    .bind(priority)
    .execute(db)
    .await
    .map_err(|e| sanitize_db_error(e, "create category rule"))?;

    let rule_id = result.last_insert_rowid();

    // Fetch and return the created rule
    sqlx::query_as::<_, CategoryRule>(
        "SELECT id, pattern, category_id, priority, created_at FROM category_rules WHERE id = ?"
    )
    .bind(rule_id)
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch created rule"))
}

pub async fn list_category_rules_impl(
    db: &SqlitePool,
    filter: Option<CategoryRuleFilter>,
) -> Result<Vec<CategoryRuleWithName>, String> {
    let query = match filter {
        Some(CategoryRuleFilter::ByCategoryId(category_id)) => {
            sqlx::query_as::<_, CategoryRuleWithName>(
                "SELECT cr.id, cr.pattern, cr.category_id, c.name as category_name, cr.priority, cr.created_at
                 FROM category_rules cr
                 JOIN categories c ON cr.category_id = c.id
                 WHERE cr.category_id = ?
                 ORDER BY cr.priority DESC, cr.created_at ASC"
            )
            .bind(category_id)
            .fetch_all(db)
            .await
        }
        None => {
            sqlx::query_as::<_, CategoryRuleWithName>(
                "SELECT cr.id, cr.pattern, cr.category_id, c.name as category_name, cr.priority, cr.created_at
                 FROM category_rules cr
                 JOIN categories c ON cr.category_id = c.id
                 ORDER BY cr.priority DESC, cr.created_at ASC"
            )
            .fetch_all(db)
            .await
        }
    };

    query.map_err(|e| sanitize_db_error(e, "load category rules"))
}

pub async fn update_category_rule_impl(
    db: &SqlitePool,
    update: UpdateCategoryRule,
) -> Result<CategoryRule, String> {
    // First, verify the rule exists
    let existing = sqlx::query_as::<_, CategoryRule>(
        "SELECT id, pattern, category_id, priority, created_at FROM category_rules WHERE id = ?"
    )
    .bind(update.id)
    .fetch_optional(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch category rule"))?;

    if existing.is_none() {
        return Err(format!("Category rule with id {} not found", update.id));
    }

    // If updating category_id, verify it exists
    if let Some(new_category_id) = update.category_id {
        let category_exists = sqlx::query("SELECT id FROM categories WHERE id = ?")
            .bind(new_category_id)
            .fetch_optional(db)
            .await
            .map_err(|e| sanitize_db_error(e, "check category exists"))?;

        if category_exists.is_none() {
            return Err(format!("Category with id {} not found", new_category_id));
        }
    }

    // Use match to handle different update combinations with static SQL
    match (&update.pattern, update.category_id, update.priority) {
        // All three fields
        (Some(pattern), Some(category_id), Some(priority)) => {
            let normalized_pattern = pattern.to_lowercase();
            sqlx::query("UPDATE category_rules SET pattern = ?, category_id = ?, priority = ? WHERE id = ?")
                .bind(&normalized_pattern)
                .bind(category_id)
                .bind(priority)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category rule"))?;
        }
        // Pattern + category_id
        (Some(pattern), Some(category_id), None) => {
            let normalized_pattern = pattern.to_lowercase();
            sqlx::query("UPDATE category_rules SET pattern = ?, category_id = ? WHERE id = ?")
                .bind(&normalized_pattern)
                .bind(category_id)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category rule"))?;
        }
        // Pattern + priority
        (Some(pattern), None, Some(priority)) => {
            let normalized_pattern = pattern.to_lowercase();
            sqlx::query("UPDATE category_rules SET pattern = ?, priority = ? WHERE id = ?")
                .bind(&normalized_pattern)
                .bind(priority)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category rule"))?;
        }
        // Category_id + priority
        (None, Some(category_id), Some(priority)) => {
            sqlx::query("UPDATE category_rules SET category_id = ?, priority = ? WHERE id = ?")
                .bind(category_id)
                .bind(priority)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category rule"))?;
        }
        // Pattern only
        (Some(pattern), None, None) => {
            let normalized_pattern = pattern.to_lowercase();
            sqlx::query("UPDATE category_rules SET pattern = ? WHERE id = ?")
                .bind(&normalized_pattern)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category rule"))?;
        }
        // Category_id only
        (None, Some(category_id), None) => {
            sqlx::query("UPDATE category_rules SET category_id = ? WHERE id = ?")
                .bind(category_id)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category rule"))?;
        }
        // Priority only
        (None, None, Some(priority)) => {
            sqlx::query("UPDATE category_rules SET priority = ? WHERE id = ?")
                .bind(priority)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category rule"))?;
        }
        // No fields provided
        (None, None, None) => {
            return Err("At least one field must be provided for update".to_string());
        }
    }

    // Fetch and return updated rule
    sqlx::query_as::<_, CategoryRule>(
        "SELECT id, pattern, category_id, priority, created_at FROM category_rules WHERE id = ?"
    )
    .bind(update.id)
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch updated rule"))
}

pub async fn delete_category_rule_impl(
    db: &SqlitePool,
    rule_id: i64,
) -> Result<DeleteCategoryRuleResponse, String> {
    // Verify the rule exists
    let existing = sqlx::query("SELECT id FROM category_rules WHERE id = ?")
        .bind(rule_id)
        .fetch_optional(db)
        .await
        .map_err(|e| sanitize_db_error(e, "check rule exists"))?;

    if existing.is_none() {
        return Err(format!("Category rule with id {} not found", rule_id));
    }

    // Delete the rule
    sqlx::query("DELETE FROM category_rules WHERE id = ?")
        .bind(rule_id)
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "delete category rule"))?;

    Ok(DeleteCategoryRuleResponse {
        success: true,
        deleted_rule_id: rule_id,
    })
}

// Tauri command handlers (extract pool from managed state)

#[tauri::command]
pub async fn create_category_rule(
    db_pool: tauri::State<'_, DbPool>,
    rule: NewCategoryRule,
) -> Result<CategoryRule, String> {
    create_category_rule_impl(&db_pool.0, rule).await
}

#[tauri::command]
pub async fn list_category_rules(
    db_pool: tauri::State<'_, DbPool>,
    filter: Option<CategoryRuleFilter>,
) -> Result<Vec<CategoryRuleWithName>, String> {
    list_category_rules_impl(&db_pool.0, filter).await
}

#[tauri::command]
pub async fn update_category_rule(
    db_pool: tauri::State<'_, DbPool>,
    update: UpdateCategoryRule,
) -> Result<CategoryRule, String> {
    update_category_rule_impl(&db_pool.0, update).await
}

#[tauri::command]
pub async fn delete_category_rule(
    db_pool: tauri::State<'_, DbPool>,
    rule_id: i64,
) -> Result<DeleteCategoryRuleResponse, String> {
    delete_category_rule_impl(&db_pool.0, rule_id).await
}
