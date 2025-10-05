# Transaction Management Contracts
**Tauri Commands for Transaction Operations**

## Command: `import_csv`
Import transactions from a CSV file.

### Request
```typescript
interface ImportCsvRequest {
  file_path: string;           // Absolute path to CSV file
  mapping_id?: number;         // Optional: Use saved column mapping
  account_id: number;          // Account to associate transactions with
}
```

### Response
```typescript
interface ImportCsvResponse {
  success: boolean;
  imported_count: number;
  skipped_count: number;       // Duplicates skipped
  errors: string[];            // Parse errors (row numbers)
  preview?: CsvPreview;        // If mapping_id not provided
}

interface CsvPreview {
  columns: string[];           // Column headers from CSV
  sample_rows: string[][];     // First 5 rows for preview
}
```

### Errors
- `FileNotFound`: CSV file doesn't exist
- `ParseError`: Invalid CSV format
- `MappingNotFound`: mapping_id doesn't exist in database

### Contract Test
```typescript
describe('import_csv command', () => {
  it('should return preview when no mapping provided', async () => {
    const response = await invoke('import_csv', {
      file_path: '/path/to/test.csv',
      account_id: 1
    });

    expect(response.preview).toBeDefined();
    expect(response.preview.columns).toHaveLength(5);
  });

  it('should import transactions with valid mapping', async () => {
    const response = await invoke('import_csv', {
      file_path: '/path/to/test.csv',
      mapping_id: 1,
      account_id: 1
    });

    expect(response.success).toBe(true);
    expect(response.imported_count).toBeGreaterThan(0);
  });

  it('should skip duplicate transactions', async () => {
    // Import same file twice
    await invoke('import_csv', { file_path: '/path/to/test.csv', mapping_id: 1, account_id: 1 });
    const response = await invoke('import_csv', { file_path: '/path/to/test.csv', mapping_id: 1, account_id: 1 });

    expect(response.skipped_count).toBeGreaterThan(0);
  });
});
```

---

## Command: `save_column_mapping`
Save CSV column mapping for future imports.

### Request
```typescript
interface SaveColumnMappingRequest {
  source_name: string;         // User-friendly name (e.g., "Chase Visa")
  date_col: string;            // Column name/index for date
  amount_col: string;
  description_col: string;
  merchant_col?: string;
}
```

### Response
```typescript
interface SaveColumnMappingResponse {
  mapping_id: number;          // ID of created mapping
}
```

### Errors
- `DuplicateName`: source_name already exists

### Contract Test
```typescript
it('should save column mapping', async () => {
  const response = await invoke('save_column_mapping', {
    source_name: 'Test Bank',
    date_col: 'Date',
    amount_col: 'Amount',
    description_col: 'Description'
  });

  expect(response.mapping_id).toBeGreaterThan(0);
});
```

---

## Command: `list_transactions`
Retrieve transactions with optional filtering.

### Request
```typescript
interface ListTransactionsRequest {
  account_id?: number;
  category_id?: number;
  start_date?: string;         // ISO 8601 format
  end_date?: string;
  search?: string;             // Optional: Search description/merchant (FR-016)
  limit?: number;              // Default 25 per page (spec FR-014)
  offset?: number;             // For pagination
}
```

### Response
```typescript
interface ListTransactionsResponse {
  transactions: Transaction[];
  total_count: number;
}

interface Transaction {
  id: number;
  account_id: number;
  category_id: number;
  date: string;
  amount: number;
  description: string;
  merchant: string | null;
  created_at: string;
}
```

### Errors
- `InvalidDateFormat`: start_date or end_date not ISO 8601

### Contract Test
```typescript
it('should list transactions with pagination', async () => {
  const response = await invoke('list_transactions', {
    limit: 10,
    offset: 0
  });

  expect(response.transactions).toHaveLength(10);
  expect(response.total_count).toBeGreaterThanOrEqual(10);
});

it('should filter transactions by date range', async () => {
  const response = await invoke('list_transactions', {
    start_date: '2025-01-01',
    end_date: '2025-12-31'
  });

  response.transactions.forEach(t => {
    expect(t.date).toMatch(/^2025-/);
  });
});
```

---

## Command: `update_transaction_category`
Manually recategorize a transaction.

### Request
```typescript
interface UpdateTransactionCategoryRequest {
  transaction_id: number;
  category_id: number;
}
```

### Response
```typescript
interface UpdateTransactionCategoryResponse {
  success: boolean;
}
```

### Errors
- `TransactionNotFound`: transaction_id doesn't exist
- `CategoryNotFound`: category_id doesn't exist

### Contract Test
```typescript
it('should update transaction category', async () => {
  const response = await invoke('update_transaction_category', {
    transaction_id: 1,
    category_id: 5
  });

  expect(response.success).toBe(true);

  // Verify category changed
  const transaction = await invoke('get_transaction', { id: 1 });
  expect(transaction.category_id).toBe(5);
});
```

---

## Command: `categorize_transaction`
Auto-categorize a transaction using category rules.

### Request
```typescript
interface CategorizeTransactionRequest {
  transaction_id: number;
}
```

