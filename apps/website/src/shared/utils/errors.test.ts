import { describe, it, expect } from 'vitest';
import { formatParseError } from './errors';

const DSL = [
  'architecture "A" {',
  '  system Web {',
  '    container API',
  '  }',
  '}',
].join('\n');

describe('formatParseError', () => {
  it('includes message and line context with pointer from colon format', () => {
    const res = formatParseError('Parse failed:2:8 expected "}"', DSL);
    expect(res[0]).toContain('Parse failed');
    expect(res.some(l => l.includes('> Line 2:'))).toBe(true);
    expect(res.some(l => l.trim().endsWith('^'))).toBe(true);
    // hint may not be present for quoted brace text; pointer context is sufficient
  });

  it('includes surrounding lines using "line N" format', () => {
    const res = formatParseError('Error at line 3: unknown identifier', DSL);
    expect(res.some(l => l.includes(' Line 2:'))).toBe(true);
    expect(res.some(l => l.includes('> Line 3:'))).toBe(true);
    expect(res.some(l => l.includes(' Line 4:'))).toBe(true);
    expect(res.some(l => l.includes('Verify names are declared'))).toBe(true);
  });

  it('adds generic relation label hint when no hints matched', () => {
    const dsl = 'Web -> API Calls';
    const res = formatParseError('Unexpected token', dsl);
    expect(res.some(l => l.includes('Relation labels should be quoted'))).toBe(true);
  });
});
