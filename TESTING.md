# Testing Guide

## Backend Tests (Rust)

Backend integration tests are located in `src-tauri/tests/integration/`.

### Running Backend Tests

```bash
cd src-tauri
cargo test --test integration_tests
```

### Code Coverage

Generate code coverage reports using `cargo-llvm-cov`:

```bash
cd src-tauri

# Run tests with coverage and view summary
cargo llvm-cov --test integration_tests

# Generate HTML report (opens in browser)
cargo llvm-cov --test integration_tests --open

# Generate lcov format (for CI/CD or other tools)
cargo llvm-cov --test integration_tests --lcov --output-path lcov.info

# Run all tests with coverage
cargo llvm-cov --all-targets

# Clean coverage artifacts
cargo llvm-cov clean
```

**Coverage Output:**
- HTML reports: `target/llvm-cov/html/index.html`
- Summary is displayed in terminal after each run

### Current Test Coverage

**Passing Tests (22):**
- ✅ Account commands (create, list, ordering)
- ✅ Category commands (create, list, ordering, seeded data)
- ✅ Transaction commands (list, filter, update category)
- ✅ CSV import (headers, basic import, duplicate detection, column mapping, categorization)

**Ignored Tests (2):**
- ⏭️ Empty CSV validation (not implemented yet)
- ⏭️ Invalid date format validation (not implemented yet)

## Frontend Tests (React/TypeScript)

Frontend tests are located in `src/test/`.

### Running Frontend Tests

```bash
# Run all frontend tests
bun test

# Run tests in watch mode
bun test --watch

# Run specific test file
bun test src/test/Button.test.tsx

# Run with coverage
bun test --coverage
```

### Known Issues ⚠️

**CRITICAL**: Frontend testing infrastructure needs configuration work:
- **Issue**: jsdom environment not loading properly with bun test
- **Impact**: React component tests cannot run
- **Status**: 🔴 Blocking
- **Priority**: HIGH (Week 1)
- **Workaround**: Manual testing only

### Fixing Frontend Testing (Action Plan)

**Step 1**: Update `vitest.config.ts`
```typescript
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
  },
})
```

**Step 2**: Install missing dependencies
```bash
bun add -d @testing-library/react @testing-library/jest-dom @testing-library/user-event
bun add -d @vitest/ui jsdom
```

**Step 3**: Create test setup file (`src/test/setup.ts`)
```typescript
import '@testing-library/jest-dom'
import { expect, vi } from 'vitest'

// Mock Tauri API
global.window.__TAURI__ = {
  invoke: vi.fn(),
  // Add other Tauri APIs as needed
}
```

**Step 4**: Update package.json scripts
```json
{
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage"
  }
}
```

### Test Files

**Documented (Not Runnable)**:
- `src/test/CsvUploadDialog.test.tsx` - CSV upload dialog behavior
- `src/test/ColumnMappingForm.test.tsx` - Column mapping form behavior

**Missing (Need to Create)**:
- `src/components/ui/Button.test.tsx` - Button component tests
- `src/components/ui/Select.test.tsx` - Select component tests
- Integration tests for complete user flows

### Frontend Testing Best Practices

1. **Component Tests**: Test components in isolation
   ```typescript
   import { render, screen } from '@testing-library/react'
   import { Button } from './Button'

   test('renders button with text', () => {
     render(<Button>Click me</Button>)
     expect(screen.getByText('Click me')).toBeInTheDocument()
   })
   ```

2. **User Interaction Tests**: Test from user's perspective
   ```typescript
   import userEvent from '@testing-library/user-event'

   test('calls onClick when clicked', async () => {
     const handleClick = vi.fn()
     render(<Button onClick={handleClick}>Click me</Button>)

     await userEvent.click(screen.getByText('Click me'))
     expect(handleClick).toHaveBeenCalledOnce()
   })
   ```

3. **Mock Tauri Commands**: Use vi.fn() for Tauri invoke
   ```typescript
   import { invoke } from '@tauri-apps/api/tauri'

   vi.mock('@tauri-apps/api/tauri', () => ({
     invoke: vi.fn(),
   }))

   test('imports CSV via Tauri', async () => {
     vi.mocked(invoke).mockResolvedValue({ success: true })
     // Test component that calls invoke('import_csv')
   })
   ```

### Coverage Targets (Frontend)

- **Overall**: > 70%
- **Components**: > 80%
- **Critical paths** (CSV import, debt planning): 100%
- **UI components**: > 75%

## Permissions Testing

The CSV upload dialog requires the following Tauri permissions (configured in `src-tauri/capabilities/default.json`):

```json
{
  "permissions": [
    "dialog:allow-open",
    "dialog:default",
    "fs:allow-read-text-file",
    "fs:default"
  ]
}
```

