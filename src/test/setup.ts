// Vitest setup file
import { afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import '@testing-library/jest-dom';

// Mock Tauri API
global.window = global.window || {};
global.window.__TAURI__ = {
  invoke: vi.fn(),
  tauri: {
    invoke: vi.fn(),
  },
};

// Mock Tauri modules
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readTextFile: vi.fn(),
  writeTextFile: vi.fn(),
}));

// Cleanup after each test case
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});
