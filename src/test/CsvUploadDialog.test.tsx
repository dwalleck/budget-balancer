/**
 * TODO: Frontend testing infrastructure needs configuration
 *
 * This test file documents the expected behavior for CSV upload dialog
 * but currently fails due to jsdom environment not being properly loaded with bun test.
 *
 * To fix:
 * 1. Configure vitest to properly load jsdom environment
 * 2. Or migrate to a different test runner that handles jsdom better
 * 3. Or use Tauri's test utilities for E2E testing instead
 *
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CsvUploadDialog } from '../components/CsvUploadDialog';

// Create mock functions
const mockOpenDialog = vi.fn();
const mockReadTextFile = vi.fn();
const mockGetCsvHeaders = vi.fn();

// Mock Tauri modules
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: mockOpenDialog,
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readTextFile: mockReadTextFile,
}));

vi.mock('../lib/tauri', () => ({
  getCsvHeaders: mockGetCsvHeaders,
  importCsv: vi.fn(),
}));

describe('CsvUploadDialog', () => {
  const mockOnImportComplete = vi.fn();
  const mockAccountId = 1;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render import CSV button', () => {
    render(
      <CsvUploadDialog
        accountId={mockAccountId}
        onImportComplete={mockOnImportComplete}
      />
    );

    expect(screen.getByText('Import CSV')).toBeInTheDocument();
  });

  it('should open dialog when trigger button is clicked', async () => {
    render(
      <CsvUploadDialog
        accountId={mockAccountId}
        onImportComplete={mockOnImportComplete}
      />
    );

    const triggerButton = screen.getByText('Import CSV');
    fireEvent.click(triggerButton);

    await waitFor(() => {
      expect(screen.getByText('Import Transactions from CSV')).toBeInTheDocument();
    });
  });

  it('should call dialog.open with correct permissions when selecting file', async () => {
    mockOpenDialog.mockResolvedValue('/path/to/file.csv');
    mockReadTextFile.mockResolvedValue('Date,Amount,Description\n2024-01-01,50.00,Test');
    mockGetCsvHeaders.mockResolvedValue(['Date', 'Amount', 'Description']);

    render(
      <CsvUploadDialog
        accountId={mockAccountId}
        onImportComplete={mockOnImportComplete}
      />
    );

    // Open dialog
    fireEvent.click(screen.getByText('Import CSV'));

    // Click select file button
    await waitFor(() => {
      expect(screen.getByText('Select CSV File')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText('Select CSV File'));

    // Verify dialog.open was called with correct parameters
    await waitFor(() => {
      expect(mockOpenDialog).toHaveBeenCalledWith({
        multiple: false,
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });
    });
  });

  it('should handle permission errors gracefully', async () => {
    mockOpenDialog.mockRejectedValue(new Error('dialog.open not allowed'));

    render(
      <CsvUploadDialog
        accountId={mockAccountId}
        onImportComplete={mockOnImportComplete}
      />
    );

    // Open dialog
    fireEvent.click(screen.getByText('Import CSV'));

    // Click select file button
    await waitFor(() => {
      expect(screen.getByText('Select CSV File')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText('Select CSV File'));

    // Verify error is displayed
    await waitFor(() => {
      expect(screen.getByText(/dialog.open not allowed/i)).toBeInTheDocument();
    });
  });

  it('should read and parse CSV file after selection', async () => {
    mockOpenDialog.mockResolvedValue('/path/to/file.csv');
    const csvContent = 'Date,Amount,Description\n2024-01-01,50.00,Coffee';
    mockReadTextFile.mockResolvedValue(csvContent);
    mockGetCsvHeaders.mockResolvedValue(['Date', 'Amount', 'Description']);

    render(
      <CsvUploadDialog
        accountId={mockAccountId}
        onImportComplete={mockOnImportComplete}
      />
    );

    // Open dialog and select file
    fireEvent.click(screen.getByText('Import CSV'));
    await waitFor(() => screen.getByText('Select CSV File'));
    fireEvent.click(screen.getByText('Select CSV File'));

    // Verify file was read
    await waitFor(() => {
      expect(mockReadTextFile).toHaveBeenCalledWith('/path/to/file.csv');
    });

    // Verify headers were parsed
    await waitFor(() => {
      expect(mockGetCsvHeaders).toHaveBeenCalledWith(csvContent);
    });
  });
});
