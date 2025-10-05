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

### Known Issues

Frontend testing infrastructure needs configuration work:
- **Issue**: jsdom environment not loading properly with bun test
- **Impact**: React component tests cannot run
- **Workaround**: Use manual testing or E2E tests
- **TODO**: Configure vitest/jsdom properly or use Tauri test utilities

### Test Files

- `src/test/CsvUploadDialog.test.tsx` - Documents expected behavior for CSV upload (not currently runnable)

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
