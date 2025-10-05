import React, { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from './ui/Dialog';
import { Button } from './ui/Button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/Select';
import { createAccount } from '../lib/tauri';

interface AccountCreationDialogProps {
  onAccountCreated: () => void;
}

export const AccountCreationDialog: React.FC<AccountCreationDialogProps> = ({
  onAccountCreated,
}) => {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState('');
  const [accountType, setAccountType] = useState<'checking' | 'savings' | 'credit_card'>('checking');
  const [initialBalance, setInitialBalance] = useState('0');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async () => {
    if (!name.trim()) {
      setError('Account name is required');
      return;
    }

    setCreating(true);
    setError(null);

    try {
      await createAccount({
        name: name.trim(),
        account_type: accountType,
        initial_balance: parseFloat(initialBalance) || 0,
      });

      // Reset form
      setName('');
      setAccountType('checking');
      setInitialBalance('0');
      setOpen(false);
      onAccountCreated();
    } catch (err) {
      setError(String(err));
    } finally {
      setCreating(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>Create Account</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create New Account</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">Account Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Chase Checking"
              className="w-full px-3 py-2 border border-input rounded-md bg-background"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">Account Type</label>
            <Select
              value={accountType}
              onValueChange={(value: any) => setAccountType(value)}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="checking">Checking</SelectItem>
                <SelectItem value="savings">Savings</SelectItem>
                <SelectItem value="credit_card">Credit Card</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">Initial Balance</label>
            <input
              type="number"
              step="0.01"
              value={initialBalance}
              onChange={(e) => setInitialBalance(e.target.value)}
              className="w-full px-3 py-2 border border-input rounded-md bg-background"
            />
          </div>

          {error && (
            <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-sm text-red-800 dark:text-red-200">
              {error}
            </div>
          )}

          <div className="flex gap-2 justify-end">
            <Button variant="outline" onClick={() => setOpen(false)} disabled={creating}>
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={creating}>
              {creating ? 'Creating...' : 'Create Account'}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};