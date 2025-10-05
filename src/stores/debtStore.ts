import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface Debt {
  id: number;
  name: string;
  balance: number;
  original_balance: number;
  interest_rate: number;
  min_payment: number;
  created_at: string;
  updated_at: string;
}

export interface PayoffPlan {
  id: number;
  strategy: "avalanche" | "snowball";
  monthly_amount: number;
  total_interest: number;
  payoff_date: string;
  monthly_breakdown: MonthlyPayment[];
}

export interface MonthlyPayment {
  month: number;
  payments: DebtPayment[];
  total_paid: number;
  remaining_balance: number;
}

export interface DebtPayment {
  debt_id: number;
  debt_name: string;
  amount: number;
  principal: number;
  interest: number;
}

interface DebtState {
  debts: Debt[];
  activePlan: PayoffPlan | null;
  loading: boolean;
  error: string | null;

  // Actions
  fetchDebts: () => Promise<void>;
  createDebt: (debt: Omit<Debt, "id" | "created_at" | "updated_at">) => Promise<void>;
  updateDebt: (id: number, debt: Partial<Debt>) => Promise<void>;
  calculatePayoffPlan: (strategy: string, monthlyAmount: number) => Promise<void>;
  recordPayment: (debtId: number, amount: number) => Promise<void>;
}

export const useDebtStore = create<DebtState>((set, get) => ({
  debts: [],
  activePlan: null,
  loading: false,
  error: null,

  fetchDebts: async () => {
    try {
      set({ loading: true, error: null });
      const debts = await invoke<Debt[]>("list_debts");
      set({ debts, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  createDebt: async (debt) => {
    try {
      set({ loading: true, error: null });
      await invoke("create_debt", debt);
      await get().fetchDebts();
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  updateDebt: async (id, debt) => {
    try {
      set({ loading: true, error: null });
      await invoke("update_debt", { id, ...debt });
      await get().fetchDebts();
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  calculatePayoffPlan: async (strategy, monthlyAmount) => {
    try {
      set({ loading: true, error: null });
      const plan = await invoke<PayoffPlan>("calculate_payoff_plan", {
        strategy,
        monthlyAmount,
      });
      set({ activePlan: plan, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  recordPayment: async (debtId, amount) => {
    try {
      set({ loading: true, error: null });
      await invoke("record_debt_payment", {
        debtId,
        amount,
        date: new Date().toISOString().split("T")[0],
      });
      await get().fetchDebts();
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },
}));
