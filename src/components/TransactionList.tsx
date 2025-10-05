import React, { useEffect } from 'react';
import { useTransactionStore } from '../stores/transactionStore';
import { useCategoryStore } from '../stores/categoryStore';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from './ui/Table';
import { TransactionCategoryEditor } from './TransactionCategoryEditor';

interface TransactionListProps {
  accountId?: number;
}

export const TransactionList: React.FC<TransactionListProps> = ({ accountId }) => {
  const { transactions, loading, error, fetchTransactions } = useTransactionStore();
  const { categories, fetchCategories } = useCategoryStore();

  useEffect(() => {
    fetchTransactions(accountId ? { account_id: accountId } : undefined);
    fetchCategories();
  }, [accountId, fetchTransactions, fetchCategories]);

  const getCategoryName = (categoryId: number) => {
    const category = categories.find((c) => c.id === categoryId);
    return category ? category.name : 'Unknown';
  };

  const formatAmount = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(amount);
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  if (loading) {
    return <div className="text-center py-8">Loading transactions...</div>;
  }

  if (error) {
    return <div className="text-center py-8 text-destructive">{error}</div>;
  }

  if (transactions.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        No transactions found. Import a CSV file to get started.
      </div>
    );
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Date</TableHead>
          <TableHead>Description</TableHead>
          <TableHead>Merchant</TableHead>
          <TableHead>Category</TableHead>
          <TableHead className="text-right">Amount</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {transactions.map((transaction) => (
          <TableRow key={transaction.id}>
            <TableCell>{formatDate(transaction.date)}</TableCell>
            <TableCell>{transaction.description}</TableCell>
            <TableCell>{transaction.merchant || '-'}</TableCell>
            <TableCell>
              <TransactionCategoryEditor
                transaction={transaction}
                categories={categories}
                currentCategoryName={getCategoryName(transaction.category_id)}
              />
            </TableCell>
            <TableCell className="text-right">{formatAmount(transaction.amount)}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
};
