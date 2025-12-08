import { describe, it, expect } from 'vitest';
import { getStyleValue, calculateNodeWidth, calculateNodeHeight } from '../style/helpers';

function makeNode(data: Record<string, any>) {
  return {
    data: (key: string) => data[key],
  } as any;
}

describe('style helpers', () => {
  it('getStyleValue returns defaults when metadata missing or empty', () => {
    const node = makeNode({ metadata: [{ key: 'style.icon', value: '' }] });
    expect(getStyleValue(node, 'style.icon', 'default')).toBe('default');
    const node2 = makeNode({});
    expect(getStyleValue(node2, 'style.icon', 'default')).toBe('default');
  });

  it('calculateNodeWidth accounts for type padding and icon presence', () => {
    const base = makeNode({ label: 'Hello World', type: 'container', metadata: [{ key: 'style.icon', value: '' }] });
    const withIcon = makeNode({ label: 'Hello World', type: 'container', metadata: [{ key: 'style.icon', value: 'db' }] });
    const w1 = calculateNodeWidth(base);
    const w2 = calculateNodeWidth(withIcon);
    expect(w2).toBeGreaterThan(w1);
    const sys = makeNode({ label: 'Sys', type: 'system' });
    expect(calculateNodeWidth(sys)).toBeGreaterThanOrEqual(120);
  });

  it('calculateNodeHeight wraps text and applies type-specific padding', () => {
    const longLabel = 'This is a very long label that should wrap across lines';
    const person = makeNode({ label: longLabel, type: 'person' });
    const container = makeNode({ label: longLabel, type: 'container' });
    const h1 = calculateNodeHeight(person);
    const h2 = calculateNodeHeight(container);
    expect(h1).toBeGreaterThan(h2); // person has larger padding
    expect(h2).toBeGreaterThanOrEqual(40);
  });
});
