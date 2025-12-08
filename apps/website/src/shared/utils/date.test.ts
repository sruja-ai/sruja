// apps/website/src/shared/utils/date.test.ts
import { describe, it, expect } from 'vitest';
import { formatDate, formatDateISO } from './date';

describe('date utilities', () => {
  const testDate = new Date('2024-01-15T10:30:00Z');

  describe('formatDate', () => {
    it('formats date in short format by default', () => {
      const result = formatDate(testDate);
      expect(result).toMatch(/Jan/);
      expect(result).toMatch(/2024/);
    });

    it('formats date in long format', () => {
      const result = formatDate(testDate, 'long');
      expect(result).toMatch(/January/);
      expect(result).toMatch(/2024/);
    });

    it('formats date in ISO format', () => {
      const result = formatDate(testDate, 'iso');
      expect(result).toMatch(/2024-01-15/);
    });

    it('handles string dates', () => {
      const result = formatDate('2024-01-15');
      expect(result).toMatch(/Jan/);
    });

    it('handles invalid dates', () => {
      const result = formatDate('invalid-date');
      expect(result).toBe('Invalid Date');
    });
  });

  describe('formatDateISO', () => {
    it('formats date as ISO string', () => {
      const result = formatDateISO(testDate);
      expect(result).toMatch(/2024-01-15/);
    });

    it('returns empty string for invalid dates', () => {
      const result = formatDateISO('invalid-date');
      expect(result).toBe('');
    });
  });
});
