# Research & Architectural Decisions
**Feature**: Budget Balancer - Debt Management & Spending Insights
**Date**: 2025-10-04

## 1. Tauri 2 Architecture & Best Practices

### Decision: Command-based Architecture with Event System
- **Rationale**: Tauri 2's command system provides type-safe communication between Rust backend and React frontend. Commands handle synchronous operations (CRUD), while events handle async updates (progress notifications).
- **Pattern**:
  - Backend: Tauri commands in `src-tauri/src/commands/` grouped by domain (transactions, debts, analytics)
  - Frontend: Invoke commands via `@tauri-apps/api/tauri`'s `invoke()` function
  - State sync: Zustand stores react to command responses
- **Alternatives Considered**:
  - REST API within Tauri: Rejected - unnecessary overhead for local desktop app
  - Direct Rust-WASM: Rejected - Tauri provides better OS integration and simpler development

### Decision: Tauri Plugin Ecosystem
- **SQL Plugin**: Use `tauri-plugin-sql` for SQLite integration
- **FS Plugin**: Use `tauri-plugin-fs` for CSV file operations
- **Dialog Plugin**: Use `tauri-plugin-dialog` for file picker UI
- **Rationale**: Official plugins provide secure, well-tested OS integration with proper permissions

## 2. Opcode Design Reference Analysis

### Decision: Adopt Opcode's Sidebar + Content Area Layout
- **Rationale**: Opcode uses proven pattern for desktop apps - persistent sidebar navigation with main content area. Fits Budget Balancer's multi-section structure (Transactions, Debts, Analytics).
- **Specific Adaptations**:
  - Sidebar sections: Dashboard, Transactions, Debts, Spending Analysis, Settings
  - Content area: Dynamic based on selected section
  - Use Radix UI Navigation Menu for sidebar structure

### Decision: shadcn/ui Component Pattern (adapted for Radix UI)
- **Rationale**: Opcode uses shadcn/ui (built on Radix UI). We'll use Radix UI directly with custom Tailwind styling matching Opcode aesthetic.
- **Component Library Approach**:
  - Create `src/components/ui/` with Radix UI wrappers
  - Base components: Button, Card, Dialog, Select, Table, Separator
  - Tailwind config matches Opcode's clean, minimal style
- **Color Scheme**: Neutral palette with subtle accents, dark mode support from Opcode reference

### Decision: Functional, Clean Design Philosophy
- **Rationale**: Opcode prioritizes substance over flash - aligns with constitutional principle. Focus on data density, clear typography, efficient workflows.
- **Visual Hierarchy**:
  - Data-first layouts (tables, charts prominent)
  - Minimal decorative elements
  - Clear CTAs for primary actions (Import CSV, Create Debt Plan)

## 3. SQLite Schema Design

### Decision: Normalized Schema with Performance Indexes
- **Rationale**: Balance between normalization (data integrity) and denormalization (query performance). Transactions table is append-only, optimize for read-heavy analytics queries.

**Core Tables**:
1. `transactions`: id, date, amount, description, merchant, category_id, account_id, created_at
2. `categories`: id, name, type (predefined/custom), parent_id (for subcategories)
3. `category_rules`: id, pattern (merchant keyword), category_id (for auto-categorization)
4. `accounts`: id, name, type (bank/credit_card), created_at
5. `debts`: id, name, balance, original_balance, interest_rate, min_payment, created_at
6. `debt_payments`: id, debt_id, amount, date, plan_id
7. `debt_plans`: id, strategy (avalanche/snowball), monthly_amount, created_at
8. `spending_targets`: id, category_id, amount, period (monthly/quarterly/yearly), start_date

**Indexes**:
- transactions: (date DESC), (category_id, date), (account_id, date)
- category_rules: (pattern) for fast matching
- debt_payments: (debt_id, date)

**Alternatives Considered**:
- PostgreSQL: Rejected - overkill for local desktop app, adds deployment complexity
- JSON files: Rejected - poor query performance, no ACID guarantees