### Response
```typescript
interface CategorizeTransactionResponse {
  category_id: number;
  matched_rule_id: number | null;  // NULL if assigned to "Uncategorized"
}
```

### Contract Test
```typescript
it('should auto-categorize using merchant keyword', async () => {
  // Assume transaction has merchant "Starbucks Coffee"
  const response = await invoke('categorize_transaction', {
    transaction_id: 1
  });

  expect(response.category_id).toBe(/* Dining category ID */);
  expect(response.matched_rule_id).toBeDefined();
});

it('should assign to Uncategorized when no rule matches', async () => {
  // Transaction with unknown merchant
  const response = await invoke('categorize_transaction', {
    transaction_id: 2
  });

  expect(response.category_id).toBe(/* Uncategorized category ID */);
  expect(response.matched_rule_id).toBeNull();
});
```

---

## Command: `create_category`
Create a custom spending category.

### Request
```typescript
interface CreateCategoryRequest {
  name: string;
  icon?: string;               // Emoji or icon name
}
```

### Response
```typescript
interface CreateCategoryResponse {
  category_id: number;
}
```

### Errors
- `DuplicateName`: Category with this name already exists

### Contract Test
```typescript
it('should create custom category', async () => {
  const response = await invoke('create_category', {
    name: 'Pet Supplies',
    icon: '🐕'
  });

  expect(response.category_id).toBeGreaterThan(0);
});
```

---

## Command: `export_transactions`
Export transactions to CSV or JSON.

### Request
```typescript
interface ExportTransactionsRequest {
  format: 'csv' | 'json';
  filters?: ListTransactionsRequest;  // Same filters as list_transactions
  output_path: string;                // Absolute path for output file
}
```

### Response
```typescript
interface ExportTransactionsResponse {
  success: boolean;
  file_path: string;
  record_count: number;
}
```

### Errors
- `WriteError`: Cannot write to output_path

### Contract Test
```typescript
it('should export transactions to CSV', async () => {
  const response = await invoke('export_transactions', {
    format: 'csv',
    output_path: '/tmp/export.csv',
    filters: { start_date: '2025-01-01' }
  });

  expect(response.success).toBe(true);
  expect(response.file_path).toBe('/tmp/export.csv');
});
```

---

## Command: `search_transactions`
Search transactions by description or merchant text with debounce.

### Request
```typescript
interface SearchTransactionsRequest {
  query: string;               // Search text (min 1 character)
  account_id?: number;         // Optional: Filter by account
  category_id?: number;        // Optional: Filter by category
  start_date?: string;         // Optional: Date range filter
  end_date?: string;
  limit?: number;              // Default 25
  offset?: number;
}
```

### Response
```typescript
interface SearchTransactionsResponse {
  transactions: Transaction[];
  total_count: number;
  query: string;               // Echo back search query
}
```

### Behavior
- **Case-insensitive** substring matching on `description` and `merchant` fields
- **Real-time filtering** with 500ms debounce (per spec FR-017) - implemented in frontend
- Returns transactions where query appears in EITHER description OR merchant
- Empty query returns all transactions (same as list_transactions)

### Errors
- `ValidationError`: Query too long (>100 characters)

### Contract Test
```typescript
describe('search_transactions command', () => {
  it('should search by description substring', async () => {
    // Assume transaction exists with description "Grocery shopping at Whole Foods"
    const response = await invoke('search_transactions', {
      query: 'grocery'
    });

    expect(response.transactions.length).toBeGreaterThan(0);
    expect(response.transactions[0].description.toLowerCase()).toContain('grocery');
  });

  it('should search by merchant substring', async () => {
    // Transaction with merchant "Starbucks Coffee"
    const response = await invoke('search_transactions', {
      query: 'starbucks'
    });

    expect(response.transactions.some(t =>
      t.merchant?.toLowerCase().includes('starbucks')
    )).toBe(true);
  });

  it('should be case-insensitive', async () => {
    const response = await invoke('search_transactions', {
      query: 'WHOLE FOODS'
    });

    expect(response.transactions.length).toBeGreaterThan(0);
  });

  it('should support pagination', async () => {
    const response = await invoke('search_transactions', {
      query: 'store',
      limit: 5,
      offset: 0
    });

    expect(response.transactions.length).toBeLessThanOrEqual(5);
    expect(response.total_count).toBeDefined();
  });
});
```

---

## Command: `delete_transaction`
Delete a single transaction.

### Request
```typescript
interface DeleteTransactionRequest {
  id: number;                  // Transaction ID to delete
}
```

### Response
```typescript
interface DeleteTransactionResponse {
  success: boolean;
  deleted_transaction_id: number;
}
```

### Behavior
- **Confirmation**: Frontend SHOULD show confirmation dialog before calling (per spec FR-050)
- Permanently removes transaction from database
- Updates account balance calculations

### Errors
- `TransactionNotFound`: Transaction ID doesn't exist

