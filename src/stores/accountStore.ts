import { create } from 'zustand';
import { Account, NewAccount, listAccounts, createAccount } from '../lib/tauri';

interface AccountStore {
  accounts: Account[];
  loading: boolean;
  error: string | null;

  fetchAccounts: () => Promise<void>;
  addAccount: (account: NewAccount) => Promise<number>;
}

export const useAccountStore = create<AccountStore>((set) => ({
  accounts: [],
  loading: false,
  error: null,

  fetchAccounts: async () => {
    set({ loading: true, error: null });
    try {
      const accounts = await listAccounts();
      set({ accounts, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  addAccount: async (account: NewAccount) => {
    try {
      const id = await createAccount(account);
      // Refresh accounts
      const accounts = await listAccounts();
      set({ accounts });
      return id;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
