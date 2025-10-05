import React, { useEffect, useState } from 'react';
import { useAccountStore } from '../stores/accountStore';
import { TransactionList } from '../components/TransactionList';
import { CsvUploadDialog } from '../components/CsvUploadDialog';
import { AccountCreationDialog } from '../components/AccountCreationDialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/Select';

export const TransactionsPage: React.FC = () => {
  const { accounts, fetchAccounts } = useAccountStore();
  const [selectedAccountId, setSelectedAccountId] = useState<number | undefined>();
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    fetchAccounts();
  }, [fetchAccounts]);

  useEffect(() => {
    if (accounts.length > 0 && !selectedAccountId) {
      setSelectedAccountId(accounts[0].id);
    }
  }, [accounts, selectedAccountId]);

  const handleImportComplete = () => {
    setRefreshKey((prev) => prev + 1);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Transactions</h1>
        <div className="flex gap-4 items-center">
          {accounts.length > 0 && (
            <>
              <div className="flex items-center gap-2">
                <label className="text-sm font-medium">Account:</label>
                <Select
                  value={selectedAccountId?.toString() || ''}
                  onValueChange={(value) => setSelectedAccountId(parseInt(value))}
                >
                  <SelectTrigger className="w-[200px]">
                    <SelectValue placeholder="Select account" />
                  </SelectTrigger>
                  <SelectContent>
                    {accounts.map((account) => (
                      <SelectItem key={account.id} value={account.id.toString()}>
                        {account.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              {selectedAccountId && (
                <CsvUploadDialog
                  accountId={selectedAccountId}
                  onImportComplete={handleImportComplete}
                />
              )}
            </>
          )}
        </div>
      </div>

      {accounts.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-muted-foreground mb-4">
            No accounts found. Create an account to get started.
          </p>
          <AccountCreationDialog onAccountCreated={() => fetchAccounts()} />
        </div>
      ) : (
        <div key={refreshKey}>
          <TransactionList accountId={selectedAccountId} />
        </div>
      )}
    </div>
  );
};
