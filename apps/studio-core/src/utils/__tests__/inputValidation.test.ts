import { describe, it, expect } from 'vitest';
import {
  validateNodeId,
  validateNodeLabel,
  sanitizeText,
  validateDslInput,
  validateSearchQuery,
  validateUrl,
  validateNodeType,
  validateRelationLabel,
} from '../inputValidation';

describe('inputValidation', () => {
  it('validates node ID rules', () => {
    expect(validateNodeId('my-system').isValid).toBe(true);
    expect(validateNodeId('  my_system  ').sanitized).toBe('my_system');
    expect(validateNodeId('')).toEqual({ isValid: false, error: 'Node ID is required' });
    expect(validateNodeId('a'.repeat(101)).isValid).toBe(false);
    expect(validateNodeId('invalid!').isValid).toBe(false);
  });

  it('sanitizes and validates node labels', () => {
    const res = validateNodeLabel('<script>x</script>Label');
    expect(res.isValid).toBe(true);
    expect(res.sanitized).toBe('xLabel');
    expect(validateNodeLabel(' ').isValid).toBe(false);
    expect(validateNodeLabel('a'.repeat(201)).isValid).toBe(false);
  });

  it('escapes HTML in sanitizeText', () => {
    expect(sanitizeText('<script>"&</script>')).toBe('&lt;script&gt;&quot;&amp;&lt;&#x2F;script&gt;');
    expect(sanitizeText('')).toBe('');
  });

  it('validates DSL input length and trimming', () => {
    const ok = validateDslInput('architecture "A" {}');
    expect(ok.isValid).toBe(true);
    expect(ok.sanitized).toBe('architecture "A" {}');
    const tooLong = validateDslInput('a'.repeat(10 * 1024 * 1024 + 1));
    expect(tooLong.isValid).toBe(false);
  });

  it('validates search query with sanitization', () => {
    const res = validateSearchQuery('x < y');
    expect(res.isValid).toBe(true);
    expect(res.sanitized).toBe('x &lt; y');
    const empty = validateSearchQuery('');
    expect(empty.isValid).toBe(true);
    expect(empty.sanitized).toBe('');
  });

  it('validates URLs with protocols', () => {
    expect(validateUrl('https://example.com').isValid).toBe(true);
    expect(validateUrl('ftp://example.com').isValid).toBe(false);
    expect(validateUrl('not-a-url').isValid).toBe(false);
  });

  it('validates node types via type guard', () => {
    expect(validateNodeType('system')).toBe(true);
    expect(validateNodeType('unknown')).toBe(false);
  });

  it('validates relation label length and sanitization', () => {
    const res = validateRelationLabel('hello <b>world</b>');
    expect(res.isValid).toBe(true);
    expect(res.sanitized).toBe('hello &lt;b&gt;world&lt;&#x2F;b&gt;');
    expect(validateRelationLabel('a'.repeat(201)).isValid).toBe(false);
    const empty = validateRelationLabel('');
    expect(empty.isValid).toBe(true);
    expect(empty.sanitized).toBe('');
  });
});
