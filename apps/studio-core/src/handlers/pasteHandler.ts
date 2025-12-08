// apps/studio-core/src/handlers/pasteHandler.ts
import React from 'react';
import type { ArchitectureJSON, ViewerInstance } from '@sruja/viewer';

export async function handlePasteNode(
  viewerRef: React.RefObject<ViewerInstance | null>,
  archData: ArchitectureJSON | null,
  selectedNodeId: string | null,
  syncDiagramToDsl: () => Promise<void>,
  setSelectedNodeId: (id: string | null) => void,
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void
) {
  if (!viewerRef.current || !archData) {
    setToast({ message: 'No architecture loaded', type: 'error' });
    return;
  }

  try {
    const clipboardText = await navigator.clipboard.readText();
    const nodeData = JSON.parse(clipboardText);

    if (!nodeData.id || !nodeData.node || !nodeData.type) {
      setToast({ message: 'Invalid clipboard data: Not a copied node', type: 'error' });
      return;
    }

    const originalId = nodeData.id;
    const node = nodeData.node;
    const nodeType = nodeData.type;

    // Generate a unique ID by appending a number
    const generateUniqueId = (baseId: string, type: string): string => {
      const parts = baseId.split('.');
      const baseName = parts[parts.length - 1];

      const checkExists = (id: string): boolean => {
        if (type === 'person') {
          return archData?.architecture?.persons?.some(p => p.id === id) || false;
        }
        if (type === 'system') {
          return archData?.architecture?.systems?.some(s => s.id === id) || false;
        }
        if (archData?.architecture?.systems) {
          for (const sys of archData.architecture.systems) {
            if (sys.containers?.some(c => c.id === baseName || `${sys.id}.${c.id}` === id)) return true;
            if (sys.datastores?.some(d => d.id === baseName || `${sys.id}.${d.id}` === id)) return true;
            if (sys.queues?.some(q => q.id === baseName || `${sys.id}.${q.id}` === id)) return true;
            if (sys.containers) {
              for (const c of sys.containers) {
                if (c.components?.some(comp => comp.id === baseName || `${sys.id}.${c.id}.${comp.id}` === id)) return true;
              }
            }
          }
        }
        return false;
      };

      let counter = 1;
      let newId = `${baseName}_copy`;

      if (type === 'person' || type === 'system') {
        while (checkExists(newId)) {
          newId = `${baseName}_copy${counter}`;
          counter++;
        }
        return newId;
      }

      if (originalId.includes('.')) {
        const originalParts = originalId.split('.');
        let parentSystemId: string | undefined;

        if (selectedNodeId && archData?.architecture?.systems?.some(s => s.id === selectedNodeId)) {
          parentSystemId = selectedNodeId;
        } else if (archData?.architecture?.systems && archData.architecture.systems.length > 0) {
          parentSystemId = archData.architecture.systems[0].id;
        }

        if (parentSystemId) {
          const childId = `${parentSystemId}.${newId}`;
          while (checkExists(childId)) {
            newId = `${baseName}_copy${counter}`;
            counter++;
          }
          return `${parentSystemId}.${newId}`;
        }
      }

      return newId;
    };

    const newId = generateUniqueId(originalId, nodeType);
    const label = node.label || node.title || newId.split('.').pop() || newId;

    if (!['person', 'system', 'container', 'datastore', 'queue', 'component', 'adr', 'requirement', 'deployment'].includes(nodeType)) {
      setToast({ message: `Cannot paste: Unsupported node type "${nodeType}"`, type: 'error' });
      return;
    }

    try {
      if (nodeType === 'person') {
        viewerRef.current.addNode('person', label);
      } else if (nodeType === 'system') {
        viewerRef.current.addNode('system', label);
      } else if (nodeType === 'container') {
        const parentId = newId.includes('.') ? newId.split('.')[0] : undefined;
        if (!parentId || !archData?.architecture?.systems?.some(s => s.id === parentId)) {
          setToast({ message: 'Cannot paste container: No valid parent system found', type: 'error' });
          return;
        }
        viewerRef.current.addNode('container', label, parentId);
      } else if (nodeType === 'datastore') {
        const parentId = newId.includes('.') ? newId.split('.')[0] : undefined;
        if (!parentId || !archData?.architecture?.systems?.some(s => s.id === parentId)) {
          setToast({ message: 'Cannot paste datastore: No valid parent system found', type: 'error' });
          return;
        }
        viewerRef.current.addNode('datastore', label, parentId);
      } else if (nodeType === 'queue') {
        const parentId = newId.includes('.') ? newId.split('.')[0] : undefined;
        if (!parentId || !archData?.architecture?.systems?.some(s => s.id === parentId)) {
          setToast({ message: 'Cannot paste queue: No valid parent system found', type: 'error' });
          return;
        }
        viewerRef.current.addNode('queue', label, parentId);
      } else if (nodeType === 'component') {
        const parts = newId.split('.');
        if (parts.length >= 2) {
          const parentId = `${parts[0]}.${parts[1]}`;
          const parentExists = archData?.architecture?.systems?.some(s =>
            s.id === parts[0] && s.containers?.some(c => c.id === parts[1])
          );
          if (!parentExists) {
            setToast({ message: 'Cannot paste component: Parent container does not exist', type: 'error' });
            return;
          }
          viewerRef.current.addNode('component', label, parentId);
        } else {
          setToast({ message: 'Cannot paste component: Invalid parent container path', type: 'error' });
          return;
        }
      } else if (nodeType === 'adr') {
        if (node.status && node.context && node.decision) {
          viewerRef.current.addNode('adr', node.title || label, undefined, {
            status: node.status,
            context: node.context,
            decision: node.decision,
            consequences: node.consequences
          });
        } else {
          viewerRef.current.addNode('adr', label);
        }
      } else if (nodeType === 'requirement') {
        viewerRef.current.addNode('requirement', label);
      } else if (nodeType === 'deployment') {
        viewerRef.current.addNode('deployment', label);
      }
    } catch (addError) {
      const errorMsg = addError instanceof Error ? addError.message : String(addError);
      setToast({ message: `Failed to paste node: ${errorMsg}`, type: 'error' });
      return;
    }

    await syncDiagramToDsl();

    setTimeout(() => {
      if (viewerRef.current) {
        viewerRef.current.selectNode(newId);
        setSelectedNodeId(newId);
      }
    }, 100);

    setToast({ message: `Pasted ${nodeType}: ${label}`, type: 'success' });
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    if (errorMsg.includes('JSON')) {
      setToast({ message: 'Failed to paste: Invalid clipboard data', type: 'error' });
    } else {
      setToast({ message: `Failed to paste: ${errorMsg}`, type: 'error' });
    }
  }
}

