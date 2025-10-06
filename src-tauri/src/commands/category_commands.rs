use crate::errors::sanitize_db_error;
use crate::models::category::{Category, CategoryFilter, DeleteCategoryResponse, NewCategory, UpdateCategory};
use crate::DbPool;
use sqlx::SqlitePool;

// Business logic functions (used by both commands and tests)

pub async fn list_categories_impl(
    db: &SqlitePool,
    filter: Option<CategoryFilter>,
) -> Result<Vec<Category>, String> {
    let query = match filter {
        Some(CategoryFilter::Predefined) => {
            "SELECT id, name, type, parent_id, icon, created_at FROM categories WHERE type = 'predefined' ORDER BY name"
        }
        Some(CategoryFilter::Custom) => {
            "SELECT id, name, type, parent_id, icon, created_at FROM categories WHERE type = 'custom' ORDER BY name"
        }
        None => {
            "SELECT id, name, type, parent_id, icon, created_at FROM categories ORDER BY name"
        }
    };

    sqlx::query_as::<_, Category>(query)
        .fetch_all(db)
        .await
        .map_err(|e| sanitize_db_error(e, "load categories"))
}

pub async fn create_category_impl(
    db: &SqlitePool,
    category: NewCategory,
) -> Result<i64, String> {
    let result = sqlx::query(
        "INSERT INTO categories (name, type, icon) VALUES (?, 'custom', ?)"
    )
    .bind(&category.name)
    .bind(&category.icon)
    .execute(db)
    .await
    .map_err(|e| {
        // Check for unique constraint violation
        let error_msg = e.to_string();
        if error_msg.to_lowercase().contains("unique") {
            format!("Category with name '{}' already exists", category.name)
        } else {
            sanitize_db_error(e, "create category")
        }
    })?;

    Ok(result.last_insert_rowid())
}

pub async fn update_category_impl(
    db: &SqlitePool,
    update: UpdateCategory,
) -> Result<Category, String> {
    // First, verify the category exists and is custom
    let existing = sqlx::query_as::<_, Category>(
        "SELECT id, name, type, parent_id, icon, created_at FROM categories WHERE id = ?"
    )
    .bind(update.id)
    .fetch_optional(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch category"))?;

    let existing = existing.ok_or_else(|| format!("Category with id {} not found", update.id))?;

    if existing.r#type == "predefined" {
        return Err("Cannot modify predefined categories".to_string());
    }

    // Use match to handle different update combinations with static SQL
    match (&update.name, &update.icon) {
        (Some(name), Some(icon)) => {
            sqlx::query("UPDATE categories SET name = ?, icon = ? WHERE id = ?")
                .bind(name)
                .bind(icon)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category"))?;
        }
        (Some(name), None) => {
            sqlx::query("UPDATE categories SET name = ? WHERE id = ?")
                .bind(name)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category"))?;
        }
        (None, Some(icon)) => {
            sqlx::query("UPDATE categories SET icon = ? WHERE id = ?")
                .bind(icon)
                .bind(update.id)
                .execute(db)
                .await
                .map_err(|e| sanitize_db_error(e, "update category"))?;
        }
        (None, None) => {
            return Err("At least one field (name or icon) must be provided for update".to_string());
        }
    }

    // Fetch and return updated category
    sqlx::query_as::<_, Category>(
        "SELECT id, name, type, parent_id, icon, created_at FROM categories WHERE id = ?"
    )
    .bind(update.id)
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch updated category"))
}

pub async fn delete_category_impl(
    db: &SqlitePool,
    category_id: i64,
) -> Result<DeleteCategoryResponse, String> {
    // First, verify the category exists and is custom
    let existing = sqlx::query_as::<_, Category>(
        "SELECT id, name, type, parent_id, icon, created_at FROM categories WHERE id = ?"
    )
    .bind(category_id)
    .fetch_optional(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch category"))?;

    let existing = existing.ok_or_else(|| format!("Category with id {} not found", category_id))?;

    if existing.r#type == "predefined" {
        return Err("Cannot delete predefined categories".to_string());
    }

    // Get Uncategorized category ID
    let uncategorized_id = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM categories WHERE name = 'Uncategorized' LIMIT 1"
    )
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "fetch Uncategorized category"))?
    .0;

    // Count transactions that will be reassigned
    let count = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM transactions WHERE category_id = ?"
    )
    .bind(category_id)
    .fetch_one(db)
    .await
    .map_err(|e| sanitize_db_error(e, "count transactions"))?
    .0;

    // Reassign transactions to Uncategorized
    sqlx::query("UPDATE transactions SET category_id = ? WHERE category_id = ?")
        .bind(uncategorized_id)
        .bind(category_id)
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "reassign transactions"))?;

    // Delete the category
    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(category_id)
        .execute(db)
        .await
        .map_err(|e| sanitize_db_error(e, "delete category"))?;

    Ok(DeleteCategoryResponse {
        success: true,
        deleted_category_id: category_id,
        reassigned_transactions_count: count,
    })
}

// Tauri command handlers (extract pool from managed state)

#[tauri::command]
pub async fn list_categories(
    db_pool: tauri::State<'_, DbPool>,
    filter: Option<CategoryFilter>,
) -> Result<Vec<Category>, String> {
    list_categories_impl(&db_pool.0, filter).await
}

#[tauri::command]
pub async fn create_category(
    db_pool: tauri::State<'_, DbPool>,
    category: NewCategory,
) -> Result<i64, String> {
    create_category_impl(&db_pool.0, category).await
}

#[tauri::command]
pub async fn update_category(
    db_pool: tauri::State<'_, DbPool>,
    update: UpdateCategory,
) -> Result<Category, String> {
    update_category_impl(&db_pool.0, update).await
}

#[tauri::command]
pub async fn delete_category(
    db_pool: tauri::State<'_, DbPool>,
    category_id: i64,
) -> Result<DeleteCategoryResponse, String> {
    delete_category_impl(&db_pool.0, category_id).await
}
