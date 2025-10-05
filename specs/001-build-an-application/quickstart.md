# Quickstart & Integration Test Scenarios
**Budget Balancer - Feature Validation**

## Purpose
This document maps the 7 acceptance scenarios from the feature specification to executable integration tests. Each scenario validates end-to-end functionality from user action through Tauri commands to database and UI rendering.

## Test Setup

### Prerequisites
1. Clean SQLite database with schema initialized
2. Predefined categories and rules loaded
3. Sample CSV files in test fixtures
4. Mock Tauri command layer (for frontend tests)

### Test Data

**Sample Accounts**:
```typescript
const testAccounts = [
  { id: 1, name: 'Chase Checking', type: 'checking' },
  { id: 2, name: 'Chase Visa', type: 'credit_card' }
];
```

**Sample CSV (test-transactions.csv)**:
```csv
Date,Amount,Description,Merchant
2025-01-15,-45.50,STARBUCKS COFFEE,Starbucks
2025-01-16,-120.00,WHOLE FOODS MARKET,Whole Foods
2025-01-17,-25.00,SHELL GAS STATION,Shell
2025-01-20,-85.00,AMAZON.COM,Amazon
2025-01-25,-45.50,STARBUCKS COFFEE,Starbucks
```

**Sample Debts**:
```typescript
const testDebts = [
  { id: 1, name: 'Credit Card A', balance: 5000, interest_rate: 19.99, min_payment: 150 },
  { id: 2, name: 'Credit Card B', balance: 3000, interest_rate: 15.50, min_payment: 90 },
  { id: 3, name: 'Credit Card C', balance: 2000, interest_rate: 22.00, min_payment: 75 }
];
```

---

## Scenario 1: CSV Upload and Automatic Categorization

### User Story
**Given** a user has CSV files containing bank and credit card transactions
**When** they upload these files to the system
**Then** the system imports all transactions and categorizes them

### Test Steps

#### Step 1.1: Upload CSV without saved mapping
```typescript
test('should preview CSV when no mapping exists', async () => {
  const response = await invoke('import_csv', {
    file_path: './fixtures/test-transactions.csv',
    account_id: 1
  });

  // Verify preview returned
  expect(response.preview).toBeDefined();
  expect(response.preview.columns).toEqual(['Date', 'Amount', 'Description', 'Merchant']);
  expect(response.preview.sample_rows).toHaveLength(5);
});
```

#### Step 1.2: Save column mapping
```typescript
test('should save column mapping for future use', async () => {
  const response = await invoke('save_column_mapping', {
    source_name: 'Chase Export Format',
    date_col: 'Date',
    amount_col: 'Amount',
    description_col: 'Description',
    merchant_col: 'Merchant'
  });

  expect(response.mapping_id).toBeGreaterThan(0);
  mappingId = response.mapping_id; // Store for next step
});
```

#### Step 1.3: Import CSV with mapping
```typescript
test('should import transactions using saved mapping', async () => {
  const response = await invoke('import_csv', {
    file_path: './fixtures/test-transactions.csv',
    mapping_id: mappingId,
    account_id: 1
  });

  expect(response.success).toBe(true);
  expect(response.imported_count).toBe(5);
  expect(response.skipped_count).toBe(0);
});
```

#### Step 1.4: Verify auto-categorization
```typescript
test('should auto-categorize transactions', async () => {
  const response = await invoke('list_transactions', {
    account_id: 1
  });

  const starbucksTransaction = response.transactions.find(t =>
    t.merchant.includes('STARBUCKS')
  );
  expect(starbucksTransaction.category_id).toBe(DINING_CATEGORY_ID);

  const wholeFoodsTransaction = response.transactions.find(t =>
    t.merchant.includes('WHOLE FOODS')
  );
  expect(wholeFoodsTransaction.category_id).toBe(GROCERIES_CATEGORY_ID);
});
```

#### Step 1.5: UI Verification
```typescript
test('UI should display imported transactions', async () => {
  render(<TransactionListPage />);

  await waitFor(() => {
    expect(screen.getByText('STARBUCKS COFFEE')).toBeInTheDocument();
    expect(screen.getByText('$45.50')).toBeInTheDocument();
    expect(screen.getByText('Dining')).toBeInTheDocument(); // Category badge
  });
});
```

**Expected Outcome**: ✅ All transactions imported, automatically categorized using merchant keywords, displayed in UI

---

## Scenario 2: Spending Analysis with Pie Chart

### User Story
**Given** a user has imported their transactions
**When** they view their spending analysis
**Then** they see a breakdown of spending by category with totals, percentages, and pie chart visualization

