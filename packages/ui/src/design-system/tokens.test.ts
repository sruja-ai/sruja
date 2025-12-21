// packages/ui/src/design-system/tokens.test.ts
import { describe, it, expect } from 'vitest';
import { colors, spacing, typography, borderRadius, shadows, breakpoints } from './tokens';

describe('design tokens', () => {
  describe('colors', () => {
    it('should have primary color scale', () => {
      expect(colors.primary).toBeDefined();
      expect(colors.primary[50]).toBe('#f0f9ff');
      expect(colors.primary[500]).toBe('#0ea5e9');
      expect(colors.primary[950]).toBe('#082f49');
    });

    it('should have neutral color scale', () => {
      expect(colors.neutral).toBeDefined();
      expect(colors.neutral[50]).toBe('#f8fafc');
      expect(colors.neutral[500]).toBe('#64748b');
      expect(colors.neutral[950]).toBe('#020617');
    });

    it('should have semantic colors', () => {
      expect(colors.success).toBeDefined();
      expect(colors.success[500]).toBe('#22c55e');
      
      expect(colors.error).toBeDefined();
      expect(colors.error[500]).toBe('#ef4444');
      
      expect(colors.warning).toBeDefined();
      expect(colors.warning[500]).toBe('#f59e0b');
      
      expect(colors.info).toBeDefined();
      expect(colors.info[500]).toBe('#3b82f6');
    });

    it('should have brand colors', () => {
      expect(colors.brand).toBeDefined();
      expect(colors.brand.violet).toBe('#7C3AED');
      expect(colors.brand.blue).toBe('#2563EB');
      expect(colors.brand.pink).toBe('#DB2777');
      expect(colors.brand.pinkLight).toBe('#F472B6');
    });

    it('should have valid hex color format', () => {
      const hexPattern = /^#[0-9A-Fa-f]{6}$/;
      
      Object.values(colors.primary).forEach(color => {
        expect(color).toMatch(hexPattern);
      });
      
      Object.values(colors.neutral).forEach(color => {
        expect(color).toMatch(hexPattern);
      });
    });
  });

  describe('spacing', () => {
    it('should have spacing scale', () => {
      expect(spacing[0]).toBe('0');
      expect(spacing[1]).toBe('0.25rem');
      expect(spacing[4]).toBe('1rem');
      expect(spacing[24]).toBe('6rem');
    });

    it('should have rem-based spacing values', () => {
      Object.values(spacing).forEach(value => {
        if (value !== '0') {
          expect(value).toMatch(/^\d+\.?\d*rem$/);
        }
      });
    });
  });

  describe('typography', () => {
    it('should have font families', () => {
      expect(typography.fontFamily.sans).toBeDefined();
      expect(typography.fontFamily.mono).toBeDefined();
      expect(typography.fontFamily.mono).toContain('Monaco');
    });

    it('should have font sizes', () => {
      expect(typography.fontSize.xs).toBe('12px');
      expect(typography.fontSize.sm).toBe('14px');
      expect(typography.fontSize.base).toBe('17px');
      expect(typography.fontSize['4xl']).toBe('48px');
    });

    it('should have font weights', () => {
      expect(typography.fontWeight.normal).toBe(400);
      expect(typography.fontWeight.medium).toBe(500);
      expect(typography.fontWeight.semibold).toBe(600);
      expect(typography.fontWeight.bold).toBe(700);
    });

    it('should have line heights', () => {
      expect(typography.lineHeight.tight).toBe(1.25);
      expect(typography.lineHeight.normal).toBe(1.5);
      expect(typography.lineHeight.relaxed).toBe(1.75);
    });
  });

  describe('borderRadius', () => {
    it('should have border radius values', () => {
      expect(borderRadius.none).toBe('0');
      expect(borderRadius.sm).toBe('0.125rem');
      expect(borderRadius.base).toBe('0.25rem');
      expect(borderRadius.full).toBe('9999px');
    });

    it('should have valid rem or px format', () => {
      Object.values(borderRadius).forEach(value => {
        if (value !== '0' && value !== '9999px') {
          expect(value).toMatch(/^\d+\.?\d*rem$/);
        }
      });
    });
  });

  describe('shadows', () => {
    it('should have shadow definitions', () => {
      expect(shadows.sm).toBeDefined();
      expect(shadows.base).toBeDefined();
      expect(shadows.md).toBeDefined();
      expect(shadows.lg).toBeDefined();
      expect(shadows.xl).toBeDefined();
    });

    it('should have valid shadow CSS format', () => {
      Object.values(shadows).forEach(shadow => {
        expect(shadow).toContain('rgb');
        expect(shadow).toContain('px');
      });
    });
  });

  describe('breakpoints', () => {
    it('should have breakpoint values', () => {
      expect(breakpoints.sm).toBe('640px');
      expect(breakpoints.md).toBe('768px');
      expect(breakpoints.lg).toBe('1024px');
      expect(breakpoints.xl).toBe('1280px');
      expect(breakpoints['2xl']).toBe('1536px');
    });

    it('should have valid px format', () => {
      Object.values(breakpoints).forEach(value => {
        expect(value).toMatch(/^\d+px$/);
      });
    });

    it('should be in ascending order', () => {
      const values = Object.values(breakpoints).map(v => parseInt(v));
      for (let i = 1; i < values.length; i++) {
        expect(values[i]).toBeGreaterThan(values[i - 1]);
      }
    });
  });
});
