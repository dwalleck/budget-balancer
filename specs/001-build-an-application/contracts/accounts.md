# Account Management Contracts
**Tauri Commands for Account Operations**

## Command: `create_account`
Create a new account manually.

### Request
```typescript
interface CreateAccountRequest {
  name: string;                    // Account name (e.g., "Chase Checking")
  account_type: string;            // Type: "checking", "savings", "credit_card"
  initial_balance: number;         // Starting balance
}
```

### Response
```typescript
interface CreateAccountResponse {
  id: number;                      // Created account ID
  name: string;
  account_type: string;
  balance: number;
  created_at: string;              // ISO 8601 timestamp
}
```

### Validation
- `name`: Required, 1-100 characters, unique
- `account_type`: Must be one of: "checking", "savings", "credit_card"
- `initial_balance`: Number, can be negative for credit cards

### Errors
- `ValidationError`: Invalid input (empty name, invalid type, etc.)
- `DuplicateAccount`: Account with same name already exists

### Contract Test
```typescript
describe('create_account command', () => {
  it('should create account with valid input', async () => {
    const response = await invoke('create_account', {
      name: 'Test Checking',
      account_type: 'checking',
      initial_balance: 1000.00
    });

    expect(response.id).toBeGreaterThan(0);
    expect(response.name).toBe('Test Checking');
    expect(response.balance).toBe(1000.00);
  });

  it('should reject duplicate account names', async () => {
    await invoke('create_account', { name: 'Duplicate', account_type: 'checking', initial_balance: 0 });

    await expect(
      invoke('create_account', { name: 'Duplicate', account_type: 'checking', initial_balance: 0 })
    ).rejects.toThrow('DuplicateAccount');
  });

  it('should reject invalid account type', async () => {
    await expect(
      invoke('create_account', { name: 'Test', account_type: 'invalid', initial_balance: 0 })
    ).rejects.toThrow('ValidationError');
  });
});
```

---

## Command: `list_accounts`
Retrieve all accounts.

### Request
```typescript
interface ListAccountsRequest {
  // No parameters - returns all accounts
}
```

### Response
```typescript
interface ListAccountsResponse {
  accounts: Account[];
}

interface Account {
  id: number;
  name: string;
  account_type: string;
  balance: number;                 // Current balance (sum of all transactions)
  created_at: string;
}
```

### Contract Test
```typescript
describe('list_accounts command', () => {
  it('should return all accounts', async () => {
    await invoke('create_account', { name: 'Account 1', account_type: 'checking', initial_balance: 100 });
    await invoke('create_account', { name: 'Account 2', account_type: 'savings', initial_balance: 200 });

    const response = await invoke('list_accounts');

    expect(response.accounts).toHaveLength(2);
    expect(response.accounts[0].name).toBe('Account 1');
  });
});
```

---

## Command: `update_account`
Update account details.

### Request
```typescript
interface UpdateAccountRequest {
  id: number;                      // Account ID to update
  name?: string;                   // Optional: New name
  account_type?: string;           // Optional: New type
  balance?: number;                // Optional: Manual balance adjustment
}
```

### Response
```typescript
interface UpdateAccountResponse {
  id: number;
  name: string;
  account_type: string;
  balance: number;
  updated_at: string;              // ISO 8601 timestamp
}
```

### Validation
- `id`: Required, must exist
- `name`: If provided, 1-100 characters, unique
- `account_type`: If provided, must be valid type
- At least one field must be provided for update

### Errors
- `AccountNotFound`: Account ID doesn't exist
- `ValidationError`: Invalid input
- `DuplicateAccount`: New name conflicts with existing account

### Contract Test
```typescript
describe('update_account command', () => {
  it('should update account name', async () => {
    const account = await invoke('create_account', { name: 'Old Name', account_type: 'checking', initial_balance: 0 });

    const response = await invoke('update_account', {
      id: account.id,
      name: 'New Name'
    });

    expect(response.name).toBe('New Name');
    expect(response.account_type).toBe('checking'); // Unchanged
  });

  it('should update balance', async () => {
    const account = await invoke('create_account', { name: 'Test', account_type: 'checking', initial_balance: 100 });

    const response = await invoke('update_account', {
      id: account.id,
      balance: 500
    });

    expect(response.balance).toBe(500);
  });

  it('should reject update with no fields', async () => {
    const account = await invoke('create_account', { name: 'Test', account_type: 'checking', initial_balance: 0 });

    await expect(
      invoke('update_account', { id: account.id })
    ).rejects.toThrow('ValidationError');
  });
});
```

---

## Command: `delete_account`
Delete an account and all associated transactions (cascade delete per spec FR-004).

### Request
```typescript
interface DeleteAccountRequest {
  id: number;                      // Account ID to delete
}
```

### Response
```typescript
interface DeleteAccountResponse {
  success: boolean;
  deleted_account_id: number;
  deleted_transactions_count: number; // Count of transactions cascaded
}
```

### Behavior
- **Cascade Delete**: All transactions associated with this account are automatically deleted (per spec FR-004 clarification)
- **Confirmation**: Frontend MUST show confirmation dialog with transaction count before calling this command (per spec FR-050)

### Errors
- `AccountNotFound`: Account ID doesn't exist

### Contract Test
```typescript
describe('delete_account command', () => {
  it('should delete account and cascade delete transactions', async () => {
    const account = await invoke('create_account', { name: 'To Delete', account_type: 'checking', initial_balance: 0 });

    // Create some transactions for this account
    await invoke('import_csv', { file_path: '/path/to/test.csv', account_id: account.id, mapping_id: 1 });

    const response = await invoke('delete_account', { id: account.id });

    expect(response.success).toBe(true);
    expect(response.deleted_transactions_count).toBeGreaterThan(0);

    // Verify account no longer exists
    await expect(invoke('update_account', { id: account.id, name: 'Test' }))
      .rejects.toThrow('AccountNotFound');
  });

  it('should return zero transactions count if account had none', async () => {
    const account = await invoke('create_account', { name: 'Empty', account_type: 'checking', initial_balance: 0 });

    const response = await invoke('delete_account', { id: account.id });

    expect(response.success).toBe(true);
    expect(response.deleted_transactions_count).toBe(0);
  });
});
```

---

## Notes

- Account balance is **computed** from sum of transactions, not stored directly (except initial_balance on create)
- Cascade delete behavior specified in spec.md FR-004 (2025-10-05 clarification)
- Frontend must implement confirmation dialogs per spec FR-050/FR-051
- All monetary values use JavaScript numbers (sufficient precision for currency with 2 decimal places)