### Test Steps

#### Step 2.1: Calculate spending by category
```typescript
test('should calculate spending breakdown', async () => {
  const response = await invoke('get_spending_by_category', {
    start_date: '2025-01-01',
    end_date: '2025-01-31',
    account_id: 1
  });

  expect(response.total_spending).toBe(321.00); // Sum of all expenses
  expect(response.categories).toHaveLength(4); // Dining, Groceries, Transportation, Shopping

  const diningCategory = response.categories.find(c => c.category_name === 'Dining');
  expect(diningCategory.amount).toBe(91.00); // 2x $45.50
  expect(diningCategory.percentage).toBeCloseTo(28.3, 1); // 91/321 * 100
});
```

#### Step 2.2: UI Pie Chart Rendering
```typescript
test('UI should display pie chart with spending data', async () => {
  render(<SpendingAnalysisPage />);

  // Select date range
  await userEvent.selectOptions(screen.getByLabelText('Time Period'), 'Monthly');

  await waitFor(() => {
    // Verify Recharts PieChart component rendered
    const pieChart = screen.getByTestId('spending-pie-chart');
    expect(pieChart).toBeInTheDocument();

    // Verify category totals displayed
    expect(screen.getByText('Dining: $91.00 (28.3%)')).toBeInTheDocument();
    expect(screen.getByText('Groceries: $120.00 (37.4%)')).toBeInTheDocument();
  });
});
```

**Expected Outcome**: ✅ Spending breakdown calculated correctly, pie chart displays proportional slices, percentages sum to 100%

---

## Scenario 3: Avalanche Debt Payoff Plan

### User Story
**Given** a user has multiple debts with different balances and interest rates
**When** they create a debt payoff plan using the avalanche method
**Then** the system generates a payment schedule prioritizing debts by highest interest rate first

### Test Steps

#### Step 3.1: Calculate avalanche plan
```typescript
test('should calculate avalanche debt payoff', async () => {
  const response = await invoke('calculate_payoff_plan', {
    strategy: 'avalanche',
    monthly_amount: 500.00
  });

  expect(response.strategy).toBe('avalanche');
  expect(response.plan_id).toBeGreaterThan(0);

  // Verify highest interest debt (Credit Card C: 22%) gets extra payment first
  const month1 = response.monthly_breakdown[0];
  const cardCPayment = month1.payments.find(p => p.debt_name === 'Credit Card C');
  const cardAPayment = month1.payments.find(p => p.debt_name === 'Credit Card A');
  const cardBPayment = month1.payments.find(p => p.debt_name === 'Credit Card B');

  // Card C (highest rate) should get: 500 - (150 + 90) = 260 extra + 75 minimum = 335
  expect(cardCPayment.amount).toBeCloseTo(335, 1);
  expect(cardAPayment.amount).toBe(150); // Minimum only
  expect(cardBPayment.amount).toBe(90);  // Minimum only
});
```

#### Step 3.2: Verify payoff order
```typescript
test('should pay off debts in interest rate order', async () => {
  const response = await invoke('calculate_payoff_plan', {
    strategy: 'avalanche',
    monthly_amount: 500.00
  });

  const summaries = response.debt_summaries.sort((a, b) => a.payoff_month - b.payoff_month);

  // Credit Card C (22%) should pay off first, then A (19.99%), then B (15.50%)
  expect(summaries[0].debt_name).toBe('Credit Card C');
  expect(summaries[1].debt_name).toBe('Credit Card A');
  expect(summaries[2].debt_name).toBe('Credit Card B');
});
```

#### Step 3.3: UI Display
```typescript
test('UI should display avalanche payment schedule', async () => {
  render(<DebtPayoffPlannerPage />);

  await userEvent.selectOptions(screen.getByLabelText('Strategy'), 'avalanche');
  await userEvent.type(screen.getByLabelText('Monthly Amount'), '500');
  await userEvent.click(screen.getByText('Calculate Plan'));

  await waitFor(() => {
    expect(screen.getByText(/Payoff Date:/)).toHaveTextContent(/2027-/); // Approximate
    expect(screen.getByText(/Total Interest:/)).toHaveTextContent(/\$1,5/); // Approximate

    // Verify payment schedule table
    const scheduleRows = screen.getAllByRole('row');
    expect(scheduleRows.length).toBeGreaterThan(10); // Multiple months
  });
});
```

**Expected Outcome**: ✅ Avalanche plan prioritizes highest interest debt, calculates payoff date and total interest, UI displays schedule

---

## Scenario 4: Snowball Debt Payoff Plan

