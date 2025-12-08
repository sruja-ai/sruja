// apps/studio-core/src/handlers/monacoHandlers.ts
import React from 'react';
import { parseDslForNodeId, findNodeInArch } from '../utils/nodeUtils';
import type { ArchitectureJSON, ViewerInstance } from '@sruja/viewer';
import { updateDocumentationForNode } from '../utils/documentationUtils';
import type { DocumentationState, SidebarState } from '../context/StudioStateContext';

export function createMonacoSelectionHandler(
  viewerRef: React.RefObject<ViewerInstance | null>,
  archData: ArchitectureJSON | null,
  dsl: string,
  selectedNodeId: string | null,
  setSelectedNodeId: (id: string | null) => void,
  setDocumentation: (state: DocumentationState | ((prev: DocumentationState) => DocumentationState)) => void,
  sidebar: SidebarState,
  setSidebar: (state: SidebarState | ((prev: SidebarState) => SidebarState)) => void
) {
  return (editor: any) => {
    if (!editor || !viewerRef.current || !archData) return;

    const selection = editor.getSelection();
    if (!selection || selection.isEmpty()) {
      // Clear selection if nothing is selected
      if (selectedNodeId && viewerRef.current.cy) {
        viewerRef.current.cy.getElementById(selectedNodeId).removeClass('highlighted');
      }
      return;
    }

    const model = editor.getModel();
    const startLine = selection.startLineNumber - 1;
    const startColumn = selection.startColumn - 1;
    const selectedText = model.getValueInRange(selection);

    // Try to find node ID in selected text or at cursor
    let nodeId: string | null = null;

    // First try the selected text
    if (selectedText.trim()) {
      const trimmed = selectedText.trim();
      // Check if it's a direct node ID
      if (findNodeInArch(archData, trimmed)) {
        nodeId = trimmed;
      } else {
        // Try parsing from DSL syntax
        nodeId = parseDslForNodeId(selectedText, 0, trimmed.length / 2, archData);
      }
    }

    // If no node found in selection, try at cursor position
    if (!nodeId) {
      nodeId = parseDslForNodeId(dsl, startLine, startColumn, archData);
    }

    if (nodeId && viewerRef.current.cy) {
      // Clear previous highlight
      if (selectedNodeId) {
        const prevNode = viewerRef.current.cy.getElementById(selectedNodeId);
        if (prevNode.length > 0) {
          prevNode.removeClass('highlighted');
          prevNode.style('border-width', '');
          prevNode.style('border-color', '');
        }
      }

      // Highlight new node
      const node = viewerRef.current.cy.getElementById(nodeId);
      if (node.length > 0) {
        node.addClass('highlighted');
        node.style('border-width', '3px');
        node.style('border-color', (getComputedStyle(document.documentElement).getPropertyValue('--color-primary') || '#3b82f6').trim());
        node.style('z-index', '10');
        viewerRef.current.cy.animate({
          center: { eles: node },
          zoom: Math.min(viewerRef.current.cy.zoom() * 1.2, 2),
        }, { duration: 300 });
        setSelectedNodeId(nodeId);

        // Update documentation state
        updateDocumentationForNode(nodeId, archData, setDocumentation);
        // Auto-switch to documentation panel if sidebar is open
        if (sidebar.showSidebar && sidebar.activePanel !== 'documentation') {
          setSidebar((prev) => ({ ...prev, activePanel: 'documentation' }));
        }
      }
    }
  };
}
