// apps/studio-core/src/handlers/explorerHandlers.ts
import type React from 'react';
import type { ViewerInstance, ArchitectureJSON } from '@sruja/viewer';

/**
 * Create handler for model explorer node selection
 */
export function createExplorerSelectHandler(
  viewerRef: React.RefObject<ViewerInstance | null>,
  archData: ArchitectureJSON | null,
  selectedNodeId: string | null,
  setSelectedNodeId: (id: string) => void,
  setDocumentation: (doc: any) => void,
  setZoomLevel: (level: number) => void,
  sidebar: { activePanel: string; showSidebar: boolean },
  setSidebar: (sidebar: any) => void
) {
  return (nodeId: string) => {
    if (!viewerRef.current || !archData) return;

    // Select node in viewer
    viewerRef.current.selectNode(nodeId);
    setSelectedNodeId(nodeId);

    // Update documentation if panel is open
    if (sidebar.activePanel === 'documentation') {
      // Documentation update logic would go here
      // This is handled by the documentation panel itself
    }

    // Update zoom level
    if (viewerRef.current.cy) {
      setZoomLevel(viewerRef.current.cy.zoom());
    }

    // Show sidebar if not visible
    if (!sidebar.showSidebar) {
      setSidebar({ ...sidebar, showSidebar: true });
    }
  };
}



