use budget_balancer_lib::commands::category_commands::create_category_impl;
use budget_balancer_lib::commands::category_rule_commands::{
    create_category_rule_impl, delete_category_rule_impl, list_category_rules_impl,
    update_category_rule_impl,
};
use budget_balancer_lib::models::category::NewCategory;
use budget_balancer_lib::models::category_rule::{
    CategoryRuleFilter, NewCategoryRule, UpdateCategoryRule,
};

// T039 [P] Contract test for create_category_rule with pattern normalization
#[tokio::test]
async fn test_create_category_rule_with_normalization() {
    let db = super::get_test_db_pool().await;

    // Create a category first
    let category = NewCategory {
        name: super::unique_name("Groceries"),
        icon: Some("🛒".to_string()),
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    // Create rule with mixed case - should be normalized to lowercase
    let rule = NewCategoryRule {
        pattern: "Whole Foods Market".to_string(), // Mixed case
        category_id,
        priority: Some(10),
    };

    let result = create_category_rule_impl(db, rule).await;
    assert!(result.is_ok(), "Failed to create category rule: {:?}", result);

    let created = result.unwrap();
    assert_eq!(
        created.pattern, "whole foods market",
        "Pattern should be normalized to lowercase"
    );
    assert_eq!(created.priority, 10);
    assert_eq!(created.category_id, category_id);
}

#[tokio::test]
async fn test_create_category_rule_default_priority() {
    let db = super::get_test_db_pool().await;

    let category = NewCategory {
        name: super::unique_name("Test Category"),
        icon: None,
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    let rule = NewCategoryRule {
        pattern: "testmerchant".to_string(),
        category_id,
        priority: None, // Should default to 0
    };

    let result = create_category_rule_impl(db, rule).await.unwrap();
    assert_eq!(result.priority, 0, "Priority should default to 0");
}

#[tokio::test]
async fn test_create_category_rule_invalid_category() {
    let db = super::get_test_db_pool().await;

    let rule = NewCategoryRule {
        pattern: "test".to_string(),
        category_id: 999999, // Non-existent category
        priority: None,
    };

    let result = create_category_rule_impl(db, rule).await;
    assert!(result.is_err(), "Should reject invalid category");
    let error_msg = result.unwrap_err().to_lowercase();
    assert!(
        error_msg.contains("category") && (error_msg.contains("not found") || error_msg.contains("exist")),
        "Error should mention category not found, got: {}",
        error_msg
    );
}

// T040 [P] Contract test for list_category_rules ordered by priority
#[tokio::test]
async fn test_list_category_rules_ordered_by_priority() {
    let db = super::get_test_db_pool().await;

    let category = NewCategory {
        name: super::unique_name("Priority Test"),
        icon: None,
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    // Create rules with different priorities
    let low_rule = NewCategoryRule {
        pattern: "low priority".to_string(),
        category_id,
        priority: Some(1),
    };
    let high_rule = NewCategoryRule {
        pattern: "high priority".to_string(),
        category_id,
        priority: Some(10),
    };
    let medium_rule = NewCategoryRule {
        pattern: "medium priority".to_string(),
        category_id,
        priority: Some(5),
    };

    create_category_rule_impl(db, low_rule).await.unwrap();
    create_category_rule_impl(db, high_rule).await.unwrap();
    create_category_rule_impl(db, medium_rule).await.unwrap();

    let result = list_category_rules_impl(db, None).await.unwrap();

    // Find our rules
    let our_rules: Vec<_> = result
        .iter()
        .filter(|r| r.category_id == category_id)
        .collect();

    assert_eq!(our_rules.len(), 3, "Should have 3 rules");
    assert_eq!(our_rules[0].pattern, "high priority", "Highest priority should be first");
    assert_eq!(our_rules[1].pattern, "medium priority", "Medium priority should be second");
    assert_eq!(our_rules[2].pattern, "low priority", "Lowest priority should be last");
}

#[tokio::test]
async fn test_list_category_rules_filter_by_category() {
    let db = super::get_test_db_pool().await;

    let cat1 = NewCategory {
        name: super::unique_name("Cat1"),
        icon: None,
    };
    let cat1_id = create_category_impl(db, cat1).await.unwrap();

    let cat2 = NewCategory {
        name: super::unique_name("Cat2"),
        icon: None,
    };
    let cat2_id = create_category_impl(db, cat2).await.unwrap();

    // Create rules for both categories
    create_category_rule_impl(
        db,
        NewCategoryRule {
            pattern: "test1".to_string(),
            category_id: cat1_id,
            priority: None,
        },
    )
    .await
    .unwrap();

    create_category_rule_impl(
        db,
        NewCategoryRule {
            pattern: "test2".to_string(),
            category_id: cat2_id,
            priority: None,
        },
    )
    .await
    .unwrap();

    // Filter by category 1
    let result = list_category_rules_impl(db, Some(CategoryRuleFilter::ByCategoryId(cat1_id)))
        .await
        .unwrap();

    let filtered: Vec<_> = result.iter().filter(|r| r.category_id == cat1_id).collect();
    assert!(!filtered.is_empty(), "Should have at least one rule for category 1");
    assert!(
        result.iter().all(|r| r.category_id == cat1_id),
        "All rules should belong to category 1"
    );
}

#[tokio::test]
async fn test_list_category_rules_includes_category_name() {
    let db = super::get_test_db_pool().await;

    let category_name = super::unique_name("Groceries");
    let category = NewCategory {
        name: category_name.clone(),
        icon: Some("🛒".to_string()),
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    create_category_rule_impl(
        db,
        NewCategoryRule {
            pattern: "safeway".to_string(),
            category_id,
            priority: None,
        },
    )
    .await
    .unwrap();

    let result = list_category_rules_impl(db, None).await.unwrap();

    let our_rule = result.iter().find(|r| r.category_id == category_id).unwrap();
    assert_eq!(
        our_rule.category_name, category_name,
        "Should include category name"
    );
}

// T041 [P] Contract test for update_category_rule
#[tokio::test]
async fn test_update_category_rule_pattern() {
    let db = super::get_test_db_pool().await;

    let category = NewCategory {
        name: super::unique_name("Test"),
        icon: None,
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    let rule = NewCategoryRule {
        pattern: "old pattern".to_string(),
        category_id,
        priority: None,
    };
    let rule_id = create_category_rule_impl(db, rule).await.unwrap().id;

    // Update with mixed case - should normalize
    let update = UpdateCategoryRule {
        id: rule_id,
        pattern: Some("New Pattern".to_string()),
        category_id: None,
        priority: None,
    };

    let result = update_category_rule_impl(db, update).await;
    assert!(result.is_ok(), "Failed to update rule: {:?}", result);

    let updated = result.unwrap();
    assert_eq!(
        updated.pattern, "new pattern",
        "Pattern should be normalized to lowercase"
    );
}

#[tokio::test]
async fn test_update_category_rule_priority_only() {
    let db = super::get_test_db_pool().await;

    let category = NewCategory {
        name: super::unique_name("Test"),
        icon: None,
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    let rule = NewCategoryRule {
        pattern: "test".to_string(),
        category_id,
        priority: Some(0),
    };
    let rule_id = create_category_rule_impl(db, rule).await.unwrap().id;

    let update = UpdateCategoryRule {
        id: rule_id,
        pattern: None,
        category_id: None,
        priority: Some(100),
    };

    let updated = update_category_rule_impl(db, update).await.unwrap();
    assert_eq!(updated.pattern, "test", "Pattern should remain unchanged");
    assert_eq!(updated.priority, 100, "Priority should be updated");
}

#[tokio::test]
async fn test_update_category_rule_move_to_different_category() {
    let db = super::get_test_db_pool().await;

    let cat1 = NewCategory {
        name: super::unique_name("Cat1"),
        icon: None,
    };
    let cat1_id = create_category_impl(db, cat1).await.unwrap();

    let cat2 = NewCategory {
        name: super::unique_name("Cat2"),
        icon: None,
    };
    let cat2_id = create_category_impl(db, cat2).await.unwrap();

    let rule = NewCategoryRule {
        pattern: "test".to_string(),
        category_id: cat1_id,
        priority: None,
    };
    let rule_id = create_category_rule_impl(db, rule).await.unwrap().id;

    let update = UpdateCategoryRule {
        id: rule_id,
        pattern: None,
        category_id: Some(cat2_id),
        priority: None,
    };

    let updated = update_category_rule_impl(db, update).await.unwrap();
    assert_eq!(updated.category_id, cat2_id, "Should move to new category");
}

#[tokio::test]
async fn test_update_category_rule_not_found() {
    let db = super::get_test_db_pool().await;

    let update = UpdateCategoryRule {
        id: 999999,
        pattern: Some("test".to_string()),
        category_id: None,
        priority: None,
    };

    let result = update_category_rule_impl(db, update).await;
    assert!(result.is_err(), "Should fail for non-existent rule");
    let error_msg = result.unwrap_err().to_lowercase();
    assert!(
        error_msg.contains("not found") || error_msg.contains("rule"),
        "Error should mention rule not found"
    );
}

// T042 [P] Contract test for delete_category_rule
#[tokio::test]
async fn test_delete_category_rule_success() {
    let db = super::get_test_db_pool().await;

    let category = NewCategory {
        name: super::unique_name("Test"),
        icon: None,
    };
    let category_id = create_category_impl(db, category).await.unwrap();

    let rule = NewCategoryRule {
        pattern: "delete-me".to_string(),
        category_id,
        priority: None,
    };
    let rule_id = create_category_rule_impl(db, rule).await.unwrap().id;

    let result = delete_category_rule_impl(db, rule_id).await;
    assert!(result.is_ok(), "Failed to delete rule: {:?}", result);

    let delete_response = result.unwrap();
    assert!(delete_response.success, "Delete should succeed");
    assert_eq!(delete_response.deleted_rule_id, rule_id);

    // Verify rule no longer exists
    let all_rules = list_category_rules_impl(db, None).await.unwrap();
    assert!(
        !all_rules.iter().any(|r| r.id == rule_id),
        "Deleted rule should not be in list"
    );
}

#[tokio::test]
async fn test_delete_category_rule_not_found() {
    let db = super::get_test_db_pool().await;

    let result = delete_category_rule_impl(db, 999999).await;
    assert!(result.is_err(), "Should fail for non-existent rule");
    let error_msg = result.unwrap_err().to_lowercase();
    assert!(
        error_msg.contains("not found"),
        "Error should mention rule not found"
    );
}
