import React from 'react';
import { Transaction, Category } from '../lib/tauri';
import { useTransactionStore } from '../stores/transactionStore';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/Select';

interface TransactionCategoryEditorProps {
  transaction: Transaction;
  categories: Category[];
  currentCategoryName: string;
}

export const TransactionCategoryEditor: React.FC<TransactionCategoryEditorProps> = ({
  transaction,
  categories,
  currentCategoryName,
}) => {
  const { updateCategory } = useTransactionStore();

  const handleCategoryChange = async (categoryId: string) => {
    await updateCategory(transaction.id, parseInt(categoryId));
  };

  return (
    <Select
      value={transaction.category_id.toString()}
      onValueChange={handleCategoryChange}
    >
      <SelectTrigger className="w-[180px]">
        <SelectValue>{currentCategoryName}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        {categories.map((category) => (
          <SelectItem key={category.id} value={category.id.toString()}>
            {category.icon && <span className="mr-2">{category.icon}</span>}
            {category.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};