### Contract Test
```typescript
describe('delete_transaction command', () => {
  it('should delete transaction successfully', async () => {
    const transactions = await invoke('list_transactions', { limit: 1 });
    const transactionId = transactions.transactions[0].id;

    const response = await invoke('delete_transaction', { id: transactionId });

    expect(response.success).toBe(true);
    expect(response.deleted_transaction_id).toBe(transactionId);

    // Verify transaction no longer exists
    const updated = await invoke('list_transactions');
    expect(updated.transactions.find(t => t.id === transactionId)).toBeUndefined();
  });

  it('should fail if transaction not found', async () => {
    await expect(
      invoke('delete_transaction', { id: 99999 })
    ).rejects.toThrow('TransactionNotFound');
  });
});
```

---

## Command: `bulk_delete_transactions`
Delete multiple selected transactions in one operation.

### Request
```typescript
interface BulkDeleteTransactionsRequest {
  transaction_ids: number[];   // Array of transaction IDs to delete
}
```

### Response
```typescript
interface BulkDeleteTransactionsResponse {
  success: boolean;
  deleted_count: number;
  failed_ids: number[];        // IDs that couldn't be deleted
}
```

### Behavior
- **Confirmation REQUIRED**: Frontend MUST show confirmation dialog with count before calling (per spec FR-051)
- Deletes transactions in a single database transaction (all or none on error)
- Skips non-existent IDs and reports them in `failed_ids`
- Maximum 1000 IDs per request (safety limit)

### Errors
- `ValidationError`: Empty array or exceeds 1000 IDs
- `DatabaseError`: Transaction rollback on failure

### Contract Test
```typescript
describe('bulk_delete_transactions command', () => {
  it('should delete multiple transactions', async () => {
    const transactions = await invoke('list_transactions', { limit: 3 });
    const ids = transactions.transactions.map(t => t.id);

    const response = await invoke('bulk_delete_transactions', {
      transaction_ids: ids
    });

    expect(response.success).toBe(true);
    expect(response.deleted_count).toBe(3);
    expect(response.failed_ids).toEqual([]);

    // Verify all deleted
    const updated = await invoke('list_transactions');
    ids.forEach(id => {
      expect(updated.transactions.find(t => t.id === id)).toBeUndefined();
    });
  });

  it('should report failed deletions', async () => {
    const response = await invoke('bulk_delete_transactions', {
      transaction_ids: [1, 99999, 2]  // 99999 doesn't exist
    });

    expect(response.failed_ids).toContain(99999);
  });

  it('should reject requests over 1000 IDs', async () => {
    const manyIds = Array.from({ length: 1001 }, (_, i) => i);

    await expect(
      invoke('bulk_delete_transactions', { transaction_ids: manyIds })
    ).rejects.toThrow('ValidationError');
  });
});
```

---

## Command: `bulk_update_category`
Update category for multiple selected transactions.

### Request
```typescript
interface BulkUpdateCategoryRequest {
  transaction_ids: number[];   // Array of transaction IDs
  category_id: number;         // New category to assign
}
```

### Response
```typescript
interface BulkUpdateCategoryResponse {
  success: boolean;
  updated_count: number;
  failed_ids: number[];        // IDs that couldn't be updated
}
```

### Behavior
- Updates all specified transactions to new category in single operation
- Skips non-existent transaction IDs and reports in `failed_ids`
- Validates category_id exists before updating
- Maximum 1000 IDs per request (safety limit)

### Errors
- `ValidationError`: Empty array or exceeds 1000 IDs
- `CategoryNotFound`: category_id doesn't exist
- `DatabaseError`: Transaction rollback on failure

### Contract Test
```typescript
describe('bulk_update_category command', () => {
  it('should update category for multiple transactions', async () => {
    const transactions = await invoke('list_transactions', { limit: 3 });
    const ids = transactions.transactions.map(t => t.id);
    const newCategory = await invoke('create_category', { name: 'Bulk Test' });

    const response = await invoke('bulk_update_category', {
      transaction_ids: ids,
      category_id: newCategory.category_id
    });

    expect(response.success).toBe(true);
    expect(response.updated_count).toBe(3);

    // Verify all updated
    const updated = await invoke('list_transactions');
    ids.forEach(id => {
      const transaction = updated.transactions.find(t => t.id === id);
      expect(transaction.category_id).toBe(newCategory.category_id);
    });
  });

  it('should reject invalid category', async () => {
    await expect(
      invoke('bulk_update_category', { transaction_ids: [1, 2], category_id: 99999 })
    ).rejects.toThrow('CategoryNotFound');
  });

  it('should report failed updates', async () => {
    const category = await invoke('create_category', { name: 'Test' });

    const response = await invoke('bulk_update_category', {
      transaction_ids: [1, 99999, 2],
      category_id: category.category_id
    });

    expect(response.failed_ids).toContain(99999);
  });
});
```

---

## Notes

- **Pagination**: Default page size is 25 transactions (spec FR-014), frontend displays pagination controls (spec FR-015)
- **Search debounce**: 500ms delay implemented in frontend (spec FR-017) to avoid excessive backend calls
- **Confirmation dialogs**: Frontend MUST confirm all delete operations showing affected count (spec FR-050, FR-051)
- **Bulk operation limits**: Maximum 1000 IDs per bulk request to prevent performance issues
- **Transaction integrity**: All bulk operations use database transactions (atomic all-or-none)
