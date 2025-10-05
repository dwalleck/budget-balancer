import { create } from 'zustand';
import { Category, NewCategory, listCategories, createCategory } from '../lib/tauri';

interface CategoryStore {
  categories: Category[];
  loading: boolean;
  error: string | null;

  fetchCategories: () => Promise<void>;
  addCategory: (category: NewCategory) => Promise<number>;
}

export const useCategoryStore = create<CategoryStore>((set) => ({
  categories: [],
  loading: false,
  error: null,

  fetchCategories: async () => {
    set({ loading: true, error: null });
    try {
      const categories = await listCategories();
      set({ categories, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  addCategory: async (category: NewCategory) => {
    try {
      const id = await createCategory(category);
      // Refresh categories
      const categories = await listCategories();
      set({ categories });
      return id;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
