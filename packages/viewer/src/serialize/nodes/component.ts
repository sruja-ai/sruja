// packages/viewer/src/serialize/nodes/component.ts
import type { Core, NodeSingular } from 'cytoscape';
import type { ArchitectureBody } from '../../types';
import { getMeta, stripPrefix, findSystem } from './helpers';

/**
 * Serialize component node (nested in container, which may be nested in system)
 */
export function serializeComponent(cy: Core, node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  const metadata = getMeta(node);

  if (!data.parent) return;

  const parentNode = cy.getElementById(data.parent);
  if (parentNode.data('type') !== 'container') return;

  const sysId = parentNode.data('parent');
  if (!sysId) return;

  const sys = findSystem(arch, sysId);
  if (!sys || !sys.containers) return;

  // Remove system prefix from container ID
  const contId = stripPrefix(parentNode.data('id'), sysId);
  const cont = sys.containers.find((c) => c.id === contId);
  if (!cont) return;

  // Ensure components array exists
  if (!cont.components) cont.components = [];

  // Remove container prefix from component ID
  const pureId = stripPrefix(data.id, parentNode.id());
  cont.components.push({
    id: pureId,
    label: data.label,
    metadata: metadata,
  });
}
