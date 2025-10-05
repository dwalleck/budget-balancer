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
  limit?: number;              // Default 100
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
