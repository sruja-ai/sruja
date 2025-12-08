// apps/studio-core/src/handlers/nodeHandlers.ts
import type React from 'react';
import type { ViewerInstance, ArchitectureJSON } from '@sruja/viewer';

interface NodeHandlersOptions {
  viewerRef: React.RefObject<ViewerInstance | null>;
  selectedNodeId: string | null;
  archData: ArchitectureJSON | null;
  copiedNode: { id: string; data: any; type: string } | null;
  setCopiedNode: (node: { id: string; data: any; type: string } | null) => void;
  setSelectedNodeId: (id: string) => void;
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void;
  syncDiagramToDslState: () => Promise<void>;
}

/**
 * Find node in architecture by ID
 */
function findNodeInArch(arch: ArchitectureJSON, id: string): any {
  const archBody = arch.architecture || {};
  if (archBody.persons) {
    const found = archBody.persons.find((p: any) => p.id === id);
    if (found) return found;
  }
  if (archBody.systems) {
    for (const system of archBody.systems) {
      if (system.id === id) return system;
      if (system.containers) {
        for (const container of system.containers) {
          if (container.id === id) return container;
          if (container.components) {
            const found = container.components.find((c: any) => c.id === id);
            if (found) return found;
          }
        }
      }
    }
  }
  return null;
}

/**
 * Handle node deletion
 */
export function createHandleDelete({
  viewerRef,
  syncDiagramToDslState,
  setToast,
}: Pick<NodeHandlersOptions, 'viewerRef' | 'syncDiagramToDslState' | 'setToast'>) {
  return async () => {
    if (viewerRef.current) {
      viewerRef.current.removeSelected();
      await syncDiagramToDslState();
      setToast({ message: 'Deleted selected elements', type: 'success' });
    }
  };
}

/**
 * Handle node copy
 */
export function createHandleCopy({
  viewerRef,
  selectedNodeId,
  archData,
  setCopiedNode,
  setToast,
}: Pick<NodeHandlersOptions, 'viewerRef' | 'selectedNodeId' | 'archData' | 'setCopiedNode' | 'setToast'>) {
  return () => {
    if (!selectedNodeId || !archData || !viewerRef.current?.cy) {
      setToast({ message: 'Select a node to copy', type: 'info' });
      return;
    }

    const cy = viewerRef.current.cy;
    const node = cy.getElementById(selectedNodeId);
    if (node.length > 0) {
      const nodeData = node.data();
      const nodeType = nodeData.type;

      const fullNodeData = findNodeInArch(archData, selectedNodeId);
      if (fullNodeData) {
        setCopiedNode({
          id: selectedNodeId,
          data: fullNodeData,
          type: nodeType,
        });
        setToast({ message: 'Node copied', type: 'success' });
      }
    }
  };
}

/**
 * Handle node paste
 */
export function createHandlePaste({
  viewerRef,
  copiedNode,
  setSelectedNodeId,
  syncDiagramToDslState,
  setToast,
}: Pick<NodeHandlersOptions, 'viewerRef' | 'copiedNode' | 'setSelectedNodeId' | 'syncDiagramToDslState' | 'setToast'>) {
  return async () => {
    if (!copiedNode || !viewerRef.current?.cy) {
      setToast({ message: 'Nothing to paste', type: 'info' });
      return;
    }

    const cy = viewerRef.current.cy;

    // Generate new ID
    const baseName = copiedNode.data.label || copiedNode.data.id;
    const newId = `${baseName}Copy${Date.now().toString().slice(-4)}`.replace(/\s+/g, '');

    // Ensure unique ID
    let uniqueId = newId;
    let counter = 1;
    while (cy.getElementById(uniqueId).length > 0) {
      uniqueId = `${newId}${counter}`;
      counter++;
    }

    // Get current pan/zoom to position pasted node
    const pan = cy.pan();
    const zoom = cy.zoom();
    const centerX = (cy.width() / 2 - pan.x) / zoom;
    const centerY = (cy.height() / 2 - pan.y) / zoom;

    // Create new node with copied data
    const newNodeData = {
      ...copiedNode.data,
      id: uniqueId,
      label: `${copiedNode.data.label || copiedNode.data.id} Copy`,
    };

    // Add node to viewer
    if (viewerRef.current) {
      viewerRef.current.addNode(copiedNode.type as any, newNodeData.label, undefined, newNodeData);

      // Position the new node at center + offset
      setTimeout(async () => {
        const newNode = cy.getElementById(uniqueId);
        if (newNode.length > 0) {
          newNode.position({ x: centerX + 100, y: centerY + 100 });
          newNode.select();
          setSelectedNodeId(uniqueId);
          await syncDiagramToDslState();
          setToast({ message: 'Node pasted', type: 'success' });
        }
      }, 100);
    }
  };
}

/**
 * Handle adding a new node
 */
export function createHandleAddNode({
  viewerRef,
  setSelectedNodeId,
  syncDiagramToDslState,
  setToast,
  setAdrModalOpen,
}: Pick<NodeHandlersOptions, 'viewerRef' | 'setSelectedNodeId' | 'syncDiagramToDslState' | 'setToast'> & {
  setAdrModalOpen: (open: boolean) => void;
}) {
  return async (type: 'person' | 'system' | 'container' | 'component' | 'datastore' | 'queue' | 'requirement' | 'adr' | 'deployment') => {
    if (type === 'adr') {
      setAdrModalOpen(true);
      return;
    }

    // Check if a node is selected to add as child
    const selected = viewerRef.current?.cy?.nodes(':selected');
    let parentId: string | undefined;

    if (selected && selected.length === 1) {
      const parentType = selected.data('type');
      if (parentType === 'system' && ['container', 'datastore', 'queue'].includes(type)) {
        parentId = selected.first().id();
      } else if (parentType === 'container' && type === 'component') {
        parentId = selected.first().id();
      }
    }

    // Generate a default name based on type
    const baseName = type.charAt(0).toUpperCase() + type.slice(1);
    const defaultName = `${baseName}${Date.now().toString().slice(-4)}`;

    // Add node directly to canvas
    if (viewerRef.current && viewerRef.current.cy) {
      const cy = viewerRef.current.cy;

      // Generate the ID that will be used (same logic as viewer.addNode)
      let id = defaultName.replace(/\s+/g, '');
      if (parentId) {
        id = `${parentId}.${id}`;
      }

      // Ensure unique ID (same logic as viewer.addNode)
      let uniqueId = id;
      let counter = 1;
      while (cy.getElementById(uniqueId).length > 0) {
        uniqueId = `${id}${counter}`;
        counter++;
      }

      // Add node
      viewerRef.current.addNode(type, defaultName, parentId);

      // Wait for node to be added and layout to run, then select it
      setTimeout(async () => {
        const newNode = cy.getElementById(uniqueId);
        if (newNode && newNode.length > 0) {
          // Select the new node
          cy.nodes().unselect();
          newNode.select();
          setSelectedNodeId(uniqueId);

          // Sync to DSL
          await syncDiagramToDslState();

          // Show toast (brief, non-intrusive)
          setToast({ message: `Added ${type}`, type: 'success' });
        }
      }, 600); // Wait for layout animation to complete
    }
  };
}