### User Story
**Given** a user has multiple debts with different balances and interest rates
**When** they create a debt payoff plan using the snowball method
**Then** the system generates a payment schedule prioritizing debts by smallest balance first

### Test Steps

#### Step 4.1: Calculate snowball plan
```typescript
test('should calculate snowball debt payoff', async () => {
  const response = await invoke('calculate_payoff_plan', {
    strategy: 'snowball',
    monthly_amount: 500.00
  });

  expect(response.strategy).toBe('snowball');

  // Verify lowest balance debt (Credit Card C: $2000) gets extra payment first
  const month1 = response.monthly_breakdown[0];
  const cardCPayment = month1.payments.find(p => p.debt_name === 'Credit Card C');

  expect(cardCPayment.amount).toBeCloseTo(335, 1); // 500 - (150 + 90) + 75 min
});
```

#### Step 4.2: Verify payoff order
```typescript
test('should pay off debts in balance order', async () => {
  const response = await invoke('calculate_payoff_plan', {
    strategy: 'snowball',
    monthly_amount: 500.00
  });

  const summaries = response.debt_summaries.sort((a, b) => a.payoff_month - b.payoff_month);

  // Credit Card C ($2000) should pay off first, then B ($3000), then A ($5000)
  expect(summaries[0].debt_name).toBe('Credit Card C');
  expect(summaries[1].debt_name).toBe('Credit Card B');
  expect(summaries[2].debt_name).toBe('Credit Card A');
});
```

#### Step 4.3: Compare strategies
```typescript
test('should show strategy comparison', async () => {
  const response = await invoke('compare_strategies', {
    monthly_amount: 500.00
  });

  expect(response.avalanche.total_interest).toBeLessThan(response.snowball.total_interest);
  expect(response.savings.interest_saved).toBeGreaterThan(0);
  expect(response.savings.months_saved).toBeGreaterThanOrEqual(0);
});
```

**Expected Outcome**: ✅ Snowball plan prioritizes smallest balance, shows strategy comparison with interest savings

---

## Scenario 5: Spending Targets Progress

### User Story
**Given** a user has set spending targets for specific categories
**When** they view their progress
**Then** they see whether they are under, at, or over their targets with progress bar indicators

### Test Steps

#### Step 5.1: Create spending targets
```typescript
test('should create spending targets', async () => {
  const diningTarget = await invoke('create_spending_target', {
    category_id: DINING_CATEGORY_ID,
    amount: 100.00,
    period: 'monthly',
    start_date: '2025-01-01'
  });

  const groceriesTarget = await invoke('create_spending_target', {
    category_id: GROCERIES_CATEGORY_ID,
    amount: 150.00,
    period: 'monthly',
    start_date: '2025-01-01'
  });

  expect(diningTarget.target_id).toBeGreaterThan(0);
  expect(groceriesTarget.target_id).toBeGreaterThan(0);
});
```

#### Step 5.2: Check progress
```typescript
test('should calculate target progress', async () => {
  const response = await invoke('get_spending_targets_progress', {
    period: 'monthly'
  });

  const diningProgress = response.targets.find(t => t.category_name === 'Dining');
  expect(diningProgress.target_amount).toBe(100.00);
  expect(diningProgress.actual_amount).toBe(91.00); // From scenario 2
  expect(diningProgress.percentage_used).toBeCloseTo(91, 1);
  expect(diningProgress.status).toBe('under');
  expect(diningProgress.remaining).toBe(9.00);

  const groceriesProgress = response.targets.find(t => t.category_name === 'Groceries');
  expect(groceriesProgress.actual_amount).toBe(120.00);
  expect(groceriesProgress.status).toBe('under');
});
```

#### Step 5.3: UI Progress Bars
```typescript
test('UI should display progress bars for targets', async () => {
  render(<SpendingTargetsPage />);

  await waitFor(() => {
    // Verify progress bars
    const diningProgressBar = screen.getByTestId('progress-bar-dining');
    expect(diningProgressBar).toHaveAttribute('aria-valuenow', '91');
    expect(diningProgressBar).toHaveAttribute('aria-valuemax', '100');

    // Verify status indicators
    expect(screen.getByText(/Under budget/)).toBeInTheDocument();
    expect(screen.getByText(/\$9.00 remaining/)).toBeInTheDocument();
  });
});
```

**Expected Outcome**: ✅ Targets tracked per category, progress calculated accurately, UI shows progress bars with status indicators

---

## Scenario 6: Debt Progress Visualizations

