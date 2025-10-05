# Spending Analysis Contracts
**Tauri Commands for Analytics and Visualization**

## Command: `get_spending_by_category`
Calculate total spending per category for a time period.

### Request
```typescript
interface GetSpendingByCategoryRequest {
  start_date: string;          // ISO 8601
  end_date: string;
  account_id?: number;         // Optional filter
}
```

### Response
```typescript
interface GetSpendingByCategoryResponse {
  period: {
    start_date: string;
    end_date: string;
  };
  categories: CategorySpending[];
  total_spending: number;
}

interface CategorySpending {
  category_id: number;
  category_name: string;
  category_icon: string;
  amount: number;              // Total spent in this category
  percentage: number;          // Percentage of total_spending
  transaction_count: number;
}
```

### Errors
- `InvalidDateRange`: end_date before start_date

### Contract Test
```typescript
it('should calculate spending by category', async () => {
  const response = await invoke('get_spending_by_category', {
    start_date: '2025-01-01',
    end_date: '2025-01-31'
  });

  expect(response.categories).toBeInstanceOf(Array);
  expect(response.total_spending).toBeGreaterThanOrEqual(0);

  // Verify percentages sum to ~100
  const totalPercentage = response.categories.reduce((sum, c) => sum + c.percentage, 0);
  expect(totalPercentage).toBeCloseTo(100, 1);
});

it('should return empty for date range with no transactions', async () => {
  const response = await invoke('get_spending_by_category', {
    start_date: '2020-01-01',
    end_date: '2020-01-31'
  });

  expect(response.categories).toHaveLength(0);
  expect(response.total_spending).toBe(0);
});
```

---

## Command: `get_spending_trends`
Get spending trends over time (for line charts).

### Request
```typescript
interface GetSpendingTrendsRequest {
  start_date: string;
  end_date: string;
  interval: 'daily' | 'weekly' | 'monthly';
  category_id?: number;        // Optional: Trends for specific category
}
```

### Response
```typescript
interface GetSpendingTrendsResponse {
  data_points: TrendPoint[];
  total_spending: number;
  average_per_interval: number;
}

interface TrendPoint {
  date: string;                // ISO 8601 (start of interval)
  amount: number;
  transaction_count: number;
}
```

### Contract Test
```typescript
it('should get monthly spending trends', async () => {
  const response = await invoke('get_spending_trends', {
    start_date: '2025-01-01',
    end_date: '2025-12-31',
    interval: 'monthly'
  });

  expect(response.data_points).toHaveLength(12);  // 12 months
  expect(response.average_per_interval).toBeGreaterThan(0);
});

it('should get trends for specific category', async () => {
  const response = await invoke('get_spending_trends', {
    start_date: '2025-01-01',
    end_date: '2025-12-31',
    interval: 'monthly',
    category_id: 1  // e.g., Groceries
  });

  expect(response.data_points).toBeDefined();
  // All amounts should be from specified category
});
```

---

## Command: `get_spending_targets_progress`
Check progress against spending targets.

### Request
```typescript
interface GetSpendingTargetsProgressRequest {
  period?: 'monthly' | 'quarterly' | 'yearly';  // Default: current month
  custom_start?: string;       // For custom date range
  custom_end?: string;
}
```

### Response
```typescript
interface GetSpendingTargetsProgressResponse {
  period: {
    start_date: string;
    end_date: string;
  };
  targets: TargetProgress[];
  overall_status: 'under' | 'on_track' | 'over';
}

interface TargetProgress {
  category_id: number;
  category_name: string;
  target_amount: number;
  actual_amount: number;
  remaining: number;           // Negative if over budget
  percentage_used: number;     // e.g., 75.5 for 75.5%
  status: 'under' | 'on_track' | 'over';
  variance: number;            // actual - target (negative is good)
}
```

### Errors
- `InvalidPeriod`: Invalid period or custom dates

### Contract Test
```typescript
it('should get spending targets progress', async () => {
  const response = await invoke('get_spending_targets_progress', {
    period: 'monthly'
  });

  expect(response.targets).toBeInstanceOf(Array);
  expect(response.overall_status).toMatch(/under|on_track|over/);

  response.targets.forEach(target => {
    expect(target.percentage_used).toBeGreaterThanOrEqual(0);
    expect(target.status).toMatch(/under|on_track|over/);
  });
});

it('should show "over" status when spending exceeds target', async () => {
  // Assume setup: Target $500, spent $600
  const response = await invoke('get_spending_targets_progress');

  const overTarget = response.targets.find(t => t.actual_amount > t.target_amount);
  expect(overTarget.status).toBe('over');
  expect(overTarget.remaining).toBeLessThan(0);
});
```

