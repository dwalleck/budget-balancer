
# In-Depth Bug Review: Budget Balancer

This document provides a more detailed analysis of potential bugs and issues in the Budget Balancer application.

## Frontend (`src/`)

### 1. State Management and Data Fetching

*   **Inefficient State Updates in `transactionStore.ts`**
    *   **Issue:** The `updateCategory` function refetches all transactions after updating a single one. This is inefficient and will cause performance problems as the number of transactions grows.
    *   **Recommendation:** Instead of refetching, update the specific transaction in the local Zustand state. This will be much faster and provide a better user experience.

*   **Missing `useEffect` Dependencies**
    *   **Issue:** Several components have `useEffect` hooks with missing dependencies. For example, in `src/pages/TrendsPage.tsx`, the `useEffect` hook is missing `fetchTrends` in its dependency array. This can lead to stale data and unexpected behavior.
    *   **Recommendation:** Add the missing dependencies to the `useEffect` dependency arrays in all components.

*   **Potential Race Conditions in Zustand Stores**
    *   **Issue:** Async actions in the Zustand stores that modify the same state could potentially lead to race conditions. For example, if `fetchAccounts` is called multiple times in quick succession, it could lead to inconsistent state.
    *   **Recommendation:** Implement a mechanism to prevent race conditions, such as disabling the fetch button while a fetch is in progress, or using a library like `zustand-middleware-immer` to ensure atomic state updates.

### 2. Component Logic and UI

*   **Uncontrolled Components in `DebtPlannerPage.tsx`**
    *   **Issue:** The `monthlyAmount` is handled as a string in the component's state and parsed to a float only when needed. This can lead to issues with validation and data consistency.
    *   **Recommendation:** Store `monthlyAmount` as a number in the component's state and handle the conversion in the `onChange` handler.

*   **Hardcoded Styles in `Select.tsx`**
    *   **Issue:** The `SelectContent` component has hardcoded dark mode styles. This will not work well with the light theme and goes against the principles of using a theming solution like Tailwind CSS.
    *   **Recommendation:** Remove the hardcoded styles and use Tailwind's `dark:` variant to apply dark mode styles.

*   **Type Safety in `AccountCreationDialog.tsx`**
    *   **Issue:** In `AccountCreationDialog.tsx`, the `onValueChange` handler for the `Select` component has a parameter of type `any`.
    *   **Recommendation:** Replace `any` with the specific type `'checking' | 'savings' | 'credit_card'` to improve type safety.

## Backend (`src-tauri/`)

### 1. SQL Injection Vulnerabilities

*   **`list_transactions` in `transaction_commands.rs`**
    *   **Issue:** The `list_transactions` function constructs an SQL query by concatenating strings. This is a serious security vulnerability that could lead to SQL injection attacks.
    *   **Recommendation:** Use `sqlx::query_as!` with placeholders (`?`) to safely bind the filter parameters to the query.

*   **`update_debt` in `debt_commands.rs`**
    *   **Issue:** The `update_debt` function dynamically constructs the `UPDATE` query, which is also vulnerable to SQL injection.
    *   **Recommendation:** Use a different approach to build the query, such as using a query builder or multiple queries, to avoid SQL injection.

### 2. Database Performance and Correctness

*   **N+1 Query Problem**
    *   **Issue:** In `src-tauri/src/commands/transaction_commands.rs`, the `export_transactions` function fetches the category name for each transaction inside a loop. A similar issue exists in `src-tauri/src/services/trends_calculator.rs` in the `get_monthly_trends` function.
    *   **Recommendation:** Use a `JOIN` in the initial query to fetch the category name along with the transaction data. This will significantly improve performance.

*   **Inefficient Connection Management**
    *   **Issue:** The `get_db` function is called in every command, which creates a new connection pool for each call. This is inefficient.
    *   **Recommendation:** Create the connection pool once in the `main` function and share it as a managed state with the commands.

*   **Lack of Database Transactions**
    *   **Issue:** In `src-tauri/src/commands/debt_commands.rs`, the `record_debt_payment` function first records a payment and then updates the debt balance in two separate queries. This should be done within a single database transaction to ensure atomicity.
    *   **Recommendation:** Use a database transaction to wrap the two queries. This will ensure that both queries succeed or fail together.

### 3. Code Quality and Consistency

*   **Inconsistent Database Libraries**
    *   **Issue:** The `src-tauri/src/db/mod.rs` file indicates that the project uses both `rusqlite` and `sqlx`. However, the repository files using `rusqlite` are not used in the commands.
    *   **Recommendation:** Remove the unused `rusqlite` repository files and use `sqlx` consistently throughout the application.

*   **Inconsistent Error Handling**
    *   **Issue:** The error handling is inconsistent across the backend. Some functions return `Result<T, String>`, while others use custom error enums.
    *   **Recommendation:** Use a consistent error handling strategy throughout the application. Using a custom error enum with `thiserror` is a good practice in Rust.

*   **Hardcoded "Magic Numbers"**
    *   **Issue:** In `src-tauri/src/services/categorizer.rs`, the "Uncategorized" category is hardcoded with an ID of `10`.
    *   **Recommendation:** Query the database for the "Uncategorized" category by name and use its ID instead of hardcoding it.
