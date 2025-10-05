# Column Mapping Management Contracts
**Tauri Commands for CSV Column Mapping Operations**

## Overview
Column mappings store CSV file structure preferences, allowing users to import from the same source (e.g., "Chase Visa") without re-selecting columns each time. Mappings specify which columns contain date, amount, description, and merchant data.

## Command: `save_column_mapping`
Save or update a column mapping for a CSV source.

### Request
```typescript
interface SaveColumnMappingRequest {
  source_name: string;             // User-friendly name (e.g., "Chase Visa")
  date_col: string;                // Column name or index for date
  amount_col: string;              // Column name or index for amount
  desc_col: string;                // Column name or index for description
  merchant_col?: string;           // Optional: Column for merchant name
}
```

### Response
```typescript
interface SaveColumnMappingResponse {
  id: number;                      // Mapping ID (created or updated)
  source_name: string;
  date_col: string;
  amount_col: string;
  desc_col: string;
  merchant_col?: string;
  created_at: string;              // ISO 8601 timestamp
  updated_at?: string;             // If updated
}
```

### Validation
- `source_name`: Required, 1-100 characters
- `date_col`, `amount_col`, `desc_col`: Required, 1-50 characters
- `merchant_col`: Optional, 1-50 characters
- If mapping with same `source_name` exists, it's updated (upsert behavior)

### Errors
- `ValidationError`: Invalid input (empty required fields, etc.)

### Contract Test
```typescript
describe('save_column_mapping command', () => {
  it('should create new mapping', async () => {
    const response = await invoke('save_column_mapping', {
      source_name: 'Chase Checking',
      date_col: 'Transaction Date',
      amount_col: 'Amount',
      desc_col: 'Description',
      merchant_col: 'Merchant'
    });

    expect(response.id).toBeGreaterThan(0);
    expect(response.source_name).toBe('Chase Checking');
  });

  it('should update existing mapping by source_name', async () => {
    const first = await invoke('save_column_mapping', {
      source_name: 'Bank XYZ',
      date_col: 'Date',
      amount_col: 'Amt',
      desc_col: 'Desc'
    });

    const updated = await invoke('save_column_mapping', {
      source_name: 'Bank XYZ',  // Same name
      date_col: 'TransDate',    // Different columns
      amount_col: 'Amount',
      desc_col: 'Description'
    });

    expect(updated.id).toBe(first.id); // Same ID (updated, not created)
    expect(updated.date_col).toBe('TransDate');
  });

  it('should handle mappings without merchant column', async () => {
    const response = await invoke('save_column_mapping', {
      source_name: 'Simple Bank',
      date_col: 'Date',
      amount_col: 'Amount',
      desc_col: 'Description'
      // merchant_col omitted
    });

    expect(response.merchant_col).toBeUndefined();
  });
});
```

---

## Command: `list_column_mappings`
Retrieve all saved column mappings.

### Request
```typescript
interface ListColumnMappingsRequest {
  // No parameters - returns all mappings
}
```

### Response
```typescript
interface ListColumnMappingsResponse {
  mappings: ColumnMapping[];
}

interface ColumnMapping {
  id: number;
  source_name: string;
  date_col: string;
  amount_col: string;
  desc_col: string;
  merchant_col?: string;
  created_at: string;
  updated_at?: string;
}
```

### Ordering
Mappings are returned alphabetically by `source_name`.

### Contract Test
```typescript
describe('list_column_mappings command', () => {
  it('should return all mappings sorted by name', async () => {
    await invoke('save_column_mapping', { source_name: 'Zebra Bank', date_col: 'Date', amount_col: 'Amt', desc_col: 'Desc' });
    await invoke('save_column_mapping', { source_name: 'Alpha Bank', date_col: 'Date', amount_col: 'Amt', desc_col: 'Desc' });

    const response = await invoke('list_column_mappings');

    expect(response.mappings.length).toBeGreaterThanOrEqual(2);
    expect(response.mappings[0].source_name).toBe('Alpha Bank'); // Alphabetical
  });

  it('should return empty array if no mappings exist', async () => {
    // Assume fresh database
    const response = await invoke('list_column_mappings');

    expect(response.mappings).toEqual([]);
  });
});
```

---

## Command: `get_column_mapping`
Retrieve a specific column mapping by ID or source name.

### Request
```typescript
interface GetColumnMappingRequest {
  id?: number;                     // Optional: Mapping ID
  source_name?: string;            // Optional: Source name
  // Must provide at least one
}
```

### Response
```typescript
interface GetColumnMappingResponse {
  mapping: ColumnMapping;
}

interface ColumnMapping {
  id: number;
  source_name: string;
  date_col: string;
  amount_col: string;
  desc_col: string;
  merchant_col?: string;
  created_at: string;
  updated_at?: string;
}
```

### Validation
- At least one of `id` or `source_name` must be provided
- If both provided, `id` takes precedence

### Errors
- `MappingNotFound`: No mapping found for given ID or source name
- `ValidationError`: Neither id nor source_name provided