## 4. Chart Library Selection

### Decision: Recharts for React Visualizations
- **Rationale**:
  - Native React components (composable, declarative)
  - Works seamlessly with Tailwind CSS theming
  - Supports all required chart types: pie, bar, line, area
  - Responsive and performant for expected data volumes (<10k points)
  - Active maintenance, good TypeScript support

**Implementation Pattern**:
- Wrapper components in `src/components/visualizations/`
- Consistent theming via Recharts config + Tailwind colors
- Chart types: `PieChart`, `BarChart`, `LineChart`, `AreaChart` (for debt projections)

**Alternatives Considered**:
- Victory: More complex API, heavier bundle
- Chart.js: Imperative API, less React-friendly
- D3.js: Too low-level, unnecessary complexity

## 5. CSV Parsing Strategy

### Decision: Rust csv Crate with Flexible Column Mapping
- **Rationale**: CSV parsing in Rust backend ensures performance and safety. User-defined column mapping stored in SQLite for reuse.

**Architecture**:
1. Tauri command `parse_csv(file_path, mapping_id?)`
2. If mapping_id provided: Load saved mapping from `column_mappings` table
3. Else: Return column preview for user to map via UI
4. User selects columns in UI → Save mapping → Re-invoke with mapping_id
5. Parse CSV using `csv` crate, apply mapping, insert transactions

**Duplicate Detection**:
- Hash of (date + amount + description) stored in transactions table
- Before insert: Check hash against existing, skip if match

**Error Handling**:
- Invalid CSV format: Return structured error to UI
- Missing required columns: Prompt user for mapping
- Parse errors: Skip row, log to error report, show summary to user

**Alternatives Considered**:
- Frontend parsing (Papa Parse): Rejected - file size limits, slower for large CSVs
- Fixed column format: Rejected - too rigid, won't work with different bank exports

## 6. Debt Calculation Algorithms

### Decision: Iterative Month-by-Month Simulation
- **Rationale**: Simple, accurate, testable. Simulate payment allocation month-by-month until all debts paid.

**Avalanche Algorithm**:
```
Sort debts by interest_rate DESC
While total_balance > 0:
  Pay minimums on all debts
  Allocate remaining monthly_amount to highest-rate debt
  Apply interest to all balances
  Record payment, update balances
  Increment month
```

**Snowball Algorithm**:
```
Sort debts by balance ASC
While total_balance > 0:
  Pay minimums on all debts
  Allocate remaining monthly_amount to lowest-balance debt
  Apply interest to all balances
  Record payment, update balances
  Increment month
```

**Interest Calculation**:
- Monthly interest = `balance * (annual_rate / 12)`
- Applied before each payment in simulation

**Output**:
- Payment schedule: Array of {month, debt_id, amount}
- Projections: {payoff_date, total_interest, month_by_month_balance}

**Alternatives Considered**:
- Financial formulas (amortization): Rejected - complex for multiple debts, doesn't handle variable payments well
- External library: Rejected - simple enough to implement, no suitable Rust crate found

## 7. Testing Strategy

### Decision: Multi-layer Testing with Vitest + Tauri Test
- **Rationale**: Constitutional requirement for TDD. Vitest provides fast, modern testing for both frontend and backend (Rust tests).

**Test Layers**:

1. **Unit Tests** (Vitest for TS, Rust `#[cfg(test)]` for Rust):
   - Pure functions: debt calculations, categorization logic, date utils
   - React components: test props, state, rendering
   - Rust services: database operations, business logic

2. **Contract Tests** (Vitest + Mock Tauri):
   - Each Tauri command tested with `@tauri-apps/api/mocks`
   - Verify request/response schemas
   - Test error handling

3. **Integration Tests** (Vitest + Tauri Test):
   - End-to-end scenarios from acceptance criteria
   - Real SQLite DB (in-memory for tests)
   - Test complete workflows: CSV import → categorization → analysis

