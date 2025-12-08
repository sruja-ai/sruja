// apps/studio-core/src/utils/nodeUtils.ts
import { findNode } from './archUtils';
import type { ArchitectureJSON } from '@sruja/viewer';

export function findNodeInArch(data: ArchitectureJSON, id: string): any {
  return findNode(data, id);
}

export function getNodeType(node: any, data?: ArchitectureJSON): string {
  if (node.type) return node.type;
  const arch = data?.architecture;

  // Infer from structure
  if ('containers' in node || 'datastores' in node || 'queues' in node) return 'system';
  if ('components' in node) return 'container';

  if (node.id && node.id.includes('.')) {
    const parts = node.id.split('.');
    if (parts.length === 2 && arch?.systems) {
      // Could be container, datastore, or queue
      for (const sys of arch.systems) {
        if (sys.id === parts[0]) {
          if (sys.containers?.some(c => c.id === parts[1])) return 'container';
          if (sys.datastores?.some(d => d.id === parts[1])) return 'datastore';
          if (sys.queues?.some(q => q.id === parts[1])) return 'queue';
        }
      }
    } else if (parts.length === 3) {
      return 'component';
    }
  }

  // Check if it's a person
  if (arch?.persons?.some(p => p.id === node.id)) return 'person';

  // Check if it's a system (must check before requirements/ADRs)
  if (arch?.systems?.some(s => s.id === node.id)) return 'system';

  // Check requirements and ADRs
  if (arch?.requirements?.some(r => r.id === node.id)) return 'requirement';
  if (arch?.adrs?.some(a => a.id === node.id)) return 'adr';
  if (arch?.deployment?.some(d => d.id === node.id)) return 'deployment';

  return 'system';
}

export function parseDslForNodeId(
  text: string,
  cursorLine: number,
  cursorColumn: number,
  archData: ArchitectureJSON | null
): string | null {
  const lines = text.split('\n');
  const currentLine = lines[cursorLine] || '';

  // Try to find node ID patterns around cursor
  // Patterns: "id", "id:", "id =", "id:", etc.
  const patterns = [
    /(\w+)\s*[:=]/g,  // id: or id =
    /(\w+)\s*$/g,      // id at end of line
  ];

  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(currentLine)) !== null) {
      const start = match.index;
      const end = start + match[1].length;
      if (cursorColumn >= start && cursorColumn <= end) {
        const nodeId = match[1];
        // Check if this ID exists in the architecture
        if (archData && findNodeInArch(archData, nodeId)) {
          return nodeId;
        }
      }
    }
  }

  // Try to find in relation syntax: "from" or "to"
  const relationPattern = /(?:from|to)\s+([\w.]+)/g;
  let match;
  while ((match = relationPattern.exec(currentLine)) !== null) {
    const start = match.index;
    const end = start + match[0].length;
    if (cursorColumn >= start && cursorColumn <= end) {
      const nodeId = match[1];
      if (archData && findNodeInArch(archData, nodeId)) {
        return nodeId;
      }
    }
  }

  return null;
}