---

## Command: `create_spending_target`
Set a spending target for a category.

### Request
```typescript
interface CreateSpendingTargetRequest {
  category_id: number;
  amount: number;
  period: 'monthly' | 'quarterly' | 'yearly';
  start_date: string;          // ISO 8601
  end_date?: string;           // Optional: NULL for recurring
}
```

### Response
```typescript
interface CreateSpendingTargetResponse {
  target_id: number;
}
```

### Errors
- `CategoryNotFound`: category_id doesn't exist
- `InvalidAmount`: amount <= 0
- `DuplicateTarget`: Target already exists for this category + period

### Contract Test
```typescript
it('should create spending target', async () => {
  const response = await invoke('create_spending_target', {
    category_id: 1,
    amount: 500.00,
    period: 'monthly',
    start_date: '2025-01-01'
  });

  expect(response.target_id).toBeGreaterThan(0);
});

it('should reject duplicate target', async () => {
  await invoke('create_spending_target', {
    category_id: 1,
    amount: 500.00,
    period: 'monthly',
    start_date: '2025-01-01'
  });

  await expect(invoke('create_spending_target', {
    category_id: 1,
    amount: 600.00,
    period: 'monthly',
    start_date: '2025-01-01'
  })).rejects.toThrow('DuplicateTarget');
});
```

---

## Command: `update_spending_target`
Update an existing spending target.

### Request
```typescript
interface UpdateSpendingTargetRequest {
  target_id: number;
  amount?: number;
  end_date?: string;
}
```

### Response
```typescript
interface UpdateSpendingTargetResponse {
  success: boolean;
}
```

### Errors
- `TargetNotFound`: target_id doesn't exist
- `InvalidAmount`: amount <= 0

### Contract Test
```typescript
it('should update spending target amount', async () => {
  const response = await invoke('update_spending_target', {
    target_id: 1,
    amount: 600.00
  });

  expect(response.success).toBe(true);
});
```

---

## Command: `get_dashboard_summary`
Get overview data for main dashboard.

### Request
```typescript
interface GetDashboardSummaryRequest {
  period: 'current_month' | 'last_30_days' | 'current_year';
}
```

### Response
```typescript
interface GetDashboardSummaryResponse {
  period: {
    start_date: string;
    end_date: string;
  };
  total_spending: number;
  total_income: number;
  net: number;                 // income - spending
  top_categories: CategorySpending[];  // Top 5 by amount
  debt_summary: {
    total_debt: number;
    total_monthly_payment: number;
    next_payoff_date: string | null;  // From active plan
  };
  target_summary: {
    on_track_count: number;
    over_count: number;
    total_variance: number;
  };
}
```

### Contract Test
```typescript
it('should get dashboard summary for current month', async () => {
  const response = await invoke('get_dashboard_summary', {
    period: 'current_month'
  });

  expect(response.total_spending).toBeGreaterThanOrEqual(0);
  expect(response.top_categories).toHaveLength(5);
  expect(response.debt_summary).toBeDefined();
  expect(response.target_summary).toBeDefined();
});
```

---

## Command: `export_analytics_report`
Export analytics data to PDF or spreadsheet.

### Request
```typescript
interface ExportAnalyticsReportRequest {
  format: 'pdf' | 'xlsx';
  start_date: string;
  end_date: string;
  include_charts: boolean;
  output_path: string;
}
```

### Response
```typescript
interface ExportAnalyticsReportResponse {
  success: boolean;
  file_path: string;
  file_size: number;           // Bytes
}
```

### Errors
- `WriteError`: Cannot write to output_path

### Contract Test
```typescript
it('should export analytics report to PDF', async () => {
  const response = await invoke('export_analytics_report', {
    format: 'pdf',
    start_date: '2025-01-01',
    end_date: '2025-12-31',
    include_charts: true,
    output_path: '/tmp/report.pdf'
  });

  expect(response.success).toBe(true);
  expect(response.file_path).toBe('/tmp/report.pdf');
  expect(response.file_size).toBeGreaterThan(0);
});
```
