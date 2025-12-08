import { describe, it, expect } from 'vitest';
import { validateAdrData, validatePropertiesUpdate } from '../inputValidation';

describe('inputValidation ADR and properties', () => {
  it('validates ADR data and sanitizes fields', () => {
    const res = validateAdrData({
      title: '<b>Use React</b>',
      status: 'Accepted',
      context: '<script>ctx</script>',
      decision: 'Pick React',
      consequences: 'Train team',
    });
    expect(res.isValid).toBe(true);
    const json = JSON.parse(res.sanitized!);
    expect(json.title).toBe('Use React');
    expect(json.status).toBe('accepted');
    expect(json.context).toContain('&lt;script&gt;');
  });

  it('rejects invalid ADR status', () => {
    const res = validateAdrData({ title: 'X', status: 'unknown' } as any);
    expect(res.isValid).toBe(false);
    expect(res.error).toContain('Status must be one of');
  });

  it('sanitizes properties update strings and preserves numbers', () => {
    const res = validatePropertiesUpdate({ label: '<b>x</b>', count: 42 });
    expect(res.isValid).toBe(true);
    const json = JSON.parse(res.sanitized!);
    expect(json.label).toBe('&lt;b&gt;x&lt;&#x2F;b&gt;');
    expect(json.count).toBe(42);
  });
});
