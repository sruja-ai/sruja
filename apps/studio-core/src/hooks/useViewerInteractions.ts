// apps/studio-core/src/hooks/useViewerInteractions.ts
import { useEffect } from 'react';
import type React from 'react';
import type { ViewerInstance } from '@sruja/viewer';

interface UseViewerInteractionsOptions {
  viewerRef: React.RefObject<ViewerInstance | null>;
  setSelectedNodeId: (id: string) => void;
  setModalConfig: (config: any) => void;
  setContextMenu: (menu: { x: number; y: number; nodeId: string | null } | null) => void;
}

/**
 * Hook for viewer interaction handlers (double-click, context menu)
 */
export function useViewerInteractions({
  viewerRef,
  setSelectedNodeId,
  setModalConfig,
  setContextMenu,
}: UseViewerInteractionsOptions): void {
  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer || !viewer.cy) return;

    const cy = viewer.cy;

    // Double-click to rename (inline editing)
    const handleDoubleClick = (evt: any) => {
      const node = evt.target;
      if (node.isNode()) {
        const nodeId = node.id();
        setSelectedNodeId(nodeId);
        // Trigger rename modal
        setModalConfig({
          isOpen: true,
          title: 'Rename Node',
          placeholder: 'Enter new id...',
          type: 'rename',
          data: { oldId: nodeId },
        });
      }
    };

    // Right-click context menu
    const handleNodeContextMenu = (evt: any) => {
      evt.preventDefault();
      const node = evt.target;
      if (node.isNode()) {
        const nodeId = node.id();
        setSelectedNodeId(nodeId);
        // Get mouse position
        const originalEvent = evt.originalEvent || evt.cyEvent?.originalEvent;
        if (originalEvent) {
          setContextMenu({
            x: originalEvent.clientX,
            y: originalEvent.clientY,
            nodeId,
          });
        }
      }
    };

    const handleBackgroundContextMenu = (evt: any) => {
      if (evt.target === cy) {
        evt.preventDefault();
        // Right-click on background
        const originalEvent = evt.originalEvent || evt.cyEvent?.originalEvent;
        if (originalEvent) {
          setContextMenu({
            x: originalEvent.clientX,
            y: originalEvent.clientY,
            nodeId: null,
          });
        }
      }
    };

    cy.on('dbltap', 'node', handleDoubleClick);
    cy.on('cxttap', 'node', handleNodeContextMenu);
    cy.on('cxttap', handleBackgroundContextMenu);

    return () => {
      cy.removeListener('dbltap', 'node', handleDoubleClick);
      cy.removeListener('cxttap', 'node', handleNodeContextMenu);
      cy.removeListener('cxttap', handleBackgroundContextMenu);
    };
  }, [viewerRef, setSelectedNodeId, setModalConfig, setContextMenu]);
}
