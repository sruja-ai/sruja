// packages/shared/src/utils/__tests__/validation.test.ts
// Unit tests for validation utilities

import { describe, it, expect } from 'vitest';
import {
  isNonEmptyString,
  validateNonEmptyString,
  isValidElementId,
  validateElementId,
  isValidPercentage,
  validatePercentage,
  validatePositiveInteger,
} from '../validation';
import { ValidationError } from '../errors';

describe('isNonEmptyString', () => {
  it('should return true for non-empty strings', () => {
    expect(isNonEmptyString('hello')).toBe(true);
    expect(isNonEmptyString('a')).toBe(true);
    expect(isNonEmptyString('   trimmed   ')).toBe(true);
  });

  it('should return false for empty strings', () => {
    expect(isNonEmptyString('')).toBe(false);
  });

  it('should return false for non-string values', () => {
    expect(isNonEmptyString(null)).toBe(false);
    expect(isNonEmptyString(undefined)).toBe(false);
    expect(isNonEmptyString(123)).toBe(false);
    expect(isNonEmptyString({})).toBe(false);
    expect(isNonEmptyString([])).toBe(false);
  });
});

describe('validateNonEmptyString', () => {
  it('should return ok result for valid non-empty string', () => {
    const result = validateNonEmptyString('hello', 'name');
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe('hello');
    }
  });

  it('should return error for empty string', () => {
    const result = validateNonEmptyString('', 'name');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).toBeInstanceOf(ValidationError);
      expect(result.error.message).toContain('name');
    }
  });

  it('should return error for non-string values', () => {
    const result = validateNonEmptyString(null, 'name');
    expect(result.ok).toBe(false);
  });

  it('should use default field name', () => {
    const result = validateNonEmptyString('');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('value');
    }
  });
});

describe('isValidElementId', () => {
  it('should return true for valid element IDs', () => {
    expect(isValidElementId('system:MySystem')).toBe(true);
    expect(isValidElementId('container:api')).toBe(true);
    expect(isValidElementId('component_a')).toBe(true);
    expect(isValidElementId('element-123')).toBe(true);
    expect(isValidElementId('a')).toBe(true);
  });

  it('should return false for invalid characters', () => {
    expect(isValidElementId('system.MySystem')).toBe(false); // dot not allowed
    expect(isValidElementId('system MySystem')).toBe(false); // space not allowed
    expect(isValidElementId('system@MySystem')).toBe(false); // @ not allowed
    expect(isValidElementId('system#MySystem')).toBe(false); // # not allowed
  });

  it('should return false for empty strings', () => {
    expect(isValidElementId('')).toBe(false);
  });

  it('should return false for non-string values', () => {
    expect(isValidElementId(null)).toBe(false);
    expect(isValidElementId(123)).toBe(false);
  });
});

describe('validateElementId', () => {
  it('should return ok result for valid element ID', () => {
    const result = validateElementId('system:MySystem', 'elementId');
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe('system:MySystem');
    }
  });

  it('should return error for invalid characters', () => {
    const result = validateElementId('system.MySystem', 'elementId');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('invalid characters');
    }
  });

  it('should return error for empty string', () => {
    const result = validateElementId('', 'elementId');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('non-empty string');
    }
  });
});

describe('isValidPercentage', () => {
  it('should return true for valid percentages', () => {
    expect(isValidPercentage(0)).toBe(true);
    expect(isValidPercentage(50)).toBe(true);
    expect(isValidPercentage(100)).toBe(true);
    expect(isValidPercentage(0.5)).toBe(true);
    expect(isValidPercentage(99.99)).toBe(true);
  });

  it('should return false for values outside range', () => {
    expect(isValidPercentage(-1)).toBe(false);
    expect(isValidPercentage(101)).toBe(false);
    expect(isValidPercentage(-0.1)).toBe(false);
    expect(isValidPercentage(100.1)).toBe(false);
  });

  it('should return false for NaN', () => {
    expect(isValidPercentage(NaN)).toBe(false);
  });

  it('should return false for Infinity', () => {
    expect(isValidPercentage(Infinity)).toBe(false);
    expect(isValidPercentage(-Infinity)).toBe(false);
  });

  it('should return false for non-number values', () => {
    expect(isValidPercentage('50')).toBe(false);
    expect(isValidPercentage(null)).toBe(false);
  });
});

describe('validatePercentage', () => {
  it('should return ok result for valid percentage', () => {
    const result = validatePercentage(50, 'availability');
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(50);
    }
  });

  it('should return error for NaN', () => {
    const result = validatePercentage(NaN, 'percentage');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('number');
    }
  });

  it('should return error for Infinity', () => {
    const result = validatePercentage(Infinity, 'percentage');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('finite');
    }
  });

  it('should return error for values outside range', () => {
    const result = validatePercentage(150, 'percentage');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('between 0 and 100');
    }
  });

  it('should accept boundary values', () => {
    expect(validatePercentage(0, 'percentage').ok).toBe(true);
    expect(validatePercentage(100, 'percentage').ok).toBe(true);
  });
});

describe('validatePositiveInteger', () => {
  it('should return ok result for positive integers', () => {
    const result = validatePositiveInteger(1, 'count');
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(1);
    }

    const result2 = validatePositiveInteger(100, 'count');
    expect(result2.ok).toBe(true);
    if (result2.ok) {
      expect(result2.value).toBe(100);
    }
  });

  it('should return error for zero', () => {
    const result = validatePositiveInteger(0, 'count');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('positive integer');
    }
  });

  it('should return error for negative numbers', () => {
    const result = validatePositiveInteger(-1, 'count');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('positive integer');
    }
  });

  it('should return error for non-integers', () => {
    const result = validatePositiveInteger(1.5, 'count');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('integer');
    }
  });

  it('should return error for NaN', () => {
    const result = validatePositiveInteger(NaN, 'count');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('number');
    }
  });

  it('should return error for Infinity', () => {
    const result = validatePositiveInteger(Infinity, 'count');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.message).toContain('finite');
    }
  });

  it('should return error for non-number values', () => {
    const result = validatePositiveInteger('1', 'count');
    expect(result.ok).toBe(false);
  });
});

