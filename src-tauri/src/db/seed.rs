// Predefined categories and category rules seeding

pub const SEED_CATEGORIES: &str = r#"
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
    (10, 'Uncategorized', 'predefined', '❓');
"#;

pub const SEED_CATEGORY_RULES: &str = r#"
INSERT OR IGNORE INTO category_rules (pattern, category_id, priority) VALUES
    -- Groceries
    ('walmart', 1, 10),
    ('target', 1, 10),
    ('whole foods', 1, 10),
    ('trader joe', 1, 10),
    ('safeway', 1, 10),
    ('kroger', 1, 10),
    ('aldi', 1, 10),
    ('costco', 1, 10),

    -- Dining
    ('starbucks', 2, 10),
    ('mcdonalds', 2, 10),
    ('chipotle', 2, 10),
    ('subway', 2, 10),
    ('dunkin', 2, 10),
    ('pizza', 2, 10),
    ('restaurant', 2, 10),
    ('cafe', 2, 10),
    ('coffee', 2, 10),

    -- Transportation
    ('uber', 3, 10),
    ('lyft', 3, 10),
    ('shell', 3, 10),
    ('chevron', 3, 10),
    ('exxon', 3, 10),
    ('gas station', 3, 10),
    ('parking', 3, 10),
    ('metro', 3, 10),
    ('transit', 3, 10),

    -- Entertainment
    ('netflix', 4, 10),
    ('spotify', 4, 10),
    ('amazon prime', 4, 10),
    ('hulu', 4, 10),
    ('disney', 4, 10),
    ('movie', 4, 10),
    ('theater', 4, 10),
    ('cinema', 4, 10),

    -- Utilities
    ('electric', 5, 10),
    ('water', 5, 10),
    ('gas company', 5, 10),
    ('internet', 5, 10),
    ('phone', 5, 10),
    ('verizon', 5, 10),
    ('at&t', 5, 10),
    ('t-mobile', 5, 10),

    -- Healthcare
    ('pharmacy', 6, 10),
    ('cvs', 6, 10),
    ('walgreens', 6, 10),
    ('doctor', 6, 10),
    ('hospital', 6, 10),
    ('clinic', 6, 10),
    ('medical', 6, 10),

    -- Shopping
    ('amazon', 7, 10),
    ('ebay', 7, 10),
    ('best buy', 7, 10),
    ('home depot', 7, 10),
    ('lowes', 7, 10),
    ('macy', 7, 10),

    -- Travel
    ('airline', 8, 10),
    ('hotel', 8, 10),
    ('airbnb', 8, 10),
    ('booking', 8, 10),
    ('expedia', 8, 10);
"#;

// Seed data is now included in the migration file, so this function is not needed
// but kept for reference
#[allow(dead_code)]
pub async fn seed_database() -> Result<(), String> {
    // Seeds are automatically run via the migration in 001_initial_schema.sql
    Ok(())
}
