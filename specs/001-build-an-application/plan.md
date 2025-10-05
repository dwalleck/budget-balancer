
# Implementation Plan: Budget Balancer - Debt Management & Spending Insights

**Branch**: `001-build-an-application` | **Date**: 2025-10-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/home/dwalleck/repos/budget-balancer/specs/001-build-an-application/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from file system structure or context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, `GEMINI.md` for Gemini CLI, `QWEN.md` for Qwen Code, or `AGENTS.md` for all other agents).
7. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 8. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Budget Balancer is a single-user desktop application for tracking spending and managing debt payoff strategies. Users upload CSV files containing transaction data, which the system automatically categorizes using predefined merchant keyword matching. The application provides comprehensive spending analysis with multiple visualization types (pie charts, bar graphs, line graphs, progress bars) across configurable time periods (monthly, quarterly, yearly, custom ranges). Core functionality includes debt payoff planning using avalanche (highest interest first) or snowball (smallest balance first) methodologies, with visual progress tracking against user-defined spending targets.

**Technical Approach**: Tauri 2 desktop application with React frontend, using Radix UI components styled with Tailwind CSS following the Opcode design pattern. State management via Zustand, local SQLite storage for financial data, and comprehensive test coverage using Vitest. The application prioritizes substance over flash with focus on functional UX and maintainability.

## Technical Context
**Language/Version**: TypeScript 5.x / React 18
**Primary Dependencies**: Tauri 2, React 18, Radix UI, Tailwind CSS, Vite, Zustand, Vitest
**Storage**: SQLite (via Tauri SQL plugin) for local data persistence with connection pooling
**Testing**: Vitest for unit/integration tests, Tauri test utilities for E2E, cargo-llvm-cov for coverage
**Target Platform**: Desktop (Windows, macOS, Linux via Tauri 2)
**Project Type**: Desktop application (Tauri + React SPA)
**Performance Goals**: <100ms UI response time, <500ms for CSV import/processing (files with 100-1000 rows), smooth 60fps visualizations
**Constraints**: Offline-capable, single-user local storage, OS-level security only
**Scale/Scope**: Personal finance management, ~10k transactions/year expected, <50MB data storage typical

## Non-Functional Requirements

### Security (See `SECURITY.md`)
- **SQL Injection Prevention**: All database queries MUST use parameterized statements
- **Input Validation**: All user input MUST be validated before processing
  - CSV file size limited to 10MB maximum
  - CSV row count limited to 10,000 rows
  - Numeric inputs validated for reasonable ranges
  - Text inputs sanitized and length-limited
- **Rate Limiting**: CSV imports throttled to minimum 2 seconds between uploads
- **Error Messages**: No internal details (file paths, database info, stack traces) exposed to users
- **Database Security**: Path validation to prevent directory traversal attacks

### Performance
- **Database**: Connection pooling with maximum 5 concurrent connections
- **Transaction Queries**: Paginated with 25 items per page (per spec FR-014)
- **CSV Import**: Stream processing for files >100 rows, progress reporting for >1000 rows
- **Async Operations**: All long-running tasks (imports, calculations) run asynchronously
- **Query Optimization**: Indexes on frequently queried columns (date, category_id, account_id)

### Code Quality
- **No Magic Numbers**: All constants extracted to named constants module
- **Consistent Error Handling**: Standardized error types across all modules
- **DRY Principles**: No duplicate code, shared utilities for common patterns
- **Testing**: TDD required for all new features
  - Backend: >60% line coverage minimum
  - Frontend: >70% coverage minimum
  - Critical paths: 100% coverage required

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Test-Driven Development Compliance (NON-NEGOTIABLE)
- [x] All features have test specifications defined (7 acceptance scenarios in spec)
- [x] Contract tests identified for all API interfaces (internal Tauri commands as contracts)
- [x] Integration tests planned for user scenarios (acceptance scenarios map to integration tests)
- [x] No untested code without leadership signoff documented (TDD workflow enforced)

### Development Philosophy Compliance
- [x] Design supports ease of development (Tauri + React with established patterns from Opcode reference)
- [x] Code maintainability considered in architecture (TypeScript, component-based React, clear separation of concerns)
- [x] Developer experience prioritized (Vite for fast builds, Vitest for testing, Zustand for simple state management)

