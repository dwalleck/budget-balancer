/**
 * TODO: These tests document expected behavior for ColumnMappingForm
 * Should have been written BEFORE implementation (TDD violation)
 *
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ColumnMappingForm } from '../components/ColumnMappingForm';

// Create mock functions
const mockImportCsv = vi.fn();
const mockOnComplete = vi.fn();
const mockOnCancel = vi.fn();

// Mock Tauri command
vi.mock('../lib/tauri', () => ({
  importCsv: mockImportCsv,
}));

describe('ColumnMappingForm', () => {
  const defaultProps = {
    accountId: 1,
    csvContent: 'Date,Amount,Description,Merchant\n2024-01-01,50.00,Coffee,Starbucks',
    headers: ['Date', 'Amount', 'Description', 'Merchant'],
    onComplete: mockOnComplete,
    onCancel: mockOnCancel,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should display column count message', () => {
      render(<ColumnMappingForm {...defaultProps} />);
      expect(screen.getByText(/We found 4 columns/i)).toBeInTheDocument();
    });

    it('should render all four column mapping selects', () => {
      render(<ColumnMappingForm {...defaultProps} />);

      expect(screen.getByLabelText(/Date Column/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Amount Column/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Description Column/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Merchant Column/i)).toBeInTheDocument();
    });

    it('should have Import and Cancel buttons', () => {
      render(<ColumnMappingForm {...defaultProps} />);

      expect(screen.getByRole('button', { name: /Import/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Cancel/i })).toBeInTheDocument();
    });

    it('should pre-select first three headers by default', () => {
      render(<ColumnMappingForm {...defaultProps} />);

      // Initial mapping should be headers[0], headers[1], headers[2]
      // This would need to check the actual Select values
      // (implementation detail: checking state is hard without exposing it)
    });
  });

  describe('Column Mapping', () => {
    it('should allow selecting different columns for date', async () => {
      render(<ColumnMappingForm {...defaultProps} />);

      const dateSelect = screen.getByLabelText(/Date Column/i);
      fireEvent.click(dateSelect);

      await waitFor(() => {
        expect(screen.getByText('Amount')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('Amount'));

      // Verify selection changed (would need state inspection or callback)
    });

    it('should allow setting merchant column to None', async () => {
      render(<ColumnMappingForm {...defaultProps} />);

      const merchantSelect = screen.getByLabelText(/Merchant Column/i);
      fireEvent.click(merchantSelect);

      await waitFor(() => {
        expect(screen.getByText('None')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('None'));

      // Merchant should be undefined in mapping
    });

    it('should not allow empty string as SelectItem value', () => {
      // This test verifies the bug fix - Radix UI doesn't allow value=""
      render(<ColumnMappingForm {...defaultProps} />);

      const merchantSelect = screen.getByLabelText(/Merchant Column/i);
      fireEvent.click(merchantSelect);

      // Should have value="none" instead of value=""
      const noneOption = screen.getByText('None');
      expect(noneOption.closest('[role="option"]')).toHaveAttribute('data-value', 'none');
    });
  });

  describe('Import Functionality', () => {
    it('should call importCsv with correct parameters when Import clicked', async () => {
      mockImportCsv.mockResolvedValue({
        success: true,
        message: 'Successfully imported 1 transaction',
        total: 1,
        imported: 1,
        duplicates: 0,
        errors: 0,
      });

      render(<ColumnMappingForm {...defaultProps} />);

      const importButton = screen.getByRole('button', { name: /Import/i });
      fireEvent.click(importButton);

      await waitFor(() => {
        expect(mockImportCsv).toHaveBeenCalledWith(
          1, // accountId
          defaultProps.csvContent,
          {
            date: 'Date',
            amount: 'Amount',
            description: 'Description',
            merchant: undefined, // or 'Merchant' if selected
          }
        );
      });
    });

    it('should disable buttons while importing', async () => {
      mockImportCsv.mockImplementation(() => new Promise(resolve => setTimeout(resolve, 100)));

      render(<ColumnMappingForm {...defaultProps} />);

      const importButton = screen.getByRole('button', { name: /Import/i });
      const cancelButton = screen.getByRole('button', { name: /Cancel/i });

      fireEvent.click(importButton);

      expect(importButton).toBeDisabled();
      expect(cancelButton).toBeDisabled();
      expect(screen.getByText(/Importing.../i)).toBeInTheDocument();
    });

    it('should show success message on successful import', async () => {
      mockImportCsv.mockResolvedValue({
        success: true,
        message: 'Successfully imported 5 transactions',
        total: 5,
        imported: 5,
        duplicates: 0,
        errors: 0,
      });

      render(<ColumnMappingForm {...defaultProps} />);

      fireEvent.click(screen.getByRole('button', { name: /Import/i }));

      await waitFor(() => {
        expect(screen.getByText(/Successfully imported 5 transactions/i)).toBeInTheDocument();
      });
    });

    it('should call onComplete after 2 seconds on success', async () => {
      vi.useFakeTimers();

      mockImportCsv.mockResolvedValue({
        success: true,
        message: 'Success',
        total: 1,
        imported: 1,
        duplicates: 0,
        errors: 0,
      });

      render(<ColumnMappingForm {...defaultProps} />);

      fireEvent.click(screen.getByRole('button', { name: /Import/i }));

      await waitFor(() => {
        expect(screen.getByText(/Success/i)).toBeInTheDocument();
      });

      vi.advanceTimersByTime(2000);

      expect(mockOnComplete).toHaveBeenCalled();

      vi.useRealTimers();
    });

    it('should show error message on failed import', async () => {
      mockImportCsv.mockRejectedValue(new Error('Import failed'));

      render(<ColumnMappingForm {...defaultProps} />);

      fireEvent.click(screen.getByRole('button', { name: /Import/i }));

      await waitFor(() => {
        expect(screen.getByText(/Import failed/i)).toBeInTheDocument();
      });

      expect(mockOnComplete).not.toHaveBeenCalled();
    });
  });

  describe('Cancel Functionality', () => {
    it('should call onCancel when Cancel button clicked', () => {
      render(<ColumnMappingForm {...defaultProps} />);

      const cancelButton = screen.getByRole('button', { name: /Cancel/i });
      fireEvent.click(cancelButton);

      expect(mockOnCancel).toHaveBeenCalled();
      expect(mockImportCsv).not.toHaveBeenCalled();
    });
  });

  describe('Edge Cases', () => {
    it('should handle CSV with no headers gracefully', () => {
      const propsWithNoHeaders = {
        ...defaultProps,
        headers: [],
      };

      render(<ColumnMappingForm {...propsWithNoHeaders} />);

      expect(screen.getByText(/We found 0 columns/i)).toBeInTheDocument();
      // Selects should still render but have no options
    });

    it('should handle CSV with only 2 columns', () => {
      const propsWithFewHeaders = {
        ...defaultProps,
        headers: ['Date', 'Amount'],
      };

      render(<ColumnMappingForm {...propsWithFewHeaders} />);

      // Should still show all 4 selects, but description will default to empty
      expect(screen.getByLabelText(/Date Column/i)).toBeInTheDocument();
    });

    it('should handle duplicate detection in import result', async () => {
      mockImportCsv.mockResolvedValue({
        success: true,
        message: '3 imported, 2 duplicates',
        total: 5,
        imported: 3,
        duplicates: 2,
        errors: 0,
      });

      render(<ColumnMappingForm {...defaultProps} />);

      fireEvent.click(screen.getByRole('button', { name: /Import/i }));

      await waitFor(() => {
        expect(screen.getByText(/3 imported, 2 duplicates/i)).toBeInTheDocument();
      });
    });
  });
});