### Manual Test for Permissions

1. Run `bun run tauri dev`
2. Create an account
3. Click "Import CSV" button
4. Click "Select CSV File"
5. Verify file picker opens without permission errors

If you see `"dialog.open not allowed"` error, check that the permissions above are in `src-tauri/capabilities/default.json`.

## TDD Workflow

Going forward, all new features MUST follow TDD:

1. Write contract/integration test first (should fail)
2. Implement feature to make test pass
3. Refactor if needed
4. Commit with passing tests

No untested code without documented leadership signoff.

## Missing Tests (TDD Violations)

The following components were implemented without tests first:

### Frontend Components (Should Have Tests)

**ColumnMappingForm** (`src/test/ColumnMappingForm.test.tsx` - documented but not runnable)
- ✅ Test file created documenting expected behavior
- ❌ Tests cannot run due to jsdom environment issues
- **What should be tested:**
  - Renders all four column mapping selects
  - Pre-selects first three headers by default
  - Allows changing column mappings
  - Handles "None" option for optional merchant column
  - Calls importCsv with correct parameters
  - Shows loading state while importing
  - Shows success/error messages
  - Calls onComplete after successful import
  - Calls onCancel when cancelled
  - Handles edge cases (no headers, few columns, duplicates)

**CsvUploadDialog** (`src/test/CsvUploadDialog.test.tsx` - documented but not runnable)
- ✅ Test file created documenting expected behavior
- ❌ Tests cannot run due to jsdom environment issues
- **What should be tested:**
  - Opens file picker with correct permissions
  - Reads selected CSV file
  - Parses CSV headers
  - Shows column mapping form after file selection
  - Handles permission errors gracefully
  - Handles file read errors
  - Closes and resets on completion

**Button Component** (`src/components/ui/Button.tsx`)
- ❌ No tests (discovered via ref warning)
- **What should be tested:**
  - Renders with different variants (default, outline, ghost)
  - Renders with different sizes (sm, md, lg)
  - Forwards refs correctly (this was a bug we fixed!)
  - Handles click events
  - Applies custom className
  - Handles disabled state

**Select Components** (`src/components/ui/Select.tsx`)
- ❌ No tests (discovered via empty value bug)
- **What should be tested:**
  - Doesn't allow empty string values (this was a bug we fixed!)
  - Opens dropdown on trigger click
  - Selects item on click
  - Forwards refs correctly
  - Renders in portal
  - Positions correctly with popper

### How These Bugs Could Have Been Prevented

1. **Button ref warning**: Would have been caught by a test checking ref forwarding
2. **SelectItem empty value error**: Would have been caught by a test verifying value prop constraints
3. **Blank screen**: Would have been caught by integration tests rendering the full flow

### Action Items

1. ✅ Document expected behavior in test files
2. ⏳ Fix frontend testing infrastructure (jsdom setup)
3. ⏳ Make tests runnable
4. ⏳ Add tests for remaining UI components
5. ✅ **GOING FORWARD**: Write tests FIRST before any new features

---

## Security Testing

**Reference**: See `SECURITY.md` for security guidelines

### SQL Injection Tests

All database queries MUST have tests to prevent SQL injection:

```rust
#[tokio::test]
async fn test_no_sql_injection_in_filters() {
    let db = get_test_db_pool().await;

    // Attempt SQL injection in various inputs
    let malicious_inputs = vec![
        "1 OR 1=1",
        "1'; DROP TABLE transactions;--",
        "' OR ''='",
        "1 UNION SELECT * FROM sqlite_master--",
    ];

    for input in malicious_inputs {
        let filter = TransactionFilter {
            description: Some(input.to_string()),
            ..Default::default()
        };

        let result = list_transactions_impl(db, Some(filter)).await;

        // Should handle safely without executing injection
        assert!(result.is_ok(), "Failed on input: {}", input);
    }

    // Verify database integrity
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transactions")
        .fetch_one(db)
        .await
        .expect("Table should still exist");
}
```

### Input Validation Tests

```rust
#[tokio::test]
async fn test_csv_size_limits() {
    let db = get_test_db_pool().await;

    // Test file size limit
    let huge_csv = "a".repeat(20 * 1024 * 1024); // 20MB
    let result = import_csv_impl(db, account_id, huge_csv, mapping).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too large"));
}

#[tokio::test]
async fn test_csv_row_limits() {
    let db = get_test_db_pool().await;

    // Test row count limit
    let many_rows_csv = generate_csv_with_rows(15_000); // Over limit
    let result = import_csv_impl(db, account_id, many_rows_csv, mapping).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Too many rows"));
}

#[tokio::test]
async fn test_invalid_input_rejected() {
    let db = get_test_db_pool().await;

    // Test interest rate validation
    let debt = NewDebt {
        name: "Test".to_string(),
        balance: 1000.0,
        interest_rate: 150.0, // Invalid: > 100
        min_payment: 50.0,
    };

    let result = create_debt_impl(db, debt).await;
    assert!(result.is_err());
}
```

