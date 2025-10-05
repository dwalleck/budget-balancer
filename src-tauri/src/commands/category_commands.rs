use crate::models::category::{Category, NewCategory};
use crate::DbPool;
use sqlx::SqlitePool;

// Business logic functions (used by both commands and tests)

pub async fn list_categories_impl(db: &SqlitePool) -> Result<Vec<Category>, String> {
    sqlx::query_as::<_, Category>(
        "SELECT id, name, type, parent_id, icon, created_at FROM categories ORDER BY name"
    )
    .fetch_all(db)
    .await
    .map_err(|e| {
        eprintln!("Database error loading categories: {}", e);
        "Failed to load categories".to_string()
    })
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
        eprintln!("Database error creating category: {}", e);
        "Failed to create category".to_string()
    })?;

    Ok(result.last_insert_rowid())
}

// Tauri command handlers (extract pool from managed state)

#[tauri::command]
pub async fn list_categories(db_pool: tauri::State<'_, DbPool>) -> Result<Vec<Category>, String> {
    list_categories_impl(&db_pool.0).await
}

#[tauri::command]
pub async fn create_category(
    db_pool: tauri::State<'_, DbPool>,
    category: NewCategory,
) -> Result<i64, String> {
    create_category_impl(&db_pool.0, category).await
}
