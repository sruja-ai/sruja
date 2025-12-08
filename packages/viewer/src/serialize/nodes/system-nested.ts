// packages/viewer/src/serialize/nodes/system-nested.ts
import type { Core, NodeSingular } from 'cytoscape';
import type { ArchitectureBody } from '../../types';
import { getMeta, stripPrefix, findSystem } from './helpers';

/**
 * Serialize container node
 */
export function serializeContainer(cy: Core, node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  const metadata = getMeta(node);

  // Check if it belongs to a system
  if (data.parent && cy.getElementById(data.parent).data('type') === 'system') {
    const sys = findSystem(arch, data.parent);
    if (sys) {
      if (!sys.containers) sys.containers = [];
      const pureId = stripPrefix(data.id, data.parent);
      sys.containers.push({
        id: pureId,
        label: data.label,
        metadata: metadata,
      });
    }
  } else {
    // Top-level container
    if (!arch.containers) arch.containers = [];
    arch.containers.push({
      id: data.id,
      label: data.label,
      metadata: metadata,
    });
  }
}

/**
 * Serialize datastore node
 */
export function serializeDatastore(cy: Core, node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  const metadata = getMeta(node);

  // Check if it belongs to a system
  if (data.parent && cy.getElementById(data.parent).data('type') === 'system') {
    const sys = findSystem(arch, data.parent);
    if (sys) {
      if (!sys.datastores) sys.datastores = [];
      const pureId = stripPrefix(data.id, data.parent);
      sys.datastores.push({
        id: pureId,
        label: data.label,
        metadata: metadata,
      });
    }
  } else {
    // Top-level datastore
    if (!arch.datastores) arch.datastores = [];
    arch.datastores.push({
      id: data.id,
      label: data.label,
      metadata: metadata,
    });
  }
}

/**
 * Serialize queue node
 */
export function serializeQueue(cy: Core, node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  const metadata = getMeta(node);

  // Check if it belongs to a system
  if (data.parent && cy.getElementById(data.parent).data('type') === 'system') {
    const sys = findSystem(arch, data.parent);
    if (sys) {
      if (!sys.queues) sys.queues = [];
      const pureId = stripPrefix(data.id, data.parent);
      sys.queues.push({
        id: pureId,
        label: data.label,
        metadata: metadata,
      });
    }
  } else {
    // Top-level queue
    if (!arch.queues) arch.queues = [];
    arch.queues.push({
      id: data.id,
      label: data.label,
      metadata: metadata,
    });
  }
}