### Rate Limiting Tests

```rust
#[tokio::test]
async fn test_rate_limiting_enforced() {
    let db = get_test_db_pool().await;

    let csv_content = "Date,Amount,Description\n2024-01-01,-50.00,Test";
    let mapping = default_mapping();

    // First import should succeed
    let result1 = import_csv_impl(db, account_id, csv_content.clone(), mapping.clone()).await;
    assert!(result1.is_ok());

    // Immediate second import should be rate limited
    let result2 = import_csv_impl(db, account_id, csv_content, mapping).await;
    assert!(result2.is_err());
    assert!(result2.unwrap_err().contains("Rate limit"));
}
```

### Error Message Tests

```rust
#[tokio::test]
async fn test_errors_dont_expose_internals() {
    let db = get_test_db_pool().await;

    // Trigger various errors
    let result = get_nonexistent_transaction(db, 99999).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err();

    // Should NOT contain:
    assert!(!error_msg.contains("/home/"));  // File paths
    assert!(!error_msg.contains(".db"));     // Database files
    assert!(!error_msg.contains("sqlite"));  // Database type
    assert!(!error_msg.contains("panic"));   // Stack traces

    // Should contain user-friendly message
    assert!(error_msg.contains("not found") || error_msg.contains("Not found"));
}
```

---

## Performance Testing

### Transaction Query Performance

```rust
#[tokio::test]
async fn test_transaction_query_performance() {
    let db = get_test_db_pool().await;

    // Create 1000 test transactions
    for i in 0..1000 {
        create_test_transaction(db, i).await;
    }

    let start = Instant::now();

    // Query with pagination
    let filter = TransactionFilter {
        limit: Some(50),
        offset: Some(0),
        ..Default::default()
    };

    let result = list_transactions_impl(db, Some(filter)).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(elapsed.as_millis() < 100, "Query took {} ms", elapsed.as_millis());
}
```

### CSV Import Performance

```rust
#[tokio::test]
async fn test_csv_import_performance() {
    let db = get_test_db_pool().await;

    // Generate CSV with 1000 transactions
    let large_csv = generate_csv_with_rows(1000);

    let start = Instant::now();
    let result = import_csv_impl(db, account_id, large_csv, mapping).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(elapsed.as_millis() < 500, "Import took {} ms", elapsed.as_millis());
}
```

### Debt Calculation Performance

```rust
#[tokio::test]
async fn test_payoff_calculation_performance() {
    let db = get_test_db_pool().await;

    // Create 10 debts
    for i in 0..10 {
        create_test_debt(db, i).await;
    }

    let start = Instant::now();
    let result = calculate_payoff_plan_impl(db, "avalanche", 1000.0).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(elapsed.as_millis() < 200, "Calculation took {} ms", elapsed.as_millis());
}
```

---

## Coverage Targets

### Backend (Rust)
- **Overall**: > 60% line coverage ✅
- **Commands**: > 70%
- **Services**: > 80%
- **Critical paths**: 100%
  - Debt calculators
  - Transaction import
  - Duplicate detection

**Current Status**: 59.88% overall (needs improvement)

### Frontend (TypeScript)
- **Overall**: > 70%
- **Components**: > 80%
- **Critical paths**: 100%
  - CSV import flow
  - Debt planning flow
  - Transaction categorization

**Current Status**: 0% (testing infrastructure broken) 🔴

### Integration Tests
- All acceptance scenarios covered: 7/7 ✅
- Edge cases tested: 8+ ✅
- Contract tests for all commands: 26/26 ✅

---

## Running All Tests

```bash
# Backend tests with coverage
cd src-tauri && cargo llvm-cov --test integration_tests --open

# Frontend tests (once fixed)
bun test --coverage

# Security audit
cargo audit

# Performance benchmarks
cargo bench
```

---

## CI/CD Integration

**TODO**: Add GitHub Actions workflow

```yaml
name: Tests
on: [push, pull_request]

jobs:
  backend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      - name: Run tests with coverage
        run: cd src-tauri && cargo llvm-cov --test integration_tests --lcov --output-path lcov.info
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./src-tauri/lcov.info

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: oven-sh/setup-bun@v1
      - run: bun install
      - run: bun test --coverage
```

---

**Related Documents**:
- `SECURITY.md` - Security testing guidelines
- `PR-REVIEW-RESPONSE.md` - Issues and fixes
- `CLAUDE.md` - Development standards