### Contract Test
```typescript
describe('get_column_mapping command', () => {
  it('should retrieve mapping by ID', async () => {
    const saved = await invoke('save_column_mapping', { source_name: 'Test Bank', date_col: 'Date', amount_col: 'Amt', desc_col: 'Desc' });

    const response = await invoke('get_column_mapping', { id: saved.id });

    expect(response.mapping.id).toBe(saved.id);
    expect(response.mapping.source_name).toBe('Test Bank');
  });

  it('should retrieve mapping by source_name', async () => {
    await invoke('save_column_mapping', { source_name: 'Chase Visa', date_col: 'Date', amount_col: 'Amt', desc_col: 'Desc' });

    const response = await invoke('get_column_mapping', { source_name: 'Chase Visa' });

    expect(response.mapping.source_name).toBe('Chase Visa');
  });

  it('should fail if mapping not found', async () => {
    await expect(
      invoke('get_column_mapping', { id: 99999 })
    ).rejects.toThrow('MappingNotFound');
  });

  it('should fail if no parameters provided', async () => {
    await expect(
      invoke('get_column_mapping', {})
    ).rejects.toThrow('ValidationError');
  });
});
```

---

## Command: `update_column_mapping`
Update an existing column mapping.

### Request
```typescript
interface UpdateColumnMappingRequest {
  id: number;                      // Mapping ID to update
  source_name?: string;            // Optional: New name
  date_col?: string;               // Optional: New date column
  amount_col?: string;             // Optional: New amount column
  desc_col?: string;               // Optional: New description column
  merchant_col?: string;           // Optional: New merchant column (or null to remove)
}
```

### Response
```typescript
interface UpdateColumnMappingResponse {
  id: number;
  source_name: string;
  date_col: string;
  amount_col: string;
  desc_col: string;
  merchant_col?: string;
  updated_at: string;              // ISO 8601 timestamp
}
```

### Validation
- `id`: Required, must exist
- At least one field must be provided for update
- Field validation same as `save_column_mapping`

### Errors
- `MappingNotFound`: Mapping ID doesn't exist
- `ValidationError`: Invalid input or no fields provided

### Contract Test
```typescript
describe('update_column_mapping command', () => {
  it('should update specific columns', async () => {
    const mapping = await invoke('save_column_mapping', { source_name: 'Test', date_col: 'D', amount_col: 'A', desc_col: 'Desc' });

    const response = await invoke('update_column_mapping', {
      id: mapping.id,
      date_col: 'Transaction Date',
      amount_col: 'Amount'
      // desc_col unchanged
    });

    expect(response.date_col).toBe('Transaction Date');
    expect(response.amount_col).toBe('Amount');
    expect(response.desc_col).toBe('Desc'); // Original value
  });

  it('should allow removing merchant column', async () => {
    const mapping = await invoke('save_column_mapping', { source_name: 'Test', date_col: 'D', amount_col: 'A', desc_col: 'Desc', merchant_col: 'M' });

    const response = await invoke('update_column_mapping', {
      id: mapping.id,
      merchant_col: null
    });

    expect(response.merchant_col).toBeNull();
  });
});
```

---

## Command: `delete_column_mapping`
Delete a saved column mapping.

### Request
```typescript
interface DeleteColumnMappingRequest {
  id: number;                      // Mapping ID to delete
}
```

### Response
```typescript
interface DeleteColumnMappingResponse {
  success: boolean;
  deleted_mapping_id: number;
}
```

### Behavior
- Does NOT affect existing transactions (they remain in database)
- Only removes the saved mapping for future CSV imports
- Optional confirmation in UI for user convenience

### Errors
- `MappingNotFound`: Mapping ID doesn't exist

### Contract Test
```typescript
describe('delete_column_mapping command', () => {
  it('should delete mapping successfully', async () => {
    const mapping = await invoke('save_column_mapping', { source_name: 'Delete Me', date_col: 'D', amount_col: 'A', desc_col: 'Desc' });

    const response = await invoke('delete_column_mapping', { id: mapping.id });

    expect(response.success).toBe(true);
    expect(response.deleted_mapping_id).toBe(mapping.id);

    // Verify mapping no longer exists
    await expect(
      invoke('get_column_mapping', { id: mapping.id })
    ).rejects.toThrow('MappingNotFound');
  });

  it('should not affect existing transactions', async () => {
    const mapping = await invoke('save_column_mapping', { source_name: 'Bank', date_col: 'Date', amount_col: 'Amt', desc_col: 'Desc' });
    const account = await invoke('create_account', { name: 'Test', account_type: 'checking', initial_balance: 0 });

    // Import using this mapping
    await invoke('import_csv', { file_path: '/path/test.csv', mapping_id: mapping.id, account_id: account.id });

    const beforeDelete = await invoke('list_transactions');
    const transactionCount = beforeDelete.transactions.length;

    // Delete mapping
    await invoke('delete_column_mapping', { id: mapping.id });

    const afterDelete = await invoke('list_transactions');
    expect(afterDelete.transactions.length).toBe(transactionCount); // Unchanged
  });
});
```

---

## Usage Workflow

### First-Time CSV Import
1. User selects CSV file
2. System displays column preview
3. User manually maps columns (Date, Amount, Description, Merchant)
4. User provides `source_name` (e.g., "Chase Checking")
5. System saves mapping and imports transactions

### Subsequent Imports
1. User selects CSV file
2. System detects source (by file name pattern or user selection)
3. System retrieves saved mapping by `source_name`
4. Import proceeds automatically with saved column configuration
5. User can edit/delete mappings in settings

## Notes

- **Upsert behavior**: `save_column_mapping` updates if `source_name` exists
- **Column identifiers**: Can be column names (strings) or numeric indices
- **Case sensitivity**: Column names are case-sensitive (match CSV headers exactly)
- **No cascading delete**: Deleting a mapping does NOT delete transactions
- Frontend should show mapping selection dropdown during CSV import
- Recommend showing mapping details (date_col, amount_col, etc.) in management UI
