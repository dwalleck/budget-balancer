import { invoke } from '@tauri-apps/api/core';

export interface ColumnMapping {
  date: string;
  amount: string;
  description: string;
  merchant?: string;
}

export interface ImportResult {
  success: boolean;
  total: number;
  imported: number;
  duplicates: number;
  errors: number;
  message: string;
}

export interface Transaction {
  id: number;
  account_id: number;
  category_id: number;
  date: string;
  amount: number;
  description: string;
  merchant?: string;
  hash: string;
  created_at: string;
}

export interface TransactionFilter {
  account_id?: number;
  category_id?: number;
  start_date?: string;
  end_date?: string;
  limit?: number;
  offset?: number;
}

export interface Category {
  id: number;
  name: string;
  type: string;
  parent_id?: number;
  icon?: string;
  created_at: string;
}

export interface NewCategory {
  name: string;
  icon?: string;
}

export interface Account {
  id: number;
  name: string;
  type: string;
  balance: number;
  created_at: string;
  updated_at: string;
}

export interface NewAccount {
  name: string;
  account_type: 'checking' | 'savings' | 'credit_card';
  initial_balance: number;
}

// CSV Commands
export const getCsvHeaders = (csvContent: string): Promise<string[]> =>
  invoke('get_csv_headers', { csvContent });

export const importCsv = (
  accountId: number,
  csvContent: string,
  mapping: ColumnMapping
): Promise<ImportResult> =>
  invoke('import_csv', { accountId, csvContent, mapping });

// Transaction Commands
export const listTransactions = (
  filter?: TransactionFilter
): Promise<Transaction[]> =>
  invoke('list_transactions', { filter });

export const countTransactions = (
  filter?: TransactionFilter
): Promise<number> =>
  invoke('count_transactions', { filter });

export const updateTransactionCategory = (
  transactionId: number,
  categoryId: number
): Promise<void> =>
  invoke('update_transaction_category', { transactionId, categoryId });

// Category Commands
export const listCategories = (): Promise<Category[]> =>
  invoke('list_categories');

export const createCategory = (category: NewCategory): Promise<number> =>
  invoke('create_category', { category });

// Account Commands
export const listAccounts = (): Promise<Account[]> =>
  invoke('list_accounts');

export const createAccount = (account: NewAccount): Promise<number> =>
  invoke('create_account', { account });
