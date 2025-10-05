# Category Rules Management Contracts
**Tauri Commands for Merchant Keyword Matching Rules**

## Overview
Category rules enable automatic transaction categorization based on merchant name patterns. Rules are applied in priority order (highest first) during CSV import and manual categorization.

## Command: `create_category_rule`
Create a new merchant matching rule.

### Request
```typescript
interface CreateCategoryRuleRequest {
  pattern: string;                 // Lowercase merchant keyword (e.g., "starbucks")
  category_id: number;             // Category to assign when matched
  priority?: number;               // Optional: Higher = checked first (default: 0)
}
```

### Response
```typescript
interface CreateCategoryRuleResponse {
  id: number;                      // Created rule ID
  pattern: string;                 // Normalized lowercase pattern
  category_id: number;
  priority: number;
  created_at: string;              // ISO 8601 timestamp
}
```

### Validation
- `pattern`: Required, 1-100 characters, converted to lowercase
- `category_id`: Required, must reference existing category
- `priority`: Integer, defaults to 0, higher values checked first

### Errors
- `ValidationError`: Invalid input (empty pattern, etc.)
- `CategoryNotFound`: category_id doesn't exist
- `DuplicateRule`: Exact pattern already exists for same category

### Contract Test
```typescript
describe('create_category_rule command', () => {
  it('should create rule with pattern normalization', async () => {
    const groceryCat = await invoke('create_category', { name: 'Groceries' });

    const response = await invoke('create_category_rule', {
      pattern: 'Whole Foods',        // Mixed case
      category_id: groceryCat.id,
      priority: 10
    });

    expect(response.pattern).toBe('whole foods'); // Normalized lowercase
    expect(response.priority).toBe(10);
  });

  it('should default priority to 0', async () => {
    const cat = await invoke('create_category', { name: 'Test' });

    const response = await invoke('create_category_rule', {
      pattern: 'testmerchant',
      category_id: cat.id
    });

    expect(response.priority).toBe(0);
  });

  it('should reject invalid category', async () => {
    await expect(
      invoke('create_category_rule', { pattern: 'test', category_id: 99999 })
    ).rejects.toThrow('CategoryNotFound');
  });
});
```

---

## Command: `list_category_rules`
Retrieve all categorization rules, ordered by priority (descending).

### Request
```typescript
interface ListCategoryRulesRequest {
  category_id?: number;            // Optional: Filter by category
}
```

### Response
```typescript
interface ListCategoryRulesResponse {
  rules: CategoryRule[];
}

interface CategoryRule {
  id: number;
  pattern: string;
  category_id: number;
  category_name: string;           // Joined from categories table
  priority: number;
  created_at: string;
}
```

### Ordering
Rules are returned in priority order (highest first), then by creation date for same priority.

### Contract Test
```typescript
describe('list_category_rules command', () => {
  it('should return rules ordered by priority', async () => {
    const cat = await invoke('create_category', { name: 'Test' });

    await invoke('create_category_rule', { pattern: 'low', category_id: cat.id, priority: 1 });
    await invoke('create_category_rule', { pattern: 'high', category_id: cat.id, priority: 10 });

    const response = await invoke('list_category_rules');

    expect(response.rules[0].pattern).toBe('high'); // Priority 10 first
    expect(response.rules[1].pattern).toBe('low');
  });

  it('should filter by category', async () => {
    const cat1 = await invoke('create_category', { name: 'Cat1' });
    const cat2 = await invoke('create_category', { name: 'Cat2' });

    await invoke('create_category_rule', { pattern: 'test1', category_id: cat1.id });
    await invoke('create_category_rule', { pattern: 'test2', category_id: cat2.id });

    const response = await invoke('list_category_rules', { category_id: cat1.id });

    expect(response.rules.every(r => r.category_id === cat1.id)).toBe(true);
    expect(response.rules.length).toBe(1);
  });

  it('should include category name in response', async () => {
    const cat = await invoke('create_category', { name: 'Groceries' });
    await invoke('create_category_rule', { pattern: 'safeway', category_id: cat.id });

    const response = await invoke('list_category_rules');

    expect(response.rules[0].category_name).toBe('Groceries');
  });
});
```

---

## Command: `update_category_rule`
Update an existing categorization rule.

### Request
```typescript
interface UpdateCategoryRuleRequest {
  id: number;                      // Rule ID to update
  pattern?: string;                // Optional: New pattern
  category_id?: number;            // Optional: New category
  priority?: number;               // Optional: New priority
}
```

### Response
```typescript
interface UpdateCategoryRuleResponse {
  id: number;
  pattern: string;
  category_id: number;
  priority: number;
  updated_at: string;              // ISO 8601 timestamp
}
```

### Validation
- `id`: Required, must exist
- At least one field must be provided for update
- `pattern`: If provided, normalized to lowercase
- `category_id`: If provided, must reference existing category

### Errors
- `RuleNotFound`: Rule ID doesn't exist
- `ValidationError`: Invalid input or no fields provided
- `CategoryNotFound`: New category_id doesn't exist