### User Story
**Given** a user has a debt payoff plan and has made payments
**When** they view their debt progress
**Then** they see visualizations including progress bars for remaining debt, bar graphs for payments made, and line graphs showing payoff trajectory

### Test Steps

#### Step 6.1: Record payments
```typescript
test('should record debt payments', async () => {
  const payment1 = await invoke('record_debt_payment', {
    debt_id: 1,
    amount: 500.00,
    date: '2025-01-31',
    plan_id: 1
  });

  const payment2 = await invoke('record_debt_payment', {
    debt_id: 1,
    amount: 500.00,
    date: '2025-02-28',
    plan_id: 1
  });

  expect(payment1.updated_balance).toBeLessThan(5000);
  expect(payment2.updated_balance).toBeLessThan(payment1.updated_balance);
});
```

#### Step 6.2: Get debt progress
```typescript
test('should retrieve debt progress with visualizations data', async () => {
  const response = await invoke('get_debt_progress', {
    debt_id: 1
  });

  expect(response.payments).toHaveLength(2);
  expect(response.total_paid).toBe(1000.00);
  expect(response.balance_history).toBeInstanceOf(Array);
  expect(response.balance_history[0].balance).toBe(5000);
  expect(response.balance_history[1].balance).toBeLessThan(5000);
});
```

#### Step 6.3: UI Visualizations
```typescript
test('UI should display debt progress visualizations', async () => {
  render(<DebtProgressPage debtId={1} />);

  await waitFor(() => {
    // Progress bar
    const progressBar = screen.getByTestId('debt-progress-bar');
    expect(progressBar).toHaveAttribute('aria-valuenow', '20'); // ~1000/5000 = 20%

    // Bar graph for payments
    expect(screen.getByTestId('payments-bar-chart')).toBeInTheDocument();

    // Line graph for balance trajectory
    expect(screen.getByTestId('balance-line-chart')).toBeInTheDocument();

    // Status indicator
    expect(screen.getByText(/On Track/)).toBeInTheDocument();
  });
});
```

**Expected Outcome**: ✅ Debt progress tracked with payment history, multiple visualization types display correctly

---

## Scenario 7: Transaction Update Triggers Recalculation

### User Story
**Given** a user uploads new transaction data
**When** the system processes it
**Then** spending category totals and debt progress are automatically updated

### Test Steps

#### Step 7.1: Upload additional transactions
```typescript
test('should import new transactions', async () => {
  const response = await invoke('import_csv', {
    file_path: './fixtures/february-transactions.csv',
    mapping_id: mappingId,
    account_id: 1
  });

  expect(response.imported_count).toBeGreaterThan(0);
});
```

#### Step 7.2: Verify spending recalculation
```typescript
test('should update spending analysis with new data', async () => {
  const response = await invoke('get_spending_by_category', {
    start_date: '2025-02-01',
    end_date: '2025-02-28'
  });

  expect(response.total_spending).toBeGreaterThan(0);
  expect(response.categories.length).toBeGreaterThan(0);
});
```

#### Step 7.3: Verify target progress update
```typescript
test('should update target progress with new spending', async () => {
  const response = await invoke('get_spending_targets_progress', {
    custom_start: '2025-02-01',
    custom_end: '2025-02-28'
  });

  // Targets should reflect new spending from February
  response.targets.forEach(target => {
    expect(target.actual_amount).toBeGreaterThanOrEqual(0);
  });
});
```

#### Step 7.4: UI Auto-refresh
```typescript
test('UI should reflect updated data', async () => {
  render(<DashboardPage />);

  // Simulate new CSV import
  const importButton = screen.getByText('Import Transactions');
  await userEvent.click(importButton);
  // ... upload flow

  await waitFor(() => {
    // Dashboard should show updated totals
    expect(screen.getByTestId('total-spending')).toHaveTextContent(/\$\d+/);
    expect(screen.getByTestId('spending-chart')).toBeInTheDocument();
  });
});
```

**Expected Outcome**: ✅ New transactions automatically update all analytics, charts refresh with new data

---

## Test Execution

### Run All Scenarios
```bash
bun run test:integration
```

### Run Individual Scenario
```bash
bun run test:integration --grep "Scenario 1"
```

### Coverage Report
```bash
bun run test:coverage
```

**Target**: 100% integration test coverage of acceptance scenarios

---

## Success Criteria

All 7 scenarios must pass with:
- ✅ All Tauri commands return expected data
- ✅ Database state updated correctly
- ✅ UI components render with correct data
- ✅ Visualizations display accurately
- ✅ User workflows complete without errors
- ✅ Performance targets met (<500ms for imports, <100ms for UI updates)
