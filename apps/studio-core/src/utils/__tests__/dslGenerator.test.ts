import { describe, it, expect } from 'vitest';
import { generateDSLFragment } from '../dslGenerator';

describe('dslGenerator', () => {
  it('returns empty string for null node', () => {
    expect(generateDSLFragment(null as any)).toBe('');
  });

  it('generates minimal system block without body', () => {
    const dsl = generateDSLFragment({ type: 'system', id: 'Sys' });
    expect(dsl).toBe('system Sys');
  });

  it('generates container with label, technology, description, and metadata', () => {
    const node = {
      type: 'container',
      id: 'Sys.Web',
      label: 'Web',
      description: 'UI Layer',
      technology: 'React',
      metadata: [{ key: 'owner', value: 'Team' }],
    };
    const dsl = generateDSLFragment(node);
    expect(dsl).toContain('container Sys.Web "Web" {');
    expect(dsl).toContain('description "UI Layer"');
    expect(dsl).toContain('technology "React"');
    expect(dsl).toContain('metadata {');
    expect(dsl).toContain('owner "Team"');
  });
});
