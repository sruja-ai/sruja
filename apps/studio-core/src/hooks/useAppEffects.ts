// apps/studio-core/src/hooks/useAppEffects.ts
/**
 * @fileoverview Custom hook for managing all application side effects (useEffect).
 * Consolidates all useEffect logic that was previously in App.tsx.
 */
import { useEffect, useRef } from 'react';
import type { ArchitectureJSON, ViewerInstance } from '@sruja/viewer';
import type { WasmApi } from '@sruja/shared';
import { logger } from '@sruja/shared';
import { updateDocumentationForNode } from '../utils/documentationUtils';
import { useAutosave } from './useAutosave';
import { persistence } from '../utils/persistence';
import type { DocumentationState } from '../context/StudioStateContext';

interface UseAppEffectsOptions {
  // Refs
  containerRef: React.RefObject<HTMLDivElement | null>;
  viewerRef: React.RefObject<ViewerInstance | null>;
  wasmApiRef: React.RefObject<WasmApi | null>;
  isUpdatingViewerRef: React.MutableRefObject<boolean>;
  isUpdatingPropertiesRef: React.MutableRefObject<boolean>;

  // State
  dsl: string;
  archData: ArchitectureJSON | null;
  selectedNodeId: string | null;
  sidebar: {
    activePanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide';
  };

  // Setters
  updateDsl: (newDsl: string) => Promise<void>;
  setSelectedNodeId: (id: string | null) => void;
  setArchData: (data: ArchitectureJSON | null) => void;
  setZoomLevel: (level: number) => void;
  setLastSaved: (date: Date | null) => void;
  setDocumentation: (doc: DocumentationState | ((prev: DocumentationState) => DocumentationState)) => void;
  setCommandPaletteOpen: (open: boolean) => void;
}

/**
 * Custom hook that consolidates all application side effects (useEffect).
 * 
 * This hook centralizes all useEffect logic that was previously in App.tsx,
 * including deep linking, keyboard shortcuts, and state synchronization.
 * 
 * @param options - Configuration object containing refs, state, and setters
 * 
 * @example
 * ```typescript
 * useAppEffects({
 *   viewerRef,
 *   wasmApiRef,
 *   // ... other options
 * });
 * ```
 */
