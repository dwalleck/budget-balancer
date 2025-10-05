import { create } from "zustand";

interface UIState {
  // Sidebar
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;

  // Modals
  csvUploadOpen: boolean;
  accountCreationOpen: boolean;
  debtFormOpen: boolean;
  targetFormOpen: boolean;

  openCsvUpload: () => void;
  closeCsvUpload: () => void;
  openAccountCreation: () => void;
  closeAccountCreation: () => void;
  openDebtForm: () => void;
  closeDebtForm: () => void;
  openTargetForm: () => void;
  closeTargetForm: () => void;

  // Loading states
  globalLoading: boolean;
  setGlobalLoading: (loading: boolean) => void;

  // Toast notifications
  toast: {
    message: string;
    type: "success" | "error" | "info" | "warning";
  } | null;
  showToast: (
    message: string,
    type: "success" | "error" | "info" | "warning"
  ) => void;
  hideToast: () => void;

  // Current page
  currentPage: string;
  setCurrentPage: (page: string) => void;
}

export const useUIStore = create<UIState>((set) => ({
  // Sidebar
  sidebarCollapsed: false,
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  // Modals
  csvUploadOpen: false,
  accountCreationOpen: false,
  debtFormOpen: false,
  targetFormOpen: false,

  openCsvUpload: () => set({ csvUploadOpen: true }),
  closeCsvUpload: () => set({ csvUploadOpen: false }),
  openAccountCreation: () => set({ accountCreationOpen: true }),
  closeAccountCreation: () => set({ accountCreationOpen: false }),
  openDebtForm: () => set({ debtFormOpen: true }),
  closeDebtForm: () => set({ debtFormOpen: false }),
  openTargetForm: () => set({ targetFormOpen: true }),
  closeTargetForm: () => set({ targetFormOpen: false }),

  // Loading states
  globalLoading: false,
  setGlobalLoading: (loading) => set({ globalLoading: loading }),

  // Toast notifications
  toast: null,
  showToast: (message, type) => set({ toast: { message, type } }),
  hideToast: () => set({ toast: null }),

  // Current page
  currentPage: "transactions",
  setCurrentPage: (page) => set({ currentPage: page }),
}));
