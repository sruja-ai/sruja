// packages/viewer/src/serialize/nodes/index.ts
import type { Core } from 'cytoscape';
import type { ArchitectureBody } from '../../types';
import { serializePerson } from './top-level';
import { serializeSystem } from './top-level';
import { serializeRequirement } from './top-level';
import { serializeADR } from './top-level';
import { serializeDeployment } from './top-level';
import { serializeContainer } from './system-nested';
import { serializeDatastore } from './system-nested';
import { serializeQueue } from './system-nested';
import { serializeComponent } from './component';

/**
 * Serialize nodes from Cytoscape to ArchitectureJSON structure
 */
export function serializeNodes(cy: Core, arch: ArchitectureBody): void {
  cy.nodes().forEach((node) => {
    const type = node.data('type');

    switch (type) {
      case 'person':
        serializePerson(node, arch);
        break;
      case 'system':
        serializeSystem(node, arch);
        break;
      case 'container':
        serializeContainer(cy, node, arch);
        break;
      case 'component':
        serializeComponent(cy, node, arch);
        break;
      case 'datastore':
        serializeDatastore(cy, node, arch);
        break;
      case 'queue':
        serializeQueue(cy, node, arch);
        break;
      case 'requirement':
        serializeRequirement(node, arch);
        break;
      case 'adr':
        serializeADR(node, arch);
        break;
      case 'deployment':
        serializeDeployment(node, arch);
        break;
    }
  });
}
