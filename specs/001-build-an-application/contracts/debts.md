# Debt Management Contracts
**Tauri Commands for Debt Operations**

## Command: `create_debt`
Add a new debt account.

### Request
```typescript
interface CreateDebtRequest {
  name: string;
  balance: number;
  interest_rate: number;       // Annual percentage (e.g., 18.5 for 18.5%)
  min_payment: number;
}
```

### Response
```typescript
interface CreateDebtResponse {
  debt_id: number;
}
```

### Errors
- `InvalidAmount`: balance or min_payment is negative
- `InvalidRate`: interest_rate outside 0-100 range

### Contract Test
```typescript
it('should create debt', async () => {
  const response = await invoke('create_debt', {
    name: 'Credit Card A',
    balance: 5000.00,
    interest_rate: 18.5,
    min_payment: 150.00
  });

  expect(response.debt_id).toBeGreaterThan(0);
});

it('should reject invalid interest rate', async () => {
  await expect(invoke('create_debt', {
    name: 'Test',
    balance: 1000,
    interest_rate: 150,  // Invalid
    min_payment: 50
  })).rejects.toThrow('InvalidRate');
});
```

---

## Command: `list_debts`
Retrieve all debt accounts.

### Request
```typescript
// No parameters
```

### Response
```typescript
interface ListDebtsResponse {
  debts: Debt[];
}

interface Debt {
  id: number;
  name: string;
  balance: number;
  original_balance: number;
  interest_rate: number;
  min_payment: number;
  created_at: string;
  updated_at: string;
}
```

### Contract Test
```typescript
it('should list all debts', async () => {
  const response = await invoke('list_debts');

  expect(response.debts).toBeInstanceOf(Array);
  expect(response.debts[0]).toHaveProperty('name');
  expect(response.debts[0]).toHaveProperty('balance');
});
```

---

## Command: `update_debt`
Update debt information.

### Request
```typescript
interface UpdateDebtRequest {
  debt_id: number;
  balance?: number;
  interest_rate?: number;
  min_payment?: number;
}
```

### Response
```typescript
interface UpdateDebtResponse {
  success: boolean;
}
```

### Errors
- `DebtNotFound`: debt_id doesn't exist
- `InvalidAmount`: balance or min_payment is negative
- `InvalidRate`: interest_rate outside 0-100 range

### Contract Test
```typescript
it('should update debt balance', async () => {
  const response = await invoke('update_debt', {
    debt_id: 1,
    balance: 4500.00
  });

  expect(response.success).toBe(true);
});
```

---

## Command: `calculate_payoff_plan`
Calculate debt payoff schedule using avalanche or snowball strategy.

### Request
```typescript
interface CalculatePayoffPlanRequest {
  strategy: 'avalanche' | 'snowball';
  monthly_amount: number;      // Total available for debt payments
}
```

### Response
```typescript
interface CalculatePayoffPlanResponse {
  plan_id: number;             // Saved plan ID
  strategy: string;
  payoff_date: string;         // ISO 8601 date when all debts paid
  total_interest: number;
  monthly_breakdown: MonthlyPayment[];
  debt_summaries: DebtSummary[];
}

interface MonthlyPayment {
  month: number;               // 1, 2, 3, ...
  date: string;                // ISO 8601
  payments: {
    debt_id: number;
    debt_name: string;
    amount: number;
  }[];
  total_balance_remaining: number;
}

interface DebtSummary {
  debt_id: number;
  debt_name: string;
  payoff_month: number;
  total_interest_paid: number;
}
```

### Errors
- `InsufficientFunds`: monthly_amount less than sum of min_payments
- `NoDebts`: No debts in database

### Contract Test
```typescript
it('should calculate avalanche payoff plan', async () => {
  const response = await invoke('calculate_payoff_plan', {
    strategy: 'avalanche',
    monthly_amount: 1000.00
  });

  expect(response.strategy).toBe('avalanche');
  expect(response.payoff_date).toBeDefined();
  expect(response.monthly_breakdown).toBeInstanceOf(Array);

  // Verify highest interest debt gets extra payment
  const firstMonth = response.monthly_breakdown[0];
  const debtPayments = firstMonth.payments.sort((a, b) => b.amount - a.amount);
  // Highest payment should go to highest interest rate debt
});

it('should calculate snowball payoff plan', async () => {
  const response = await invoke('calculate_payoff_plan', {
    strategy: 'snowball',
    monthly_amount: 1000.00
  });

  expect(response.strategy).toBe('snowball');

  // Verify lowest balance debt gets extra payment
  const firstMonth = response.monthly_breakdown[0];
  const debtPayments = firstMonth.payments.sort((a, b) => b.amount - a.amount);
  // Highest payment should go to lowest balance debt
});

it('should reject insufficient monthly amount', async () => {
  await expect(invoke('calculate_payoff_plan', {
    strategy: 'avalanche',
    monthly_amount: 50.00  // Less than minimum payments
  })).rejects.toThrow('InsufficientFunds');
});
```

