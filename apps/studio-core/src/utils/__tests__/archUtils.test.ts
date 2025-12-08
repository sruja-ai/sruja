import { describe, it, expect } from 'vitest';
import { findNode, updateNode } from '../archUtils';

const ARCH: any = {
  architecture: {
    persons: [{ id: 'User' }],
    systems: [
      {
        id: 'ECommerce',
        containers: [
          { id: 'WebApp', components: [{ id: 'API' }] },
        ],
        datastores: [{ id: 'DB' }],
        queues: [{ id: 'Events' }],
      },
    ],
  },
};

describe('archUtils', () => {
  it('findNode resolves qualified nodes and raw ids', () => {
    expect(findNode(ARCH, 'ECommerce')?.id).toBe('ECommerce');
    expect(findNode(ARCH, 'ECommerce.WebApp')?.id).toBe('WebApp');
    expect(findNode(ARCH, 'ECommerce.WebApp.API')?.id).toBe('API');
    expect(findNode(ARCH, 'ECommerce.DB')?.id).toBe('DB');
    expect(findNode(ARCH, 'User')?.id).toBe('User');
    expect(findNode(ARCH, 'Missing')).toBeNull();
  });

  it('updateNode deep-clones and updates target by qualified id', () => {
    const updated = updateNode(ARCH, 'ECommerce.WebApp', (node: any) => ({ ...node, label: 'Web' }));
    const originalCont = ARCH.architecture.systems[0].containers[0];
    const updatedCont = updated.architecture.systems[0].containers[0];
    expect(updatedCont.label).toBe('Web');
    expect(originalCont.label).toBeUndefined();
  });
});