export function useAppEffects(options: UseAppEffectsOptions) {
  const {
    containerRef,
    viewerRef,
    wasmApiRef,
    isUpdatingViewerRef,
    isUpdatingPropertiesRef,
    dsl,
    archData,
    selectedNodeId,
    sidebar,
    updateDsl,
    setSelectedNodeId,
    setArchData,
    setZoomLevel,
    setLastSaved,
    setDocumentation,
    setCommandPaletteOpen,
  } = options;

  // Drag prevention for viewer
  useEffect(() => {
    if (!containerRef.current) return;
    const preventDrag = (e: Event) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    };
    const viewerElement = containerRef.current;
    viewerElement.addEventListener('dragstart', preventDrag);
    viewerElement.addEventListener('drag', preventDrag);
    const setupSvgDragPrevention = (svg: SVGElement) => {
      svg.setAttribute('draggable', 'false');
      svg.addEventListener('dragstart', preventDrag);
    };
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeName === 'svg') {
            setupSvgDragPrevention(node as SVGElement);
          } else if (node.nodeType === Node.ELEMENT_NODE) {
            const svgs = (node as Element).querySelectorAll('svg');
            svgs.forEach(setupSvgDragPrevention);
          }
        });
      });
    });
    observer.observe(viewerElement, { childList: true, subtree: true });
    return () => {
      viewerElement.removeEventListener('dragstart', preventDrag);
      viewerElement.removeEventListener('drag', preventDrag);
      observer.disconnect();
    };
  }, [containerRef]);

  // Keep documentation panel in sync when DSL/architecture updates
  useEffect(() => {
    if (sidebar.activePanel === 'documentation') {
      updateDocumentationForNode(selectedNodeId, archData, setDocumentation);
    }
  }, [archData, selectedNodeId, sidebar.activePanel, setDocumentation]);

  // Reload viewer when archData changes (e.g., after adding nodes via syncDiagramToDslState)
  // This is a fallback for when archData changes directly without going through updateViewer
  // NOTE: This should NOT trigger when updateViewer is called, as that already calls viewer.load()
  // NOTE: This should NOT trigger when updating properties, as that should update nodes directly
  useEffect(() => {
    // Skip if updateViewer is currently running
    if (isUpdatingViewerRef.current) {
      return;
    }
    
    // Skip if we're updating properties (they update nodes directly, not via reload)
    if (isUpdatingPropertiesRef.current) {
      return;
    }
    
    if (viewerRef.current && archData && wasmApiRef.current) {
      // Check if viewer needs to be updated
      // Only update if the viewer's current data doesn't match archData
      // This prevents infinite loops and only updates when necessary
      try {
        const currentData = viewerRef.current.toJSON();
        const currentArchStr = JSON.stringify(currentData.architecture || {});
        const newArchStr = JSON.stringify(archData.architecture || {});
        
        // Only reload if the architecture actually changed and viewer is ready
        if (currentArchStr !== newArchStr && viewerRef.current.cy && !viewerRef.current.cy.destroyed()) {
          console.log('Effect: Reloading viewer with new archData (fallback)');
          isUpdatingViewerRef.current = true;
          viewerRef.current.load(archData).then(() => {
            // Ensure viewer resizes and fits after loading
            if (viewerRef.current?.cy && !viewerRef.current.cy.destroyed()) {
              setTimeout(() => {
                if (viewerRef.current?.cy && !viewerRef.current.cy.destroyed()) {
                  viewerRef.current.cy.resize();
                  viewerRef.current.cy.fit(undefined, 80);
                  setZoomLevel(viewerRef.current.cy.zoom());
                }
                isUpdatingViewerRef.current = false;
              }, 100);
            } else {
              isUpdatingViewerRef.current = false;
            }
          }).catch((err) => {
            logger.error('Failed to reload viewer with new archData', {
              component: 'studio',
              action: 'reload_viewer',
              errorType: err instanceof Error ? err.constructor.name : 'unknown',
              error: err instanceof Error ? err.message : String(err),
            });
            isUpdatingViewerRef.current = false;
          });
        }
      } catch (err) {
        // Viewer might not be ready yet, ignore
        console.debug('Viewer not ready for archData update:', err);
      }
    }
  }, [archData, viewerRef, wasmApiRef, setZoomLevel, isUpdatingViewerRef, isUpdatingPropertiesRef]);

  // Add Cmd+K shortcut for command palette
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modKey = isMac ? e.metaKey : e.ctrlKey;

      if (modKey && e.key === 'k') {
        e.preventDefault();
        setCommandPaletteOpen(true);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [setCommandPaletteOpen]);

  // Track zoom changes
  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer?.cy) return;
    const cy = viewer.cy;

    const updateZoom = () => {
      setZoomLevel(cy.zoom());
    };

    cy.on('zoom', updateZoom);
    cy.on('pan', updateZoom);
    updateZoom(); // Initial zoom level

    return () => {
      cy.off('zoom', updateZoom);
      cy.off('pan', updateZoom);
    };
  }, [archData, viewerRef, setZoomLevel]); // Re-run when diagram changes

  // Auto-save every 30 seconds
  useEffect(() => {
    const interval = setInterval(() => {
      if (dsl && wasmApiRef.current) {
        const ts = new Date().toISOString();
        import('@sruja/shared').then(({ storeSet }) => {
          storeSet('sruja-studio-autosave', dsl);
          storeSet('sruja-studio-autosave-timestamp', ts);
          setLastSaved(new Date(ts));
        }).catch(() => { });
      }
    }, 30000);

    return () => clearInterval(interval);
  }, [dsl, wasmApiRef, setLastSaved]);

  // Autosave
  useAutosave(dsl);

  // Load autosave on mount
  useEffect(() => {
    const loadSaved = async () => {
      const saved = await persistence.loadLocal();
      if (saved && saved !== dsl) {
        updateDsl(saved);
        console.log('Restored DSL from autosave');
      }
    };
    loadSaved();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Run once
}



