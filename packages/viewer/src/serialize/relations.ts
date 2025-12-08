// packages/viewer/src/serialize/relations.ts
import type { Core } from 'cytoscape';
import type { ArchitectureBody } from '../types';

/**
 * Serialize edges from Cytoscape to ArchitectureJSON relations
 */
export function serializeRelations(
  cy: Core,
  arch: ArchitectureBody
): void {
  cy.edges().forEach(edge => {
    const data = edge.data();
    const sourceId = data.source;
    const targetId = data.target;
    const label = data.label;

    const sourceNode = cy.getElementById(sourceId);
    const sourceType = sourceNode.data('type');

    // Determine where to attach the relation
    let added = false;

    if (
      sourceType === 'system' ||
      sourceType === 'container' ||
      sourceType === 'component' ||
      sourceType === 'datastore' ||
      sourceType === 'queue'
    ) {
      // Try to find the system this node belongs to
      let sysId = sourceType === 'system' ? sourceId : sourceNode.data('parent');
      // If parent is container, go up one more
      if (sysId && cy.getElementById(sysId).data('type') === 'container') {
        sysId = cy.getElementById(sysId).data('parent');
      }

      // If we found a system, add relation there
      const sys = arch.systems?.find(s => s.id === sysId);
      if (sys) {
        if (!sys.relations) sys.relations = [];
        // Use relative references if possible
        const relSource =
          sourceId === sysId
            ? '.'
            : sourceId.startsWith(sysId + '.')
              ? sourceId.slice(sysId.length + 1)
              : sourceId;

        let relTarget = targetId;
        if (targetId.startsWith(sysId + '.')) {
          relTarget = targetId.slice(sysId.length + 1);
        }

        sys.relations.push({
          from: relSource,
          to: relTarget,
          label
        });
        added = true;
      }
    }

    if (!added) {
      if (!arch.relations) arch.relations = [];
      arch.relations.push({
        from: sourceId,
        to: targetId,
        label
      });
    }
  });
}
