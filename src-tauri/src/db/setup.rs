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
    const DEFAULT_RULE_PRIORITY: i64 = 10;
    let rules = vec![
        // Groceries
        ("walmart", 1, DEFAULT_RULE_PRIORITY),
        ("target", 1, DEFAULT_RULE_PRIORITY),
        ("whole foods", 1, DEFAULT_RULE_PRIORITY),
        ("trader joe", 1, DEFAULT_RULE_PRIORITY),
        ("safeway", 1, DEFAULT_RULE_PRIORITY),
        ("kroger", 1, DEFAULT_RULE_PRIORITY),
        // Dining
        ("starbucks", 2, DEFAULT_RULE_PRIORITY),
        ("mcdonalds", 2, DEFAULT_RULE_PRIORITY),
        ("chipotle", 2, DEFAULT_RULE_PRIORITY),
        ("restaurant", 2, DEFAULT_RULE_PRIORITY),
        ("cafe", 2, DEFAULT_RULE_PRIORITY),
        // Transportation
        ("uber", 3, DEFAULT_RULE_PRIORITY),
        ("lyft", 3, DEFAULT_RULE_PRIORITY),
        ("shell", 3, DEFAULT_RULE_PRIORITY),
        ("gas station", 3, DEFAULT_RULE_PRIORITY),
        // Entertainment
        ("netflix", 4, DEFAULT_RULE_PRIORITY),
        ("spotify", 4, DEFAULT_RULE_PRIORITY),
        ("movie", 4, DEFAULT_RULE_PRIORITY),
        // Shopping
        ("amazon", 7, DEFAULT_RULE_PRIORITY),
        ("ebay", 7, DEFAULT_RULE_PRIORITY),
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