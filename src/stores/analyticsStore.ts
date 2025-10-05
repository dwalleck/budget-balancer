import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface CategorySpending {
  category_id: number;
  category_name: string;
  category_icon: string | null;
  amount: number;
  percentage: number;
  transaction_count: number;
}

export interface SpendingByCategory {
  period: {
    start_date: string;
    end_date: string;
  };
  categories: CategorySpending[];
  total_spending: number;
}

export interface TrendPoint {
  date: string;
  amount: number;
  transaction_count: number;
}

export interface SpendingTrends {
  data_points: TrendPoint[];
  total_spending: number;
  average_per_interval: number;
}

export interface TargetProgress {
  category_id: number;
  category_name: string;
  target_amount: number;
  actual_amount: number;
  remaining: number;
  percentage_used: number;
  status: "under" | "on_track" | "over";
  variance: number;
}

export interface DashboardSummary {
  period: {
    start_date: string;
    end_date: string;
  };
  total_spending: number;
  total_income: number;
  net: number;
  top_categories: CategorySpending[];
  debt_summary: {
    total_debt: number;
    total_monthly_payment: number;
    next_payoff_date: string | null;
  };
  target_summary: {
    on_track_count: number;
    over_count: number;
    total_variance: number;
  };
}

interface AnalyticsState {
  spendingByCategory: SpendingByCategory | null;
  trends: SpendingTrends | null;
  targets: TargetProgress[];
  dashboard: DashboardSummary | null;
  loading: boolean;
  error: string | null;

  // Actions
  fetchSpendingByCategory: (
    startDate: string,
    endDate: string,
    accountId?: number
  ) => Promise<void>;
  fetchTrends: (
    startDate: string,
    endDate: string,
    interval: "daily" | "weekly" | "monthly",
    categoryId?: number
  ) => Promise<void>;
  fetchTargetsProgress: (
    period?: string,
    customStart?: string,
    customEnd?: string
  ) => Promise<void>;
  fetchDashboard: (period: string) => Promise<void>;
  createTarget: (
    categoryId: number,
    amount: number,
    period: string,
    startDate: string
  ) => Promise<void>;
}

export const useAnalyticsStore = create<AnalyticsState>((set) => ({
  spendingByCategory: null,
  trends: null,
  targets: [],
  dashboard: null,
  loading: false,
  error: null,

  fetchSpendingByCategory: async (startDate, endDate, accountId) => {
    try {
      set({ loading: true, error: null });
      const data = await invoke<SpendingByCategory>("get_spending_by_category", {
        startDate,
        endDate,
        accountId,
      });
      set({ spendingByCategory: data, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  fetchTrends: async (startDate, endDate, interval, categoryId) => {
    try {
      set({ loading: true, error: null });
      const data = await invoke<SpendingTrends>("get_spending_trends", {
        startDate,
        endDate,
        interval,
        categoryId,
      });
      set({ trends: data, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  fetchTargetsProgress: async (period, customStart, customEnd) => {
    try {
      set({ loading: true, error: null });
      const data = await invoke<{ targets: TargetProgress[] }>(
        "get_spending_targets_progress",
        {
          period,
          customStart,
          customEnd,
        }
      );
      set({ targets: data.targets, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  fetchDashboard: async (period) => {
    try {
      set({ loading: true, error: null });
      const data = await invoke<DashboardSummary>("get_dashboard_summary", {
        period,
      });
      set({ dashboard: data, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  createTarget: async (categoryId, amount, period, startDate) => {
    try {
      set({ loading: true, error: null });
      await invoke("create_spending_target", {
        categoryId,
        amount,
        period,
        startDate,
      });
      set({ loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },
}));
