# Tasks: Budget Balancer - Debt Management & Spending Insights

**Input**: Design documents from `/home/dwalleck/repos/budget-balancer/specs/001-build-an-application/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md
**Tech Stack**: TypeScript 5.x / React 18 + Tauri 2, Radix UI, Tailwind CSS, Vite, Zustand, Vitest, SQLite

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- All tasks follow TDD: tests before implementation

---

## Phase 3.1: Setup & Database Foundation

- [x] T001 Initialize Tauri 2 project structure with React frontend and Rust backend per plan.md structure
- [x] T002 Configure dependencies: React 18, Radix UI, Tailwind CSS, Vite, Zustand, Vitest in package.json
- [x] T003 Configure Rust dependencies: tauri, sqlx, tokio, serde in src-tauri/Cargo.toml
- [x] T004 [P] Set up ESLint and Prettier for TypeScript/React in .eslintrc and .prettierrc
- [x] T005 [P] Set up Clippy and rustfmt for Rust in src-tauri/.cargo/config.toml
- [x] T006 Create SQLite schema migrations in src-tauri/migrations/001_initial_schema.sql with all 8 tables from data-model.md
- [x] T007 Implement database initialization with connection pooling (DbPool, max 5 connections) in src-tauri/src/db/mod.rs
- [x] T008 Seed predefined categories and category_rules in src-tauri/src/db/seed.rs
- [x] T009 Create constants module in src-tauri/src/constants.rs (DEFAULT_CATEGORY_ID, MAX_CSV_ROWS, etc.)
- [x] T010 Set up Vitest configuration with Tauri mocks in vitest.config.ts
- [x] T011 Configure cargo-llvm-cov for backend test coverage in src-tauri/Cargo.toml
- [ ] T011a **[PRIORITY]** Set up GitHub Actions CI/CD workflow in .github/workflows/ci.yml
  - Build for all platforms (Windows, macOS, Linux)
  - Run linters (ESLint, Prettier, Clippy, rustfmt)
  - Execute backend tests (cargo test) - **continue on test failure**
  - Execute frontend tests (bun test) - **continue on test failure**
  - **BLOCK on build failures** (compilation errors)
  - Generate test reports and upload as artifacts

### Code Quality & Best Practices (Based on Audit)
- [ ] T011b **[CRITICAL]** Fix production unwrap() usage in Rust code
  - Replace unwrap() in avalanche_calculator.rs:81 with unwrap_or(Ordering::Equal)
  - Replace unwrap() in snowball_calculator.rs:65 with unwrap_or(Ordering::Equal)
  - Fix Mutex unwrap() in rate_limiter.rs:193 with proper error handling
- [ ] T011c **[CRITICAL]** Fix TypeScript any types in visualization components
  - Create proper interfaces for Recharts props in SpendingPieChart.tsx
  - Create proper interfaces for Recharts props in SpendingBarChart.tsx
  - Create proper interfaces for Recharts props in TrendsLineChart.tsx
  - Fix any type in AccountCreationDialog.tsx:75
- [ ] T011d **[HIGH]** Fix ESLint warnings and errors
  - Remove unused variables (prefix with _ or remove imports)
  - Fix useEffect dependency warnings in TransactionList.tsx and TransactionsPage.tsx
  - Run `bun run lint --fix` for auto-fixable issues
- [ ] T011e **[HIGH]** Fix Clippy warnings in Rust code
  - Fix manual range contains in debt_commands.rs:158
  - Fix unnecessary lazy evaluation in debt_commands.rs:318
  - Run `cargo clippy --fix` for auto-fixable issues
- [ ] T011f [P] Add React.memo to expensive components
  - Memoize SpendingPieChart component
  - Memoize SpendingBarChart component
  - Memoize TrendsLineChart component
- [ ] T011g [P] Add documentation to public Rust APIs
  - Add /// comments to all public service functions
  - Add examples for complex debt calculation algorithms
  - Document error cases for command handlers
- [ ] T011h [P] Add JSDoc to public React components
  - Document props and usage for all page components
  - Add examples for complex components (charts, forms)
  - Document custom hooks

---

## Phase 3.2: Backend - Data Models

### Data Models & Database Layer
- [x] T012 [P] Create Account model in src-tauri/src/models/account.rs
- [x] T013 [P] Create Transaction model with hash generation in src-tauri/src/models/transaction.rs
- [x] T014 [P] Create Category model in src-tauri/src/models/category.rs
- [x] T015 [P] Create CategoryRule model in src-tauri/src/models/category_rule.rs
- [x] T016 [P] Create ColumnMapping model in src-tauri/src/models/column_mapping.rs
- [x] T017 [P] Create Debt model in src-tauri/src/models/debt.rs
- [x] T018 [P] Create DebtPayment model in src-tauri/src/models/debt_payment.rs
- [x] T019 [P] Create SpendingTarget model in src-tauri/src/models/spending_target.rs

---

## Phase 3.3: Backend - Contract Tests (TDD) ⚠️ MUST COMPLETE BEFORE 3.4

**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### Account Management Tests (contracts/accounts.md - 4 commands)
- [x] T020 [P] Contract test for create_account command in src-tauri/tests/integration/accounts_test.rs
- [x] T021 [P] Contract test for list_accounts command in src-tauri/tests/integration/accounts_test.rs
- [ ] T022 [P] Contract test for update_account command in src-tauri/tests/integration/accounts_test.rs
- [ ] T023 [P] Contract test for delete_account with cascade in src-tauri/tests/integration/accounts_test.rs

### Transaction Management Tests (contracts/transactions.md - 11 commands)
- [x] T024 [P] Contract test for import_csv command in src-tauri/tests/integration/transactions_test.rs
- [x] T025 [P] Contract test for list_transactions with pagination (25/page) in src-tauri/tests/integration/transactions_test.rs
- [ ] T026 [P] Contract test for search_transactions with debounce in src-tauri/tests/integration/transactions_test.rs
- [x] T027 [P] Contract test for update_transaction_category command in src-tauri/tests/integration/transactions_test.rs
- [ ] T028 [P] Contract test for delete_transaction command in src-tauri/tests/integration/transactions_test.rs
- [ ] T029 [P] Contract test for bulk_delete_transactions (max 1000 IDs) in src-tauri/tests/integration/transactions_test.rs
- [ ] T030 [P] Contract test for bulk_update_category command in src-tauri/tests/integration/transactions_test.rs
- [x] T031 [P] Contract test for categorize_transaction with rules in src-tauri/tests/integration/transactions_test.rs
- [x] T032 [P] Contract test for export_transactions to CSV/JSON in src-tauri/tests/integration/transactions_test.rs
- [ ] T033 [P] Contract test for save_column_mapping with upsert in src-tauri/tests/integration/column_mappings_test.rs
- [ ] T034 [P] Contract test for create_category command in src-tauri/tests/integration/categories_test.rs

### Category Management Tests (contracts/categories.md - 4 commands)
- [ ] T035 [P] Contract test for create_category (custom) in src-tauri/tests/integration/categories_test.rs
- [ ] T036 [P] Contract test for list_categories with type filter in src-tauri/tests/integration/categories_test.rs
- [ ] T037 [P] Contract test for update_category (custom only) in src-tauri/tests/integration/categories_test.rs
- [ ] T038 [P] Contract test for delete_category with reassignment to Uncategorized in src-tauri/tests/integration/categories_test.rs

### Category Rules Tests (contracts/category_rules.md - 4 commands)
- [ ] T039 [P] Contract test for create_category_rule with pattern normalization in src-tauri/tests/integration/category_rules_test.rs
- [ ] T040 [P] Contract test for list_category_rules ordered by priority in src-tauri/tests/integration/category_rules_test.rs
- [ ] T041 [P] Contract test for update_category_rule command in src-tauri/tests/integration/category_rules_test.rs
- [ ] T042 [P] Contract test for delete_category_rule command in src-tauri/tests/integration/category_rules_test.rs

### Column Mapping Tests (contracts/column_mappings.md - 5 commands)
- [ ] T043 [P] Contract test for save_column_mapping with upsert behavior in src-tauri/tests/integration/column_mappings_test.rs
- [ ] T044 [P] Contract test for list_column_mappings sorted by name in src-tauri/tests/integration/column_mappings_test.rs
- [ ] T045 [P] Contract test for get_column_mapping by ID or source_name in src-tauri/tests/integration/column_mappings_test.rs
- [ ] T046 [P] Contract test for update_column_mapping command in src-tauri/tests/integration/column_mappings_test.rs
- [ ] T047 [P] Contract test for delete_column_mapping command in src-tauri/tests/integration/column_mappings_test.rs

### Debt Management Tests (contracts/debts.md - 9 commands)
- [x] T048 [P] Contract test for create_debt command in src-tauri/tests/integration/debts_test.rs
- [x] T049 [P] Contract test for list_debts command in src-tauri/tests/integration/debts_test.rs
- [x] T050 [P] Contract test for update_debt command in src-tauri/tests/integration/debts_test.rs
- [ ] T051 [P] Contract test for delete_debt with cascade in src-tauri/tests/integration/debts_test.rs
- [x] T052 [P] Contract test for calculate_payoff_plan avalanche strategy in src-tauri/tests/integration/debts_test.rs
- [x] T053 [P] Contract test for calculate_payoff_plan snowball strategy in src-tauri/tests/integration/debts_test.rs
- [x] T054 [P] Contract test for get_payoff_plan command in src-tauri/tests/integration/debts_test.rs
- [x] T055 [P] Contract test for record_debt_payment command in src-tauri/tests/integration/debts_test.rs
- [x] T056 [P] Contract test for get_debt_progress with payment history in src-tauri/tests/integration/debts_test.rs
- [x] T057 [P] Contract test for compare_strategies avalanche vs snowball in src-tauri/tests/integration/debts_test.rs

### Analytics Tests (contracts/analytics.md - 7 commands)
- [x] T058 [P] Contract test for get_spending_by_category with percentages in src-tauri/tests/integration/analytics_test.rs
- [x] T059 [P] Contract test for get_spending_trends by interval in src-tauri/tests/integration/analytics_test.rs
- [x] T060 [P] Contract test for get_spending_targets_progress with status in src-tauri/tests/integration/analytics_test.rs
- [x] T061 [P] Contract test for create_spending_target command in src-tauri/tests/integration/analytics_test.rs
- [x] T062 [P] Contract test for update_spending_target command in src-tauri/tests/integration/analytics_test.rs
- [x] T063 [P] Contract test for get_dashboard_summary command in src-tauri/tests/integration/analytics_test.rs
- [x] T064 [P] Contract test for export_analytics_report to PDF/XLSX in src-tauri/tests/integration/analytics_test.rs

---

## Phase 3.4: Backend - Services & Commands Implementation (ONLY after tests fail)

### Account Services & Commands
- [ ] T065 Create AccountService with CRUD operations in src-tauri/src/services/account_service.rs
- [ ] T066 Implement account commands (create, list, update, delete) in src-tauri/src/commands/accounts.rs

### Category Services & Commands
- [ ] T067 Create CategoryService with predefined protection in src-tauri/src/services/category_service.rs
- [ ] T068 Implement category commands (create, list, update, delete) in src-tauri/src/commands/categories.rs

### Category Rule Services & Commands
- [ ] T069 Create CategoryRuleService with pattern matching in src-tauri/src/services/category_rule_service.rs
- [ ] T070 Implement category_rule commands (create, list, update, delete) in src-tauri/src/commands/category_rules.rs

### Column Mapping Services & Commands
- [ ] T071 Create ColumnMappingService with upsert logic in src-tauri/src/services/column_mapping_service.rs
- [ ] T072 Implement column_mapping commands (save, list, get, update, delete) in src-tauri/src/commands/column_mappings.rs

### Transaction Services & Commands (Enhanced)
- [x] T073 Create TransactionService with duplicate detection in src-tauri/src/services/transaction_service.rs
- [x] T074 Create CsvImportService with streaming and progress in src-tauri/src/services/csv_import_service.rs
- [x] T075 Create CategorizationService with rule priority in src-tauri/src/services/categorization_service.rs
- [ ] T076 Implement search_transactions command with debounce in src-tauri/src/commands/transactions.rs
- [ ] T077 Implement delete_transaction command in src-tauri/src/commands/transactions.rs
- [ ] T078 Implement bulk transaction commands (bulk_delete, bulk_update_category) in src-tauri/src/commands/transactions.rs

### Debt Services & Commands (Enhanced)
- [x] T079 Create DebtService with CRUD operations in src-tauri/src/services/debt_service.rs
- [x] T080 Create DebtCalculationService for avalanche/snowball algorithms in src-tauri/src/services/debt_calculation_service.rs
- [ ] T081 Implement delete_debt command with cascade in src-tauri/src/commands/debts.rs

### Analytics Services & Commands
- [x] T082 Create AnalyticsService for spending calculations in src-tauri/src/services/analytics_service.rs
- [x] T083 Create ExportService for CSV/JSON/PDF export in src-tauri/src/services/export_service.rs

### Security & Validation
- [ ] T084 Implement input validation middleware with sanitization in src-tauri/src/middleware/validation.rs
- [ ] T085 Implement rate limiter for CSV imports (2 sec throttle) in src-tauri/src/middleware/rate_limiter.rs
- [ ] T086 Create custom error types with safe user messages in src-tauri/src/errors.rs
- [ ] T087 Add SQL injection prevention tests for all queries in src-tauri/tests/security/sql_injection_test.rs

---

## Phase 3.5: Frontend - TypeScript Types & Stores

### Type Definitions
- [x] T088 [P] Create TypeScript types for Account in src/types/account.ts
- [x] T089 [P] Create TypeScript types for Transaction in src/types/transaction.ts
- [x] T090 [P] Create TypeScript types for Category in src/types/category.ts
- [ ] T091 [P] Create TypeScript types for CategoryRule in src/types/category_rule.ts
- [ ] T092 [P] Create TypeScript types for ColumnMapping in src/types/column_mapping.ts
- [x] T093 [P] Create TypeScript types for Debt and DebtPlan in src/types/debt.ts
- [x] T094 [P] Create TypeScript types for Analytics responses in src/types/analytics.ts

### Zustand Stores
- [x] T095 [P] Create accountStore with CRUD actions in src/stores/accountStore.ts
- [x] T096 [P] Create transactionStore with pagination state in src/stores/transactionStore.ts
- [x] T097 [P] Create categoryStore with predefined/custom separation in src/stores/categoryStore.ts
- [ ] T098 [P] Create categoryRuleStore in src/stores/categoryRuleStore.ts
- [ ] T099 [P] Create columnMappingStore in src/stores/columnMappingStore.ts
- [x] T100 [P] Create debtStore with plan state in src/stores/debtStore.ts
- [x] T101 [P] Create analyticsStore with caching in src/stores/analyticsStore.ts
- [x] T102 [P] Create uiStore for loading/error states in src/stores/uiStore.ts

---

## Phase 3.6: Frontend - Base Components & UI Kit

### Radix UI Wrappers (following Opcode pattern)
- [x] T103 [P] Create Button component with Radix in src/components/ui/Button.tsx
- [x] T104 [P] Create Dialog component with Radix in src/components/ui/Dialog.tsx
- [x] T105 [P] Create Select component with Radix in src/components/ui/Select.tsx
- [x] T106 [P] Create Table component with Radix in src/components/ui/Table.tsx
- [x] T107 [P] Create Tabs component with Radix in src/components/ui/Tabs.tsx
- [ ] T108 [P] Create Toast component with Radix in src/components/ui/Toast.tsx
- [x] T109 [P] Create ProgressBar component with Radix in src/components/ui/ProgressBar.tsx
- [ ] T110 [P] Create Checkbox component with Radix in src/components/ui/Checkbox.tsx

### Layout & Navigation
- [x] T111 Create AppLayout with sidebar navigation in src/components/layout/AppLayout.tsx
- [x] T112 Create Sidebar with route navigation in src/components/layout/Sidebar.tsx
- [ ] T113 Create Header with breadcrumbs in src/components/layout/Header.tsx

### Reusable Components
- [ ] T114 [P] Create ConfirmationDialog with count display in src/components/common/ConfirmationDialog.tsx
- [x] T115 [P] Create LoadingSpinner component in src/components/common/LoadingSpinner.tsx
- [x] T116 [P] Create EmptyState component in src/components/common/EmptyState.tsx
- [ ] T117 [P] Create ErrorBoundary component in src/components/common/ErrorBoundary.tsx
- [ ] T118 [P] Create Pagination component in src/components/common/Pagination.tsx

---

## Phase 3.7: Frontend - Enhanced Feature Components

### Account Management UI (NEW - FR-001 to FR-004)
- [ ] T119 Create AccountList component with account cards in src/components/accounts/AccountList.tsx
- [x] T120 Create AccountFormDialog for create/edit in src/components/accounts/AccountFormDialog.tsx
- [ ] T121 Create AccountDeleteDialog with cascade warning in src/components/accounts/AccountDeleteDialog.tsx
- [ ] T122 Create AccountsPage with account management in src/pages/AccountsPage.tsx

### Transaction Management UI (ENHANCED - FR-014 to FR-021)
- [x] T123 Create TransactionList with pagination (25/page) in src/components/transactions/TransactionList.tsx
- [ ] T124 Create TransactionRow with checkbox for bulk select in src/components/transactions/TransactionRow.tsx
- [ ] T125 Create TransactionSearchBar with 500ms debounce in src/components/transactions/TransactionSearchBar.tsx
- [ ] T126 Create TransactionFilters (date, category, account) in src/components/transactions/TransactionFilters.tsx
- [ ] T127 Create BulkActionToolbar with delete/update actions in src/components/transactions/BulkActionToolbar.tsx
- [ ] T128 Create TransactionDeleteDialog with count in src/components/transactions/TransactionDeleteDialog.tsx
- [x] T129 Create CategorySelectDialog for manual categorization in src/components/transactions/CategorySelectDialog.tsx
- [x] T130 Create TransactionsPage with all transaction features in src/pages/TransactionsPage.tsx

### CSV Import UI (ENHANCED - FR-007 to FR-009)
- [x] T131 Create CsvImportDialog with file picker in src/components/import/CsvImportDialog.tsx
- [x] T132 Create ColumnMappingForm with preview in src/components/import/ColumnMappingForm.tsx
- [ ] T133 Create ColumnMappingSelector for saved mappings in src/components/import/ColumnMappingSelector.tsx
- [ ] T134 Create ImportProgress with row count in src/components/import/ImportProgress.tsx
- [ ] T135 Create ColumnMappingManagement page in src/pages/ColumnMappingManagement.tsx

### Category Management UI (NEW - FR-022 to FR-025)
- [ ] T136 Create CategoryList with predefined/custom sections in src/components/categories/CategoryList.tsx
- [ ] T137 Create CategoryFormDialog for custom categories in src/components/categories/CategoryFormDialog.tsx
- [ ] T138 Create CategoryDeleteDialog with reassignment warning in src/components/categories/CategoryDeleteDialog.tsx
- [ ] T139 Create CategoryRuleList ordered by priority in src/components/categories/CategoryRuleList.tsx
- [ ] T140 Create CategoryRuleFormDialog for pattern editing in src/components/categories/CategoryRuleFormDialog.tsx
- [ ] T141 Create CategoriesPage with category & rule management in src/pages/CategoriesPage.tsx

### Debt Management UI (ENHANCED - FR-032 to FR-040)
- [x] T142 Create DebtList with debt cards showing balance/rate in src/components/debts/DebtList.tsx
- [x] T143 Create DebtFormDialog for create/edit in src/components/debts/DebtFormDialog.tsx
- [ ] T144 Create DebtDeleteDialog with cascade warning in src/components/debts/DebtDeleteDialog.tsx
- [x] T145 Create PayoffPlannerForm with strategy selector in src/components/debts/PayoffPlannerForm.tsx
- [x] T146 Create PayoffScheduleTable with monthly breakdown in src/components/debts/PayoffScheduleTable.tsx
- [ ] T147 Create StrategyComparison showing avalanche vs snowball in src/components/debts/StrategyComparison.tsx
- [ ] T148 Create DebtPaymentForm for recording payments in src/components/debts/DebtPaymentForm.tsx
- [ ] T149 Create DebtProgressCard with payment history in src/components/debts/DebtProgressCard.tsx
- [x] T150 Create DebtsPage with debt management in src/pages/DebtsPage.tsx
- [x] T151 Create PayoffPlannerPage with plan creation in src/pages/PayoffPlannerPage.tsx

### Analytics & Visualization UI (FR-041 to FR-048)
- [x] T152 [P] Create PieChart component using Recharts in src/components/visualizations/PieChart.tsx
- [x] T153 [P] Create BarChart component using Recharts in src/components/visualizations/BarChart.tsx
- [x] T154 [P] Create LineChart component using Recharts in src/components/visualizations/LineChart.tsx
- [x] T155 [P] Create ProgressBarChart component in src/components/visualizations/ProgressBarChart.tsx
- [x] T156 Create SpendingByCategoryCard with pie chart in src/components/analytics/SpendingByCategoryCard.tsx
- [x] T157 Create SpendingTrendsCard with line chart in src/components/analytics/SpendingTrendsCard.tsx
- [ ] T158 Create SpendingTargetsList with progress bars in src/components/analytics/SpendingTargetsList.tsx
- [ ] T159 Create SpendingTargetFormDialog for create/edit in src/components/analytics/SpendingTargetFormDialog.tsx
- [x] T160 Create DateRangeSelector with presets in src/components/analytics/DateRangeSelector.tsx
- [x] T161 Create DashboardSummaryCard with key metrics in src/components/analytics/DashboardSummaryCard.tsx
- [x] T162 Create DashboardPage with overview widgets in src/pages/DashboardPage.tsx
- [x] T163 Create AnalyticsPage with detailed charts in src/pages/AnalyticsPage.tsx

---

## Phase 3.8: Integration Tests (E2E Scenarios from quickstart.md)

- [ ] T164 [P] Integration test: Scenario 1 - CSV upload and categorization in tests/integration/csv_import.test.ts
- [ ] T165 [P] Integration test: Scenario 2 - Spending analysis with pie charts in tests/integration/spending_analysis.test.ts
- [ ] T166 [P] Integration test: Scenario 3 - Avalanche debt payoff plan in tests/integration/avalanche_payoff.test.ts
- [ ] T167 [P] Integration test: Scenario 4 - Snowball debt payoff plan in tests/integration/snowball_payoff.test.ts
- [ ] T168 [P] Integration test: Scenario 5 - Spending targets progress in tests/integration/spending_targets.test.ts
- [ ] T169 [P] Integration test: Scenario 6 - Debt progress visualizations in tests/integration/debt_progress.test.ts
- [ ] T170 [P] Integration test: Scenario 7 - Transaction update triggers recalculation in tests/integration/auto_update.test.ts

---

## Phase 3.9: Polish & Optimization

### User Experience (FR-050 confirmation dialogs)
- [ ] T171 Implement confirmation dialogs for all delete operations with count display
- [ ] T172 Add loading states to all async operations in all pages
- [ ] T173 Add toast notifications for success/error feedback using Toast component
- [ ] T174 Add empty states for lists with no data using EmptyState component
- [ ] T175 Add error boundaries around major sections using ErrorBoundary component
- [ ] T176 Implement optimistic UI updates in transaction operations

### Performance (FR-014/FR-015 pagination, FR-017 search debounce)
- [ ] T177 Add virtualization to TransactionList for >100 items using react-window
- [ ] T178 Implement debounced search in TransactionSearchBar (500ms per spec FR-017)
- [ ] T179 Add caching to analyticsStore for expensive calculations
- [ ] T180 Optimize chart rendering with React.memo on visualization components
- [ ] T181 Add indexes to SQLite queries per data-model.md

### Accessibility (WCAG AA compliance per spec FR-044) ⚠️ CRITICAL GAPS FOUND
**See ACCESSIBILITY_AUDIT.md for detailed findings (Score: 5.5/10)**

#### Priority 1 (Blocking - Must Fix for WCAG AA)
- [ ] T182 **[CRITICAL]** Add htmlFor and id to ALL form inputs
  - AccountCreationDialog.tsx (3 inputs)
  - ColumnMappingForm.tsx (4 inputs)
  - DebtPlannerPage.tsx (all debt form inputs)
  - Add aria-required="true" to required fields
- [ ] T183 **[CRITICAL]** Add text/icon indicators for color-coded information (per FR-044)
  - DashboardPage.tsx: Add ↑/↓ icons for income/expense
  - Error/success messages: Add icons, not just red/green backgrounds
- [ ] T184 **[CRITICAL]** Add alternative text/data tables for all charts
  - SpendingPieChart.tsx: Add sr-only data table
  - SpendingBarChart.tsx: Add sr-only data table
  - TrendsLineChart.tsx: Add sr-only data table
  - Add role="img" and aria-label to chart containers
- [ ] T185 **[CRITICAL]** Add required field indicators with asterisk and aria-required
  - Visual indicator: <span className="text-red-600">*</span>
  - Programmatic: aria-required="true"
- [ ] T186 **[CRITICAL]** Add skip to main content link in AppLayout.tsx

#### Priority 2 (High - Should Fix)
- [ ] T187 Add aria-label to all icon-only buttons
  - Edit buttons, delete buttons, action icons
- [ ] T188 Add role="alert" and aria-live to error messages
  - All error message containers
- [ ] T189 Associate error messages with inputs using aria-describedby
  - Add id to error spans
  - Reference in input aria-describedby
- [ ] T190 Add scope="col" attributes to table headers in Table.tsx

#### Priority 3 (Testing & Validation)
- [ ] T191 Test full keyboard navigation (no mouse) across all pages
- [ ] T192 Test with screen reader (NVDA/JAWS/VoiceOver)
- [ ] T193 Run automated a11y audit with axe-core browser extension
- [ ] T194 Add live regions for dynamic content updates
- [ ] T195 Install and configure @axe-core/react for automated testing

### Testing & Quality
- [ ] T196 [P] Unit tests for accountStore in tests/unit/stores/accountStore.test.ts
- [ ] T197 [P] Unit tests for transactionStore in tests/unit/stores/transactionStore.test.ts
- [ ] T198 [P] Unit tests for debtStore in tests/unit/stores/debtStore.test.ts
- [ ] T199 [P] Unit tests for analyticsStore in tests/unit/stores/analyticsStore.test.ts
- [ ] T200 [P] Component tests for AccountList in tests/unit/components/AccountList.test.tsx
- [ ] T201 [P] Component tests for TransactionList in tests/unit/components/TransactionList.test.tsx
- [ ] T202 [P] Component tests for DebtList in tests/unit/components/DebtList.test.tsx
- [ ] T203 [P] Component tests for PieChart in tests/unit/components/PieChart.test.tsx
- [ ] T204 Run backend coverage report (target: >60%) with cargo llvm-cov
- [ ] T205 Run frontend coverage report (target: >70%) with vitest --coverage
- [ ] T206 Fix any remaining ESLint warnings (beyond T011d)
- [ ] T207 Fix any remaining Clippy warnings (beyond T011e)

### Documentation
- [ ] T208 [P] Update CLAUDE.md with final project structure
- [ ] T209 [P] Create user guide in docs/USER_GUIDE.md
- [ ] T210 [P] Document CSV import format in docs/CSV_FORMAT.md
- [ ] T211 [P] Add inline code comments for complex algorithms (debt calculation, categorization)

---

## Dependencies

### Critical Path
1. **Setup** (T001-T011) → All other phases
2. **Backend Tests** (T020-T064) → Backend Implementation (T065-T087)
3. **Models** (T012-T019) → Services (T065-T083) → Commands (T066-T083)
4. **Types** (T088-T094) → Stores (T095-T102)
5. **Base Components** (T103-T118) → Feature Components (T119-T163)
6. **All Implementation** → Integration Tests (T164-T170)
7. **All Implementation** → Polish (T171-T202)

### Parallel Execution Groups

**Group 1 - Setup (can run together)**:
```
T004 (ESLint/Prettier), T005 (Clippy/rustfmt), T010 (Vitest config), T011a (CI/CD setup)
```

**Group 2 - Data Models (all independent)**:
```
T012-T019 (all model files in parallel)
```

**Group 3 - Backend Contract Tests (all independent)**:
```
T020-T064 (all contract tests in parallel - 45 tests across different files)
```

**Group 4 - Frontend Types (all independent)**:
```
T088-T094 (all type files in parallel)
```

**Group 5 - Zustand Stores (all independent)**:
```
T095-T102 (all store files in parallel)
```

**Group 6 - UI Components (all independent)**:
```
T103-T110 (Radix wrappers in parallel)
T114-T118 (reusable components in parallel)
```

**Group 7 - Visualization Components (all independent)**:
```
T152-T155 (chart components in parallel)
```

**Group 8 - Integration Tests (all independent)**:
```
T164-T170 (all scenario tests in parallel)
```

**Group 9 - Unit Tests (all independent)**:
```
T187-T194 (all unit tests in parallel)
```

**Group 10 - Documentation (all independent)**:
```
T199-T202 (all docs in parallel)
```

---

## Validation Checklist

- [x] All contracts (44 commands total) have corresponding tests (T020-T064)
  - 4 Account commands (T020-T023)
  - 11 Transaction commands (T024-T034)
  - 4 Category commands (T035-T038)
  - 4 Category Rule commands (T039-T042)
  - 5 Column Mapping commands (T043-T047)
  - 9 Debt commands (T048-T057 includes delete_debt)
  - 7 Analytics commands (T058-T064)
- [x] All entities (8 tables) have model tasks (T012-T019)
- [x] All tests come before implementation (Phase 3.3 before 3.4)
- [x] Parallel tasks truly independent (different files, no shared state)
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task
- [x] All 7 integration scenarios from quickstart.md covered (T164-T170)
- [x] Security requirements addressed (T084-T087: validation, rate limiting, error handling, SQL injection)
- [x] Performance requirements addressed (T177-T181: virtualization, debounce, caching, indexes)
- [x] Accessibility requirements addressed (WCAG AA per FR-044: T182-T186)
- [x] Testing coverage targets specified (>60% backend, >70% frontend: T195-T196)
- [x] Confirmation dialogs for all destructive operations (per FR-050: T171)
- [x] Pagination (25/page per FR-014: T123-T127)
- [x] Search debounce (500ms per FR-017: T125, T178)

---

## New Features Coverage (from 2025-10-05 spec updates)

### Account Management (FR-001 to FR-004)
- **Backend**: T020-T023, T065-T066
- **Frontend**: T119-T122
- **Features**: CRUD operations, cascade delete, balance tracking

### Enhanced Transaction Management (FR-014 to FR-021)
- **Backend**: T026, T028-T030, T076-T078
- **Frontend**: T124-T128
- **Features**: Pagination, search, delete, bulk operations

### Category Management (FR-022 to FR-025)
- **Backend**: T035-T042, T067-T070
- **Frontend**: T136-T141
- **Features**: Custom categories, predefined protection, reassignment on delete

### Column Mapping Management (FR-007 to FR-009)
- **Backend**: T043-T047, T071-T072
- **Frontend**: T133-T135
- **Features**: Save/load mappings, upsert behavior, mapping management

### Enhanced Debt Management (FR-032 to FR-040)
- **Backend**: T051, T081
- **Frontend**: T144, T147-T149
- **Features**: Delete with cascade, payment recording, progress tracking

### Confirmation Dialogs (FR-050)
- **Frontend**: T114, T121, T128, T144, T171
- **Features**: Count display, explicit confirmation for all destructive operations

---

## Implementation Status

### ✅ COMPLETED (from previous work)
- Setup & Database (T001-T011): 100%
- Data Models (T012-T019): 100%
- Core Tauri Commands: 26 commands implemented
- Basic Frontend: Dashboard, Transactions, Debts, Analytics
- Backend Test Infrastructure: 56/68 passing tests (82%)

### 🔨 IN PROGRESS / TODO
- **Account Management**: T022-T023 (tests), T065-T066 (backend), T119-T122 (frontend)
- **Enhanced Transactions**: T026, T028-T030 (tests), T076-T078 (backend), T124-T128 (frontend)
- **Category Management**: T035-T042 (tests), T067-T070 (backend), T136-T141 (frontend)
- **Category Rules**: T039-T042 (tests), T069-T070 (backend), T139-T140 (frontend)
- **Column Mappings**: T043-T047 (tests), T071-T072 (backend), T133-T135 (frontend)
- **Enhanced Debt**: T051 (test), T081 (backend), T144, T147-T149 (frontend)
- **Confirmation Dialogs**: T114, T121, T128, T144 (frontend)
- **Security**: T084-T087 (validation, rate limiting, error handling)
- **Polish**: T171-T202 (UX, performance, accessibility, testing, docs)

---

## Notes

- **Total Tasks**: 225 tasks (up from 139)
- **Added Tasks**: 86 new tasks (63 features + 1 CI/CD + 7 code quality + 14 accessibility + 1 testing)
- **Estimated Duration**: 5-7 additional weeks for full implementation of new features + a11y compliance

### Immediate Priorities (FIX BEFORE RELEASE)
- **Code Quality**:
  - T011a (CI/CD) - Set up immediately for continuous integration
  - T011b (CRITICAL) - Fix production unwrap() that can cause panics
  - T011c (CRITICAL) - Fix TypeScript any types that break type safety
  - T011d-e (HIGH) - Fix linting warnings before merging PRs
- **Accessibility (BLOCKING for WCAG AA)**:
  - T182 (CRITICAL) - Add form label associations (htmlFor + id)
  - T183 (CRITICAL) - Fix color-only information (add icons)
  - T184 (CRITICAL) - Add chart alternatives for screen readers
  - T185 (CRITICAL) - Add required field indicators
  - T186 (CRITICAL) - Add skip to main content link

### Project Standards
- **TDD Required**: All contract tests (T020-T064) must fail before implementation begins
- **Parallel Execution**: 95+ tasks marked [P] can run concurrently
- **Critical Security**: Tasks T084-T087 address SQL injection, rate limiting, and input validation
- **Test Coverage**: Backend 60%+ (T204), Frontend 70%+ (T205) per plan.md requirements
- **Constitution Compliance**: TDD enforced, substance over flash, ease of development prioritized
- **Accessibility**: WCAG AA compliance REQUIRED per FR-044 - See ACCESSIBILITY_AUDIT.md (Score: 5.5/10 - Needs Work)
- **Confirmation UX**: All destructive operations require confirmation with count (FR-050)
- **Code Quality**: See CODE_QUALITY_AUDIT.md for detailed analysis (Score: 7.6/10 - Good)
- **Best Practices**: Following GitHub Copilot Rust, TypeScript/React, and a11y guidelines
