import { create } from 'zustand';
import { Transaction, TransactionFilter, listTransactions, countTransactions, updateTransactionCategory } from '../lib/tauri';

interface TransactionStore {
  transactions: Transaction[];
  totalCount: number;
  loading: boolean;
  error: string | null;

  fetchTransactions: (filter?: TransactionFilter) => Promise<void>;
  fetchCount: (filter?: TransactionFilter) => Promise<void>;
  updateCategory: (transactionId: number, categoryId: number) => Promise<void>;
}

export const useTransactionStore = create<TransactionStore>((set, get) => ({
  transactions: [],
  totalCount: 0,
  loading: false,
  error: null,

  fetchTransactions: async (filter?: TransactionFilter) => {
    set({ loading: true, error: null });
    try {
      const transactions = await listTransactions(filter);
      set({ transactions, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  fetchCount: async (filter?: TransactionFilter) => {
    try {
      const totalCount = await countTransactions(filter);
      set({ totalCount });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  updateCategory: async (transactionId: number, categoryId: number) => {
    try {
      await updateTransactionCategory(transactionId, categoryId);
      // Update local state instead of refetching
      set((state) => ({
        transactions: state.transactions.map((t) =>
          t.id === transactionId ? { ...t, category_id: categoryId } : t
        ),
      }));
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
