import { describe, it, expect } from 'vitest';
import { calculateReadiness } from '../readiness';

describe('readiness', () => {
  it('returns empty report for null', () => {
    const r = calculateReadiness(null as any);
    expect(r.items.length).toBe(0);
    expect(r.overallScore).toBe(0);
  });

  it('adds items for missing container technology and component description', () => {
    const arch: any = {
      architecture: {
        persons: [{ id: 'User' }],
        systems: [{ id: 'Sys', containers: [{ id: 'Web', components: [{ id: 'API' }] }] }],
      },
    };
    const r = calculateReadiness(arch);
    expect(r.items.some(i => i.id.startsWith('cont-tech-'))).toBe(true);
    expect(r.items.some(i => i.id.startsWith('comp-desc-'))).toBe(true);
    expect(r.overallScore).toBeLessThanOrEqual(100);
    // Basics present, so score not capped below 40
    expect(r.overallScore).toBeGreaterThan(40);
  });
});
