// packages/ui/src/utils/cn.test.ts
import { describe, it, expect } from 'vitest';
import { cn } from './cn';

describe('cn', () => {
  it('should merge multiple class strings', () => {
    const result = cn('class1', 'class2', 'class3');
    expect(result).toContain('class1');
    expect(result).toContain('class2');
    expect(result).toContain('class3');
  });

  it('should filter out falsy values', () => {
    const result = cn('class1', undefined, null, false, 'class2');
    expect(result).not.toContain('undefined');
    expect(result).not.toContain('null');
    expect(result).not.toContain('false');
    expect(result).toContain('class1');
    expect(result).toContain('class2');
  });

  it('should handle empty arrays', () => {
    const result = cn();
    expect(result).toBe('');
  });

  it('should handle only falsy values', () => {
    const result = cn(undefined, null, false);
    expect(result).toBe('');
  });

  it('should merge Tailwind classes correctly', () => {
    // twMerge should handle conflicting classes
    const result = cn('p-4', 'p-6');
    // Should only keep the last conflicting class
    expect(result).toBe('p-6');
  });

  it('should handle mixed valid and invalid classes', () => {
    const result = cn('valid-class', undefined, 'another-valid', null);
    expect(result).toContain('valid-class');
    expect(result).toContain('another-valid');
  });

  it('should handle single class', () => {
    const result = cn('single-class');
    expect(result).toBe('single-class');
  });

  it('should merge classes with spaces', () => {
    const result = cn('class1 class2', 'class3');
    expect(result).toContain('class1');
    expect(result).toContain('class2');
    expect(result).toContain('class3');
  });
});
