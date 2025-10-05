use tauri_plugin_sql::{Migration, MigrationKind};

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "Initial schema",
            sql: include_str!("../../migrations/001_initial_schema.sql"),
            kind: MigrationKind::Up,
        }
    ]
}

pub async fn init_database(_app: &tauri::AppHandle) -> Result<(), String> {
    // Database initialization happens automatically via plugin with migrations
    // This function can be used for additional setup if needed
    println!("Database initialized successfully");
    Ok(())
}