### Substance Over Flash Compliance
- [x] User functionality prioritized over UI aesthetics (focus on data analysis, debt calculations, progress tracking)
- [x] Feature design focuses on usability and reliability (CSV import workflow, automatic categorization, clear visualizations)
- [x] User experience centered on application value (financial insights, debt payoff planning, spending control)

**Violations**: None

## Project Structure

### Documentation (this feature)
```
specs/001-build-an-application/
├── plan.md              # This file (/plan command output)
├── spec.md              # Feature specification
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
│   ├── accounts.md      # Account management contracts (4 commands)
│   ├── transactions.md  # Transaction management contracts (11 commands)
│   ├── categories.md    # Category management contracts (4 commands)
│   ├── category_rules.md # Categorization rules contracts (4 commands)
│   ├── column_mappings.md # CSV mapping contracts (5 commands)
│   ├── debts.md         # Debt management contracts (9 commands)
│   └── analytics.md     # Spending analysis contracts (7 commands)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
src-tauri/
├── src/
│   ├── commands/        # Tauri command handlers (backend logic)
│   ├── models/          # Data models and domain logic
│   ├── services/        # Business logic services
│   └── db/              # SQLite database layer
└── tests/
    ├── integration/     # Integration tests for Tauri commands
    └── unit/            # Unit tests for services/models

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

tests/
├── integration/        # Frontend integration tests
└── unit/               # Frontend unit tests
```

**Structure Decision**: Desktop application structure with Tauri backend (Rust) and React frontend (TypeScript). The `src-tauri/` directory contains Rust backend code with Tauri commands acting as the internal API layer. The `src/` directory contains React frontend code following component-based architecture. Tests are colocated with their respective codebases. This structure supports clear separation between system-level operations (file I/O, database) in Tauri and UI logic in React.

## Phase 0: Outline & Research

### Research Topics Identified
Based on Technical Context and user requirements, the following areas require research:

1. **Tauri 2 Best Practices**: Architecture patterns, command structure, state management between Rust/React
2. **Opcode Design Reference**: UI/UX patterns, component structure, styling approach from https://github.com/winfunc/opcode
3. **SQLite Schema Design**: Optimal schema for transactions, debts, categories with query performance
4. **Chart Libraries**: React chart library compatible with Radix UI/Tailwind (Recharts, Victory, or alternatives)
5. **CSV Parsing**: Robust CSV parsing in Tauri (Rust csv crate) with column mapping
6. **Debt Calculations**: Avalanche vs Snowball algorithm implementation, interest calculation accuracy
7. **Testing Strategy**: Vitest + Tauri test patterns, mocking Tauri commands in frontend tests

### Research Execution
Research will be consolidated in `research.md` with decisions, rationale, and alternatives for each topic.

**Output**: research.md with all architectural decisions documented

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

### Design Artifacts to Generate

1. **Data Model** (`data-model.md`):
   - SQLite schema for all 7 entities from spec
   - Relationships and foreign keys
   - Indexes for query performance
   - Validation rules and constraints

2. **Tauri Command Contracts** (`contracts/`):
   - Account commands (create, list, update, delete)
   - Transaction commands (import, list, search, categorize, update, delete, bulk operations)
   - Category commands (create, list, update, delete)
   - Category rules commands (create, list, update, delete)
   - Column mapping commands (save, list, get, update, delete)
   - Debt commands (create, list, update, delete, calculate payoff)
   - Analytics commands (spending by category, trends, progress, targets)
   - File operations (CSV import, data export)

3. **Integration Test Scenarios** (`quickstart.md`):
   - Map 7 acceptance scenarios to executable test steps
   - Test data setup requirements
   - Expected outcomes for each scenario

4. **Agent Context** (`CLAUDE.md`):
   - Technology stack summary
   - Project structure quick reference
   - Recent design decisions
   - Development workflow

