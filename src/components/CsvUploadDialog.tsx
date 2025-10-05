import React, { useState } from 'react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from './ui/Dialog';
import { Button } from './ui/Button';
import { ColumnMappingForm } from './ColumnMappingForm';
import { getCsvHeaders } from '../lib/tauri';

interface CsvUploadDialogProps {
  accountId: number;
  onImportComplete: () => void;
}

export const CsvUploadDialog: React.FC<CsvUploadDialogProps> = ({
  accountId,
  onImportComplete,
}) => {
  const [open, setOpen] = useState(false);
  const [csvContent, setCsvContent] = useState<string | null>(null);
  const [headers, setHeaders] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const handleFileSelect = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });

      if (selected && typeof selected === 'string') {
        const content = await readTextFile(selected);
        setCsvContent(content);

        // Get headers from CSV
        const csvHeaders = await getCsvHeaders(content);
        setHeaders(csvHeaders);
        setError(null);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleComplete = () => {
    setCsvContent(null);
    setHeaders([]);
    setOpen(false);
    onImportComplete();
  };

  const handleCancel = () => {
    setCsvContent(null);
    setHeaders([]);
    setError(null);
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>Import CSV</Button>
      </DialogTrigger>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Import Transactions from CSV</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          {!csvContent ? (
            <div className="flex flex-col items-center gap-4 py-8">
              <Button onClick={handleFileSelect}>Select CSV File</Button>
              {error && (
                <p className="text-sm text-destructive">{error}</p>
              )}
            </div>
          ) : (
            <ColumnMappingForm
              accountId={accountId}
              csvContent={csvContent}
              headers={headers}
              onComplete={handleComplete}
              onCancel={handleCancel}
            />
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};