---

## Command: `get_payoff_plan`
Retrieve a saved payoff plan.

### Request
```typescript
interface GetPayoffPlanRequest {
  plan_id: number;
}
```

### Response
```typescript
// Same as CalculatePayoffPlanResponse
```

### Errors
- `PlanNotFound`: plan_id doesn't exist

### Contract Test
```typescript
it('should retrieve saved payoff plan', async () => {
  const response = await invoke('get_payoff_plan', {
    plan_id: 1
  });

  expect(response.plan_id).toBe(1);
  expect(response.monthly_breakdown).toBeDefined();
});
```

---

## Command: `record_debt_payment`
Manually record a payment toward a debt.

### Request
```typescript
interface RecordDebtPaymentRequest {
  debt_id: number;
  amount: number;
  date: string;                // ISO 8601
  plan_id?: number;            // Optional: Link to plan
}
```

### Response
```typescript
interface RecordDebtPaymentResponse {
  payment_id: number;
  updated_balance: number;     // New debt balance after payment
}
```

### Errors
- `DebtNotFound`: debt_id doesn't exist
- `InvalidAmount`: amount <= 0 or > debt balance
- `InvalidDate`: Future date not allowed

### Contract Test
```typescript
it('should record debt payment and update balance', async () => {
  const response = await invoke('record_debt_payment', {
    debt_id: 1,
    amount: 500.00,
    date: '2025-10-01'
  });

  expect(response.payment_id).toBeGreaterThan(0);
  expect(response.updated_balance).toBeLessThan(/* original balance */);
});

it('should reject payment exceeding balance', async () => {
  await expect(invoke('record_debt_payment', {
    debt_id: 1,
    amount: 999999.00,
    date: '2025-10-01'
  })).rejects.toThrow('InvalidAmount');
});
```

---

## Command: `get_debt_progress`
Get payment history and progress for a debt.

### Request
```typescript
interface GetDebtProgressRequest {
  debt_id: number;
  start_date?: string;
  end_date?: string;
}
```

### Response
```typescript
interface GetDebtProgressResponse {
  debt: Debt;
  payments: DebtPayment[];
  total_paid: number;
  balance_history: BalancePoint[];
}

interface DebtPayment {
  id: number;
  amount: number;
  date: string;
  plan_id: number | null;
}

interface BalancePoint {
  date: string;
  balance: number;
}
```

### Contract Test
```typescript
it('should get debt progress with payment history', async () => {
  const response = await invoke('get_debt_progress', {
    debt_id: 1
  });

  expect(response.debt).toBeDefined();
  expect(response.payments).toBeInstanceOf(Array);
  expect(response.total_paid).toBeGreaterThanOrEqual(0);
  expect(response.balance_history).toBeInstanceOf(Array);
});
```

---

## Command: `compare_strategies`
Compare avalanche vs snowball for current debts.

### Request
```typescript
interface CompareStrategiesRequest {
  monthly_amount: number;
}
```

### Response
```typescript
interface CompareStrategiesResponse {
  avalanche: StrategyComparison;
  snowball: StrategyComparison;
  savings: {
    interest_saved: number;     // Avalanche saves this much vs Snowball
    months_saved: number;        // Avalanche finishes this many months earlier
  };
}

interface StrategyComparison {
  strategy: string;
  payoff_date: string;
  total_interest: number;
  payoff_months: number;
}
```

### Contract Test
```typescript
it('should compare avalanche vs snowball strategies', async () => {
  const response = await invoke('compare_strategies', {
    monthly_amount: 1000.00
  });

  expect(response.avalanche).toBeDefined();
  expect(response.snowball).toBeDefined();
  expect(response.savings).toBeDefined();

  // Avalanche should typically save interest
  expect(response.avalanche.total_interest).toBeLessThanOrEqual(
    response.snowball.total_interest
  );
});
```