4. **Visual/Component Tests** (Vitest + Testing Library):
   - User interactions: clicks, form submissions
   - Chart rendering: verify data passed to Recharts
   - Navigation flows

**Mocking Strategy**:
- Mock Tauri commands in frontend tests: `mockIPC()` from `@tauri-apps/api/mocks`
- Mock SQLite in Rust tests: Use `:memory:` database
- Mock file system: Use temp directories for CSV tests

**Coverage Goals**:
- Unit: >80% coverage
- Integration: All 7 acceptance scenarios
- Contract: 100% of Tauri commands

**Alternatives Considered**:
- Jest: Rejected - Vitest is faster, better ESM support, Vite-native
- Playwright: Rejected - overkill for desktop app, Tauri Test sufficient
- Manual testing only: Rejected - violates constitution

## 8. State Management Architecture

### Decision: Zustand with Domain-based Stores
- **Rationale**: Lightweight, TypeScript-friendly, no boilerplate. Simpler than Redux, more scalable than Context API.

**Store Structure**:
- `useTransactionStore`: transactions list, filters, selected transaction
- `useDebtStore`: debts list, active plan, calculation results
- `useAnalyticsStore`: spending summaries, chart data, date ranges
- `useUIStore`: sidebar state, modals, loading states
- `useSettingsStore`: preferences, column mappings, categories

**Pattern**:
```typescript
// Stores call Tauri commands, update local state
const useTransactionStore = create<TransactionStore>((set, get) => ({
  transactions: [],
  async loadTransactions() {
    const data = await invoke('list_transactions')
    set({ transactions: data })
  }
}))
```

**Rationale Over Alternatives**:
- Redux: Rejected - too much boilerplate for desktop app scope
- Context API: Rejected - performance issues with frequent updates (charts)
- Jotai/Recoil: Rejected - atomic approach unnecessary, Zustand simpler

## 9. Build & Development Workflow

### Decision: Vite + Tauri CLI with Hot Reload
- **Rationale**: Vite provides instant HMR for frontend, Tauri CLI handles Rust compilation and bundling.

**Dev Workflow**:
1. `bun install` (following Opcode pattern for faster installs)
2. `bun run tauri dev` - starts Vite dev server + Tauri window
3. HMR for React changes, Rust recompile on save
4. SQLite database in `~/.local/share/budget-balancer/` (dev mode)

**Build Workflow**:
1. `bun run tauri build` - optimized production bundle
2. Platform-specific installers: `.msi` (Windows), `.dmg` (macOS), `.AppImage` (Linux)
3. Code signing: Deferred to deployment phase

**Testing Workflow**:
1. `bun run test` - Vitest unit/integration tests
2. `bun run test:ui` - Vitest UI mode for debugging
3. `bun run tauri test` - Tauri integration tests

**Alternatives Considered**:
- npm/pnpm: Rejected - Bun is faster, matches Opcode setup
- Webpack: Rejected - Vite is faster, better DX
- Manual Rust compilation: Rejected - Tauri CLI handles cross-platform builds

## Summary of Key Decisions

| Area | Decision | Primary Rationale |
|------|----------|-------------------|
| Architecture | Tauri commands + Zustand stores | Type-safe, simple state sync |
| UI Framework | Radix UI + Tailwind (Opcode pattern) | Accessible, customizable, proven |
| Database | SQLite with normalized schema | Local storage, good performance |
| Charts | Recharts | React-native, declarative, complete |
| CSV | Rust csv crate + flexible mapping | Performance, user control |
| Debt Calc | Iterative simulation | Accurate, testable, simple |
| Testing | Vitest + Tauri Test (multi-layer) | Constitutional compliance, modern |
| State | Zustand domain stores | Lightweight, TypeScript-friendly |
| Build | Vite + Tauri CLI + Bun | Fast, modern, proven (Opcode) |

All decisions align with constitutional principles: maintainability (TypeScript, established patterns), testability (comprehensive test strategy), substance over flash (functional UI from Opcode).
