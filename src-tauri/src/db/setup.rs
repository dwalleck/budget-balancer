use sqlx::SqlitePool;

pub async fn initialize_database(pool: &SqlitePool) -> Result<(), String> {
    // Create accounts table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            type TEXT NOT NULL CHECK(type IN ('checking', 'savings', 'credit_card')),
            balance REAL NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create accounts table: {}", e))?;

    // Create categories table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            type TEXT NOT NULL CHECK(type IN ('predefined', 'custom')),
            parent_id INTEGER,
            icon TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create categories table: {}", e))?;

    // Create transactions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            date TEXT NOT NULL,
            amount REAL NOT NULL,
            description TEXT NOT NULL,
            merchant TEXT,
            hash TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create transactions table: {}", e))?;

    // Create category_rules table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS category_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pattern TEXT NOT NULL,
            category_id INTEGER NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create category_rules table: {}", e))?;

    // Insert default categories
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO categories (id, name, type, icon) VALUES
            (1, 'Groceries', 'predefined', '🛒'),
            (2, 'Dining', 'predefined', '🍽️'),
            (3, 'Transportation', 'predefined', '🚗'),
            (4, 'Entertainment', 'predefined', '🎬'),
            (5, 'Utilities', 'predefined', '⚡'),
            (6, 'Healthcare', 'predefined', '🏥'),
            (7, 'Shopping', 'predefined', '🛍️'),
            (8, 'Travel', 'predefined', '✈️'),
            (9, 'Income', 'predefined', '💰'),
            (10, 'Uncategorized', 'predefined', '❓')
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert default categories: {}", e))?;

    // Insert category rules for auto-categorization
    let rules = vec![
        // Groceries
        ("walmart", 1, 10),
        ("target", 1, 10),
        ("whole foods", 1, 10),
        ("trader joe", 1, 10),
        ("safeway", 1, 10),
        ("kroger", 1, 10),
        // Dining
        ("starbucks", 2, 10),
        ("mcdonalds", 2, 10),
        ("chipotle", 2, 10),
        ("restaurant", 2, 10),
        ("cafe", 2, 10),
        // Transportation
        ("uber", 3, 10),
        ("lyft", 3, 10),
        ("shell", 3, 10),
        ("gas station", 3, 10),
        // Entertainment
        ("netflix", 4, 10),
        ("spotify", 4, 10),
        ("movie", 4, 10),
        // Shopping
        ("amazon", 7, 10),
        ("ebay", 7, 10),
    ];

    for (pattern, category_id, priority) in rules {
        sqlx::query(
            "INSERT OR IGNORE INTO category_rules (pattern, category_id, priority) VALUES (?, ?, ?)",
        )
        .bind(pattern)
        .bind(category_id)
        .bind(priority)
        .execute(pool)
        .await
        .ok(); // Ignore errors for duplicate rules
    }

    Ok(())
}