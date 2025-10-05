import React, { useState } from 'react';
import { Button } from './ui/Button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/Select';
import { importCsv, ColumnMapping } from '../lib/tauri';

interface ColumnMappingFormProps {
  accountId: number;
  csvContent: string;
  headers: string[];
  onComplete: () => void;
  onCancel: () => void;
}

export const ColumnMappingForm: React.FC<ColumnMappingFormProps> = ({
  accountId,
  csvContent,
  headers,
  onComplete,
  onCancel,
}) => {
  const [mapping, setMapping] = useState<ColumnMapping>({
    date: headers[0] || '',
    amount: headers[1] || '',
    description: headers[2] || '',
    merchant: undefined,
  });
  const [importing, setImporting] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleImport = async () => {
    setImporting(true);
    setError(null);
    try {
      const importResult = await importCsv(accountId, csvContent, mapping);
      setResult(importResult.message);
      setTimeout(() => {
        onComplete();
      }, 2000);
    } catch (err) {
      setError(String(err));
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="text-sm text-gray-600 mb-4">
        Map your CSV columns to transaction fields. We found {headers.length} columns.
      </div>
      <div className="space-y-3">
        <div>
          <label className="block text-sm font-medium mb-1">Date Column</label>
          <Select
            value={mapping.date}
            onValueChange={(value) => setMapping({ ...mapping, date: value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select date column" />
            </SelectTrigger>
            <SelectContent>
              {headers.map((header) => (
                <SelectItem key={header} value={header}>
                  {header}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Amount Column</label>
          <Select
            value={mapping.amount}
            onValueChange={(value) => setMapping({ ...mapping, amount: value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select amount column" />
            </SelectTrigger>
            <SelectContent>
              {headers.map((header) => (
                <SelectItem key={header} value={header}>
                  {header}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Description Column</label>
          <Select
            value={mapping.description}
            onValueChange={(value) => setMapping({ ...mapping, description: value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select description column" />
            </SelectTrigger>
            <SelectContent>
              {headers.map((header) => (
                <SelectItem key={header} value={header}>
                  {header}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Merchant Column (Optional)</label>
          <Select
            value={mapping.merchant || 'none'}
            onValueChange={(value) => setMapping({ ...mapping, merchant: value === 'none' ? undefined : value })}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select merchant column (optional)" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none">None</SelectItem>
              {headers.map((header) => (
                <SelectItem key={header} value={header}>
                  {header}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {result && (
        <div className="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded text-sm text-green-800 dark:text-green-200">
          {result}
        </div>
      )}

      {error && (
        <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-sm text-red-800 dark:text-red-200">
          {error}
        </div>
      )}

      <div className="flex gap-2 justify-end">
        <Button variant="outline" onClick={onCancel} disabled={importing}>
          Cancel
        </Button>
        <Button onClick={handleImport} disabled={importing}>
          {importing ? 'Importing...' : 'Import'}
        </Button>
      </div>
    </div>
  );
};