**Output**: data-model.md, contracts/*.md, quickstart.md, CLAUDE.md

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

### Task Generation Strategy

**From Contracts** (25-30 tasks):
- Setup: Project initialization, dependencies, Tauri configuration
- Database: SQLite schema creation, migration system (using sqlx::migrate!() macro at app startup)
- **Account commands** (NEW): create_account, list_accounts, update_account, delete_account
- Transaction commands: import_csv, list_transactions, update_transaction_category, search_transactions, delete_transaction, bulk_delete_transactions, bulk_update_category
- **Category commands** (NEW): create_category, list_categories, update_category, delete_category
- **Category rules commands** (NEW): create_category_rule, list_category_rules, update_category_rule, delete_category_rule
- **Column mapping commands** (NEW): save_column_mapping, list_column_mappings, get_column_mapping, update_column_mapping, delete_column_mapping
- Debt commands: create_debt, list_debts, update_debt, delete_debt, calculate_avalanche, calculate_snowball
- Analytics commands: spending_by_category, spending_trends, debt_progress
- Each command gets: contract test [P] → implementation → unit tests [P]

**From Data Model** (5-8 tasks):
- Entity models in Rust (Transaction, Debt, Category, etc.) [P]
- Database access layer for each entity [P]
- Validation logic per entity [P]

**From User Stories** (7 integration tests):
- Acceptance scenario 1: CSV upload and categorization
- Acceptance scenario 2: Spending analysis with pie charts
- Acceptance scenario 3: Avalanche debt payoff plan
- Acceptance scenario 4: Snowball debt payoff plan
- Acceptance scenario 5: Spending targets progress
- Acceptance scenario 6: Debt progress visualizations
- Acceptance scenario 7: Transaction update triggers recalculation

**UI Implementation** (18-22 tasks):
- Base components: layout, navigation, theme
- **Account management UI** (NEW): account list, create/edit dialog, delete confirmation
- Transaction views: import, list with pagination, search bar, categorize, delete confirmation
- **Bulk operations UI** (NEW): checkbox selection, bulk action toolbar, bulk delete/update confirmation
- **Category management UI** (NEW): category list, create/edit dialog, delete with reassignment
- **Category rules UI** (NEW): rules management page, pattern editor, priority configuration
- **Column mappings UI** (NEW): mapping list, create/edit dialog, mapping selection in CSV import
- Debt views: add/edit, delete confirmation, payoff planner
- Analytics views: spending dashboard, progress tracking
- Visualizations: pie chart, bar chart, line chart, progress bar components
- **Confirmation dialogs** (NEW): generic confirmation component with count display

**Ordering Strategy**:
1. Setup & Database foundation
2. Backend: Tauri commands (TDD: tests → implementation)
3. Frontend: Components & pages (TDD: tests → implementation)
4. Integration: E2E test scenarios
5. Polish: Error handling, loading states, performance optimization

**Estimated Output**: 85-100 numbered, ordered tasks in tasks.md with [P] markers for parallel execution

**Added Scope** (from 2025-10-05 spec updates):
- Account CRUD operations (4 commands + UI)
- Category management (4 commands + UI)
- Category rules management (4 commands + UI)
- Column mappings management (4 commands + UI)
- Transaction enhancements (search, delete, bulk operations - 4 commands + UI)
- Debt delete operation (1 command + UI)
- Confirmation dialog system
- Pagination implementation (25/page)

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)
**Phase 4**: Implementation (execute tasks.md following constitutional principles)
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

No violations - section not applicable.

## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved (only FR-033 deferred as future scope)
- [x] Complexity deviations documented (none)

**Artifacts Generated**:
- [x] research.md - Architectural decisions and technology choices
- [x] data-model.md - SQLite schema with 8 tables and indexes
- [x] contracts/accounts.md - 4 Tauri commands for account management (2025-10-05)
- [x] contracts/transactions.md - 11 Tauri commands for transaction management (updated 2025-10-05)
- [x] contracts/categories.md - 4 Tauri commands for category management (2025-10-05)
- [x] contracts/category_rules.md - 4 Tauri commands for categorization rules (2025-10-05)
- [x] contracts/column_mappings.md - 5 Tauri commands for CSV mapping management (2025-10-05)
- [x] contracts/debts.md - 9 Tauri commands for debt operations (updated 2025-10-05)
- [x] contracts/analytics.md - 7 Tauri commands for spending analysis
- [x] quickstart.md - 7 integration test scenarios mapped from acceptance criteria
- [x] CLAUDE.md - Agent context file with tech stack and structure

---
*Based on Constitution v1.0.0 - See `.specify/memory/constitution.md`*
