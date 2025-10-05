# Category Management Contracts
**Tauri Commands for Category Operations**

## Command: `create_category`
Create a new custom spending category.

### Request
```typescript
interface CreateCategoryRequest {
  name: string;                    // Category name (e.g., "Fast Food")
  icon?: string;                   // Optional: Emoji or icon identifier
  parent_id?: number;              // Optional: For subcategories (future feature)
}
```

### Response
```typescript
interface CreateCategoryResponse {
  id: number;                      // Created category ID
  name: string;
  type: string;                    // Always "custom" for user-created
  icon?: string;
  parent_id?: number;
  created_at: string;              // ISO 8601 timestamp
}
```

### Validation
- `name`: Required, 1-50 characters, unique
- `icon`: Optional, single emoji or max 20 characters
- `parent_id`: If provided, must reference existing category

### Errors
- `ValidationError`: Invalid input (empty name, name too long, etc.)
- `DuplicateCategory`: Category with same name already exists
- `ParentNotFound`: parent_id doesn't exist

### Contract Test
```typescript
describe('create_category command', () => {
  it('should create custom category', async () => {
    const response = await invoke('create_category', {
      name: 'My Custom Category',
      icon: '🎯'
    });

    expect(response.id).toBeGreaterThan(0);
    expect(response.name).toBe('My Custom Category');
    expect(response.type).toBe('custom');
    expect(response.icon).toBe('🎯');
  });

  it('should reject duplicate category names', async () => {
    await invoke('create_category', { name: 'Duplicate' });

    await expect(
      invoke('create_category', { name: 'Duplicate' })
    ).rejects.toThrow('DuplicateCategory');
  });
});
```

---

## Command: `list_categories`
Retrieve all categories (predefined and custom).

### Request
```typescript
interface ListCategoriesRequest {
  type?: 'predefined' | 'custom'; // Optional: Filter by type
}
```

### Response
```typescript
interface ListCategoriesResponse {
  categories: Category[];
}

interface Category {
  id: number;
  name: string;
  type: 'predefined' | 'custom';
  icon?: string;
  parent_id?: number;
  created_at: string;
}
```

### Contract Test
```typescript
describe('list_categories command', () => {
  it('should return all categories', async () => {
    const response = await invoke('list_categories');

    expect(response.categories.length).toBeGreaterThan(0);
    expect(response.categories.some(c => c.type === 'predefined')).toBe(true);
  });

  it('should filter by type', async () => {
    await invoke('create_category', { name: 'Custom 1' });

    const response = await invoke('list_categories', { type: 'custom' });

    expect(response.categories.every(c => c.type === 'custom')).toBe(true);
  });
});
```

---

## Command: `update_category`
Update a custom category's name or icon.

### Request
```typescript
interface UpdateCategoryRequest {
  id: number;                      // Category ID to update
  name?: string;                   // Optional: New name
  icon?: string;                   // Optional: New icon
}
```

### Response
```typescript
interface UpdateCategoryResponse {
  id: number;
  name: string;
  type: string;
  icon?: string;
  updated_at: string;              // ISO 8601 timestamp
}
```

### Validation
- `id`: Required, must exist, must be type "custom"
- At least one field (name or icon) must be provided
- `name`: If provided, must be unique

### Errors
- `CategoryNotFound`: Category ID doesn't exist
- `CannotModifyPredefined`: Attempting to update predefined category (per spec FR-024)
- `ValidationError`: Invalid input
- `DuplicateCategory`: New name conflicts with existing category

### Contract Test
```typescript
describe('update_category command', () => {
  it('should update custom category name', async () => {
    const category = await invoke('create_category', { name: 'Old Name' });

    const response = await invoke('update_category', {
      id: category.id,
      name: 'New Name'
    });

    expect(response.name).toBe('New Name');
  });

  it('should reject update of predefined category', async () => {
    const categories = await invoke('list_categories', { type: 'predefined' });
    const predefinedId = categories.categories[0].id;

    await expect(
      invoke('update_category', { id: predefinedId, name: 'Modified' })
    ).rejects.toThrow('CannotModifyPredefined');
  });

  it('should update icon only', async () => {
    const category = await invoke('create_category', { name: 'Test', icon: '🎯' });

    const response = await invoke('update_category', {
      id: category.id,
      icon: '🎨'
    });

    expect(response.name).toBe('Test'); // Unchanged
    expect(response.icon).toBe('🎨');
  });
});
```

---

## Command: `delete_category`
Delete a custom category and reassign transactions to "Uncategorized".

### Request
```typescript
interface DeleteCategoryRequest {
  id: number;                      // Category ID to delete
}
```

### Response
```typescript
interface DeleteCategoryResponse {
  success: boolean;
  deleted_category_id: number;
  reassigned_transactions_count: number; // Count reassigned to "Uncategorized"
}
```

### Behavior
- **Predefined Protection**: Cannot delete predefined categories (per spec FR-024)
- **Transaction Reassignment**: All transactions in this category are moved to "Uncategorized" (per spec FR-025 clarification)
- **Confirmation**: Frontend MUST show confirmation dialog before calling this command (per spec FR-050)

### Errors
- `CategoryNotFound`: Category ID doesn't exist
- `CannotDeletePredefined`: Attempting to delete predefined category

### Contract Test
```typescript
describe('delete_category command', () => {
  it('should delete custom category and reassign transactions', async () => {
    const category = await invoke('create_category', { name: 'To Delete' });

    // Assign some transactions to this category
    const transactions = await invoke('list_transactions');
    await invoke('update_transaction_category', {
      transaction_id: transactions.transactions[0].id,
      category_id: category.id
    });

    const response = await invoke('delete_category', { id: category.id });

    expect(response.success).toBe(true);
    expect(response.reassigned_transactions_count).toBeGreaterThan(0);

    // Verify transactions moved to "Uncategorized"
    const updated = await invoke('list_transactions');
    const uncategorizedId = (await invoke('list_categories'))
      .categories
      .find(c => c.name === 'Uncategorized').id;

    expect(updated.transactions[0].category_id).toBe(uncategorizedId);
  });

  it('should reject deletion of predefined categories', async () => {
    const categories = await invoke('list_categories', { type: 'predefined' });
    const predefinedId = categories.categories[0].id;

    await expect(
      invoke('delete_category', { id: predefinedId })
    ).rejects.toThrow('CannotDeletePredefined');
  });

  it('should return zero reassignment count if no transactions', async () => {
    const category = await invoke('create_category', { name: 'Empty' });

    const response = await invoke('delete_category', { id: category.id });

    expect(response.success).toBe(true);
    expect(response.reassigned_transactions_count).toBe(0);
  });
});
```

---

## Predefined Categories

The following categories are seeded on initial database setup (per spec FR-010):

1. Groceries (🛒)
2. Dining (🍽️)
3. Transportation (🚗)
4. Entertainment (🎬)
5. Utilities (⚡)
6. Healthcare (🏥)
7. Shopping (🛍️)
8. Travel (✈️)
9. Income (💰)
10. **Uncategorized** (❓) - Default for unmatched transactions

## Notes

- **Predefined categories** cannot be modified or deleted (spec FR-024)
- **Custom categories** can be updated and deleted (spec FR-022, FR-023)
- **Uncategorized** category is required and must always exist (reassignment target per spec FR-025)
- Frontend must implement confirmation dialogs per spec FR-050
- Category deletion does NOT cascade to transactions - they are reassigned (per 2025-10-05 clarification)
