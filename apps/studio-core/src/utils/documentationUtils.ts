// apps/studio-core/src/utils/documentationUtils.ts
import type { ArchitectureJSON } from '@sruja/viewer';
import { findNodeInArch, getNodeType } from './nodeUtils';
import type { DocumentationState } from '../context/StudioStateContext';

export function updateDocumentationForNode(
  nodeId: string | null,
  archData: ArchitectureJSON | null,
  setDocumentation: (state: DocumentationState | ((prev: DocumentationState) => DocumentationState)) => void
) {
  if (!nodeId || !archData) {
    setDocumentation({
      selectedNodeType: null,
      selectedNodeId: undefined,
      selectedNodeLabel: undefined,
    });
    return;
  }

  const node = findNodeInArch(archData, nodeId);
  if (node) {
    const nodeType = getNodeType(node, archData);
    setDocumentation({
      selectedNodeType: nodeType,
      selectedNodeId: nodeId,
      selectedNodeLabel: node.label || nodeId,
    });
  }
}








