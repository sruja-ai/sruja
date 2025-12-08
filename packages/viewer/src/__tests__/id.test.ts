import { describe, it, expect } from 'vitest';
import { resolveNodeId } from '../utils/id';

const ARCH: any = {
  architecture: {
    systems: [
      {
        id: 'Sys',
        containers: [
          { id: 'API', components: [{ id: 'Service' }] },
          { id: 'Web' },
        ],
      },
    ],
  },
};

describe('resolveNodeId', () => {
  it('returns direct id when present', () => {
    const ids = new Set(['User']);
    expect(resolveNodeId('User', ARCH.architecture, ids)).toBe('User');
  });

  it('resolves system/container/component qualified ids', () => {
    const ids = new Set(['Sys.API.Service', 'Sys.Web']);
    expect(resolveNodeId('Service', ARCH.architecture, ids)).toBe('Sys.API.Service');
    expect(resolveNodeId('Web', ARCH.architecture, ids)).toBe('Sys.Web');
  });

  it('returns null when not found', () => {
    const ids = new Set(['Sys.X']);
    expect(resolveNodeId('Y', ARCH.architecture, ids)).toBeNull();
  });
});
