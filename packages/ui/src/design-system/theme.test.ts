// packages/ui/src/design-system/theme.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { getTheme, lightTheme, darkTheme } from './theme';

describe('theme', () => {
  beforeEach(() => {
    // Reset window.matchMedia mock
    vi.clearAllMocks();
  });

  describe('lightTheme', () => {
    it('should have all required theme properties', () => {
      expect(lightTheme.background).toBeDefined();
      expect(lightTheme.surface).toBeDefined();
      expect(lightTheme.border).toBeDefined();
      expect(lightTheme.text.primary).toBeDefined();
      expect(lightTheme.text.secondary).toBeDefined();
      expect(lightTheme.text.tertiary).toBeDefined();
      expect(lightTheme.primary.default).toBeDefined();
      expect(lightTheme.primary.hover).toBeDefined();
      expect(lightTheme.primary.active).toBeDefined();
      expect(lightTheme.neutral).toBeDefined();
      expect(lightTheme.semantic.success).toBeDefined();
      expect(lightTheme.semantic.error).toBeDefined();
      expect(lightTheme.semantic.warning).toBeDefined();
      expect(lightTheme.semantic.info).toBeDefined();
    });

    it('should have light background color', () => {
      expect(lightTheme.background).toBe('#ffffff');
      expect(lightTheme.surface).toBe('#f8fafc');
    });

    it('should have dark text colors', () => {
      expect(lightTheme.text.primary).toBe('#0f172a');
      expect(lightTheme.text.secondary).toBe('#475569');
    });
  });

  describe('darkTheme', () => {
    it('should have all required theme properties', () => {
      expect(darkTheme.background).toBeDefined();
      expect(darkTheme.surface).toBeDefined();
      expect(darkTheme.border).toBeDefined();
      expect(darkTheme.text.primary).toBeDefined();
      expect(darkTheme.text.secondary).toBeDefined();
      expect(darkTheme.text.tertiary).toBeDefined();
      expect(darkTheme.primary.default).toBeDefined();
      expect(darkTheme.primary.hover).toBeDefined();
      expect(darkTheme.primary.active).toBeDefined();
      expect(darkTheme.neutral).toBeDefined();
      expect(darkTheme.semantic.success).toBeDefined();
      expect(darkTheme.semantic.error).toBeDefined();
      expect(darkTheme.semantic.warning).toBeDefined();
      expect(darkTheme.semantic.info).toBeDefined();
    });

    it('should have dark background color', () => {
      expect(darkTheme.background).toBe('#0f172a');
      expect(darkTheme.surface).toBe('#1e293b');
    });

    it('should have light text colors', () => {
      expect(darkTheme.text.primary).toBe('#f1f5f9');
      expect(darkTheme.text.secondary).toBe('#cbd5e1');
    });
  });

  describe('getTheme', () => {
    it('should return light theme when mode is "light"', () => {
      const theme = getTheme('light');
      expect(theme).toBe(lightTheme);
    });

    it('should return dark theme when mode is "dark"', () => {
      const theme = getTheme('dark');
      expect(theme).toBe(darkTheme);
    });

    it('should return light theme as default when window is undefined', () => {
      const originalWindow = global.window;
      delete (global as any).window;
      
      const theme = getTheme('system');
      expect(theme).toBe(lightTheme);
      
      global.window = originalWindow;
    });

    it('should return light theme for system mode when prefers light', () => {
      Object.defineProperty(window, 'matchMedia', {
        writable: true,
        value: vi.fn().mockImplementation((query: string) => ({
          matches: false, // prefers light
          media: query,
          onchange: null,
          addListener: vi.fn(),
          removeListener: vi.fn(),
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
          dispatchEvent: vi.fn(),
        })),
      });

      const theme = getTheme('system');
      expect(theme).toBe(lightTheme);
    });

    it('should return dark theme for system mode when prefers dark', () => {
      Object.defineProperty(window, 'matchMedia', {
        writable: true,
        value: vi.fn().mockImplementation((query: string) => ({
          matches: true, // prefers dark
          media: query,
          onchange: null,
          addListener: vi.fn(),
          removeListener: vi.fn(),
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
          dispatchEvent: vi.fn(),
        })),
      });

      const theme = getTheme('system');
      expect(theme).toBe(darkTheme);
    });

    it('should check prefers-color-scheme media query', () => {
      const matchMediaSpy = vi.fn().mockImplementation((query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      }));

      Object.defineProperty(window, 'matchMedia', {
        writable: true,
        value: matchMediaSpy,
      });

      getTheme('system');
      expect(matchMediaSpy).toHaveBeenCalledWith('(prefers-color-scheme: dark)');
    });

    it('should default to system mode when no mode provided', () => {
      Object.defineProperty(window, 'matchMedia', {
        writable: true,
        value: vi.fn().mockImplementation(() => ({
          matches: false,
          media: '',
          onchange: null,
          addListener: vi.fn(),
          removeListener: vi.fn(),
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
          dispatchEvent: vi.fn(),
        })),
      });

      const theme = getTheme();
      expect(theme).toBe(lightTheme);
    });
  });
});
