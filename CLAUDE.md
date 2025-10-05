# budget-balancer Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-04

## Active Technologies
- TypeScript 5.x / React 18 + Tauri 2, React 18, Radix UI, Tailwind CSS, Vite, Zustand, Vitest (001-build-an-application)

## Project Structure
```
src/
├── components/          # React UI components
│   ├── ui/             # Radix UI wrapper components
│   ├── transactions/   # Transaction-related components
│   ├── debts/          # Debt management components
│   └── visualizations/ # Chart/graph components
├── pages/              # Main application pages/views
├── stores/             # Zustand state stores
├── lib/                # Utility functions and helpers
└── types/              # TypeScript type definitions

src-tauri/
├── src/
│   ├── commands/        # Tauri command handlers (backend logic)
│   ├── models/          # Data models and domain logic
│   ├── services/        # Business logic services
│   └── db/              # SQLite database layer
└── tests/
    ├── integration/     # Integration tests for Tauri commands
    └── unit/            # Unit tests for services/models

tests/
├── integration/        # Frontend integration tests
└── unit/               # Frontend unit tests
```

## Commands

### Development
```bash
# Run development server
bun run tauri dev

# Build for production
bun run tauri build

# Run linter
bun run lint
```

### Testing
```bash
# Backend tests with coverage
cd src-tauri && cargo llvm-cov --test integration_tests --open

# Frontend tests (once fixed)
bun test

# Security audit
cargo audit
```

## Code Style

### TypeScript / React
- Follow standard TypeScript conventions
- Use functional components with hooks
- Prefer composition over inheritance
- Keep components small and focused

### Rust
- Follow Rust standard naming conventions
- Use `_impl` suffix for business logic functions
- Tauri commands delegate to `_impl` functions
- Proper error handling (no unwrap in production code)

## Recent Changes
- 001-build-an-application: Added TypeScript 5.x / React 18 + Tauri 2, React 18, Radix UI, Tailwind CSS, Vite, Zustand, Vitest
- 2025-10-04: Migrated all commands to use DbPool for connection pooling
- 2025-10-04: Added code coverage reporting with cargo-llvm-cov
- 2025-10-04: Updated testing documentation with security and performance sections

## Security Guidelines 🔒

**See `SECURITY.md` for complete guidelines**

### Critical Rules

1. **SQL Injection Prevention**
   - ✅ Always use parameterized queries
   - ❌ Never concatenate user input into SQL
   - Use SQLx's `.bind()` for all parameters

2. **Input Validation**
   - Validate file sizes (max 10MB for CSV)
   - Validate row counts (max 10,000 rows)
   - Validate numeric ranges before database
   - Sanitize text input

3. **Error Messages**
   - ❌ Never expose file paths or database details
   - ✅ Log detailed errors internally
   - ✅ Return generic user-friendly messages

4. **Rate Limiting**
   - Implement throttling on expensive operations
   - Min 2 seconds between CSV imports

### What NOT To Do ❌

```rust
// ❌ DON'T: Build SQL by string concatenation
let query = format!("SELECT * FROM transactions WHERE id = {}", user_input);

// ❌ DON'T: Expose errors to users
.map_err(|e| e.to_string())

// ❌ DON'T: Skip validation
db.execute(&user_provided_sql)
```

### What TO Do ✅

```rust
// ✅ DO: Use parameterized queries
sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
    .bind(transaction_id)
    .fetch_all(db)
    .await?;

// ✅ DO: Sanitize errors
.map_err(|e| {
    eprintln!("Database error: {}", e);  // Log internally
    "Failed to load transactions".to_string()  // Return safe message
})

// ✅ DO: Validate input
if amount <= 0.0 {
    return Err("Amount must be positive".to_string());
}
```

## Code Quality Standards 📐

### Rust Backend

1. **No Magic Numbers**
   - Extract all constants to `src-tauri/src/constants.rs`
   - Example: `const DEFAULT_CATEGORY_ID: i64 = 10;`

2. **Consistent Error Handling**
   - Use custom error types (consider `thiserror` crate)
   - Return `Result<T, String>` from all commands
   - Map errors consistently

3. **Function Organization**
   - Business logic in `*_impl` functions taking `&SqlitePool`
   - Tauri commands extract pool and delegate to `_impl`
   - Keeps logic testable without Tauri runtime

4. **Testing Requirements**
   - Write tests FIRST (TDD)
   - All commands must have contract tests
   - Critical paths require 100% coverage
   - Security tests for all input handling

### TypeScript Frontend

1. **Component Structure**
   - Small, focused components
   - Separate logic from presentation
   - Use hooks for state and side effects

2. **Testing Requirements**
   - Test components in isolation
   - Mock Tauri API calls
   - Test user interactions
   - Aim for >80% coverage

3. **Type Safety**
   - Define interfaces for all data structures
   - No `any` types (use `unknown` if needed)
   - Strict TypeScript config

## Performance Considerations ⚡

1. **Database Queries**
   - Use connection pooling (DbPool) ✅
   - Implement pagination (default 50 items)
   - Add indexes for common queries
   - Avoid N+1 queries

2. **CSV Processing**
   - Stream large files
   - Show progress for imports >1000 rows
   - Use batch inserts

3. **UI Rendering**
   - Virtualize long lists
   - Lazy load charts
   - Debounce expensive operations

## Recent Learnings from PR Review 📚

**PR #1 Review Highlights**:

**Strengths** ✅:
- Strong backend architecture
- Comprehensive backend testing (60% coverage)
- Good domain logic implementation

**Areas for Improvement** 🔴:
1. Frontend testing infrastructure broken (HIGH)
2. SQL injection risks in dynamic queries (HIGH)
3. No rate limiting on CSV imports (HIGH)
4. Missing pagination on transactions (MEDIUM)
5. Error messages expose internals (MEDIUM)

**Action Items**:
- See `PR-REVIEW-RESPONSE.md` for detailed plan
- Week 1: Fix frontend tests + security issues
- Week 2: Performance + error handling
- Week 3: Code quality improvements

## Related Documentation

- `TESTING.md` - Comprehensive testing guide
- `SECURITY.md` - Security best practices
- `PR-REVIEW-RESPONSE.md` - PR feedback and responses
- `specs/001-build-an-application/` - Full feature spec and plan

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->