### Contract Test
```typescript
describe('update_category_rule command', () => {
  it('should update rule pattern', async () => {
    const cat = await invoke('create_category', { name: 'Test' });
    const rule = await invoke('create_category_rule', { pattern: 'old', category_id: cat.id });

    const response = await invoke('update_category_rule', {
      id: rule.id,
      pattern: 'New Pattern'
    });

    expect(response.pattern).toBe('new pattern'); // Normalized
  });

  it('should update priority only', async () => {
    const cat = await invoke('create_category', { name: 'Test' });
    const rule = await invoke('create_category_rule', { pattern: 'test', category_id: cat.id, priority: 0 });

    const response = await invoke('update_category_rule', {
      id: rule.id,
      priority: 100
    });

    expect(response.pattern).toBe('test'); // Unchanged
    expect(response.priority).toBe(100);
  });

  it('should move rule to different category', async () => {
    const cat1 = await invoke('create_category', { name: 'Cat1' });
    const cat2 = await invoke('create_category', { name: 'Cat2' });
    const rule = await invoke('create_category_rule', { pattern: 'test', category_id: cat1.id });

    const response = await invoke('update_category_rule', {
      id: rule.id,
      category_id: cat2.id
    });

    expect(response.category_id).toBe(cat2.id);
  });
});
```

---

## Command: `delete_category_rule`
Delete a categorization rule.

### Request
```typescript
interface DeleteCategoryRuleRequest {
  id: number;                      // Rule ID to delete
}
```

### Response
```typescript
interface DeleteCategoryRuleResponse {
  success: boolean;
  deleted_rule_id: number;
}
```

### Behavior
- Does NOT affect existing transactions (they keep their assigned categories)
- Only affects future auto-categorization
- Confirmation recommended for user-created rules

### Errors
- `RuleNotFound`: Rule ID doesn't exist

### Contract Test
```typescript
describe('delete_category_rule command', () => {
  it('should delete rule successfully', async () => {
    const cat = await invoke('create_category', { name: 'Test' });
    const rule = await invoke('create_category_rule', { pattern: 'delete-me', category_id: cat.id });

    const response = await invoke('delete_category_rule', { id: rule.id });

    expect(response.success).toBe(true);
    expect(response.deleted_rule_id).toBe(rule.id);

    // Verify rule no longer exists
    const rules = await invoke('list_category_rules');
    expect(rules.rules.find(r => r.id === rule.id)).toBeUndefined();
  });

  it('should not affect existing transactions', async () => {
    const cat = await invoke('create_category', { name: 'Test' });
    const rule = await invoke('create_category_rule', { pattern: 'merchant', category_id: cat.id });

    // Create transaction with this categorization
    const account = await invoke('create_account', { name: 'Test', account_type: 'checking', initial_balance: 0 });
    await invoke('import_csv', { file_path: '/path/with/merchant.csv', account_id: account.id });

    const beforeDelete = await invoke('list_transactions');
    const originalCategoryId = beforeDelete.transactions[0].category_id;

    // Delete the rule
    await invoke('delete_category_rule', { id: rule.id });

    const afterDelete = await invoke('list_transactions');
    expect(afterDelete.transactions[0].category_id).toBe(originalCategoryId); // Unchanged
  });
});
```

---

## Matching Algorithm

When categorizing a transaction:

1. Rules are evaluated in **priority order** (highest first)
2. Transaction merchant field is **normalized to lowercase**
3. Pattern matching uses **case-insensitive substring search**
4. **First matching rule** assigns the category
5. If **no rules match**, transaction assigned to "Uncategorized"

### Example
```typescript
// Rules (in priority order):
// Priority 10: "whole foods" → Groceries
// Priority 10: "starbucks" → Dining
// Priority 5:  "amazon" → Shopping
// Priority 0:  "gas" → Transportation

// Matches:
"WHOLE FOODS MARKET #123" → Groceries (substring match)
"Starbucks Coffee" → Dining (case-insensitive)
"Amazon.com" → Shopping
"Shell Gas Station" → Transportation
"Random Merchant" → Uncategorized (no match)
```

## Predefined Rules

The following rules are seeded on initial database setup:

| Pattern | Category | Priority |
|---------|----------|----------|
| walmart | Groceries | 5 |
| kroger | Groceries | 5 |
| safeway | Groceries | 5 |
| mcdonalds | Dining | 5 |
| starbucks | Dining | 5 |
| chipotle | Dining | 5 |
| uber | Transportation | 5 |
| shell | Transportation | 5 |
| chevron | Transportation | 5 |
| netflix | Entertainment | 5 |
| spotify | Entertainment | 5 |

## Notes

- Patterns are **always stored lowercase** for consistent matching
- **Priority ties** are broken by creation date (older rules first)
- Rules can be created for **both predefined and custom categories**
- **Case-insensitive substring matching** allows flexible merchant name variations
- Users can override auto-categorization by manually recategorizing transactions
