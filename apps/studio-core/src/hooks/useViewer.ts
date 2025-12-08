// apps/studio-core/src/hooks/useViewer.ts
import { useEffect, useRef, useState } from 'react';
import { createViewer, type ArchitectureJSON, type ViewerInstance } from '@sruja/viewer';
import type { WasmApi } from '@sruja/shared';
import { initWasm } from '../wasm';
import { EXAMPLES } from '../examples';
import { SAMPLE_JSON } from '../constants/sampleData';
import LZString from 'lz-string';
import { findNodeInArch, getNodeType } from '../utils/nodeUtils';
import { logger } from '@sruja/shared';

interface UseViewerOptions {
  containerRef: React.RefObject<HTMLDivElement | null>;
  showSidebar: boolean;
  activeSidebarPanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide';
  updateDsl: (dsl: string) => Promise<void>;
  setSelectedNodeId: (id: string | null) => void;
  setArchData: (data: ArchitectureJSON | null) => void;
  setSelectedDocNodeType: (type: string | null) => void;
  setSelectedDocNodeId: (id: string | undefined) => void;
  setSelectedDocNodeLabel: (label: string | undefined) => void;
  setZoomLevel: (level: number) => void;
  setIsWasmLoading: (loading: boolean) => void;
  initialDsl?: string;
  viewerRefExternal?: React.RefObject<ViewerInstance | null>;
  wasmApiRefExternal?: React.RefObject<WasmApi | null>;
}

export function useViewer(options: UseViewerOptions) {
  const {
    containerRef,
    showSidebar,
    activeSidebarPanel,
    updateDsl,
    setSelectedNodeId,
    setArchData,
    setSelectedDocNodeType,
    setSelectedDocNodeId,
    setSelectedDocNodeLabel,
    setZoomLevel,
    setIsWasmLoading,
    initialDsl,
  } = options;

  const viewerRef = useRef<ViewerInstance | null>(null);
  const wasmApiRef = useRef<WasmApi | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    let viewer: ViewerInstance | null = null;
    const cleanupFunctions: Array<() => void> = [];

    // MutationObserver to detect when viewer container is ready
    const viewerElement = containerRef.current;
    if (viewerElement) {
      const observer = new MutationObserver(() => {
        if (viewer && viewer.cy && !viewer.cy.destroyed()) {
          viewer.cy.resize();
        }
      });
      observer.observe(viewerElement, { childList: true, subtree: true });
      cleanupFunctions.push(() => observer.disconnect());
    }

    // Handle window resize
    const handleResize = () => {
      if (viewer && viewer.cy && !viewer.cy.destroyed()) {
        viewer.cy.resize();
        viewer.cy.fit(undefined, 80);
      }
    };

    window.addEventListener('resize', handleResize);
    cleanupFunctions.push(() => window.removeEventListener('resize', handleResize));

    // Initialize WASM first, then create viewer
    setIsWasmLoading(true);
    initWasm()
      .then((api) => {
        wasmApiRef.current = api;
        if (options.wasmApiRefExternal) {
          options.wasmApiRefExternal.current = api;
        }
        const dslToUse = initialDsl || EXAMPLES['Simple Web App'];
        updateDsl(dslToUse);

        // Parse the DSL first
        return api.parseDslToJson(initialDsl).then((parsedJson: string) => {
          const parsed = JSON.parse(parsedJson);

          // Create viewer with the actual parsed data
          viewer = createViewer({
            container: containerRef.current!,
            data: parsed,
            onSelect: (id: string | null) => {
              setSelectedNodeId(id);
              if (id && parsed) {
                const node = findNodeInArch(parsed, id);
                if (node) {
                  const nodeType = getNodeType(node, parsed);
                  setSelectedDocNodeType(nodeType);
                  setSelectedDocNodeId(id);
                  setSelectedDocNodeLabel(node.label || id);
                  if (showSidebar && activeSidebarPanel !== 'documentation') {
                    // Auto-switch handled by parent
                  }
                }
              } else {
                setSelectedDocNodeType(null);
              }
            }
          });
          viewerRef.current = viewer;
          if (options.viewerRefExternal) {
            options.viewerRefExternal.current = viewer;
          }
          (window as any).srujaViewer = viewer;
          setArchData(parsed);

          viewer.init();

          // Wait for Cytoscape to be fully ready
          const waitForCytoscape = () => {
            const currentViewer = viewer;
            if (currentViewer && currentViewer.cy && !currentViewer.cy.destroyed()) {
              currentViewer.cy.ready(() => {
                if (currentViewer && currentViewer.cy && !currentViewer.cy.destroyed()) {
                  currentViewer.cy.resize();
                  currentViewer.cy.fit(undefined, 80);
                  setZoomLevel(currentViewer.cy.zoom());
                }
              });
            } else {
              setTimeout(waitForCytoscape, 50);
            }
          };

          setTimeout(waitForCytoscape, 50);
          setIsWasmLoading(false);
        });
      })
      .catch((err) => {
        logger.error('WASM init failed', {
          component: 'studio',
          action: 'init_wasm',
          errorType: err instanceof Error ? err.constructor.name : 'unknown',
          error: err instanceof Error ? err.message : String(err),
        });
        setIsWasmLoading(false);
        // Fallback: create viewer with sample data
        viewer = createViewer({
          container: containerRef.current!,
          data: SAMPLE_JSON,
          onSelect: (id: string | null) => {
            setSelectedNodeId(id);
            if (id) {
              const node = findNodeInArch(SAMPLE_JSON, id);
              if (node) {
                const nodeType = getNodeType(node, SAMPLE_JSON);
                if (activeSidebarPanel === 'documentation') {
                  setSelectedDocNodeType(nodeType);
                  setSelectedDocNodeId(id);
                  setSelectedDocNodeLabel(node.label || id);
                }
              }
            } else {
              setSelectedDocNodeType(null);
              setSelectedDocNodeId(undefined);
              setSelectedDocNodeLabel(undefined);
            }
          }
        });
        viewerRef.current = viewer;
        if (options.viewerRefExternal) {
          options.viewerRefExternal.current = viewer;
        }
        (window as any).srujaViewer = viewer;
        viewer.init();
      });

    return () => {
      cleanupFunctions.forEach((cleanup) => cleanup());
      if (viewer) {
        viewer.destroy();
      }
    };
  }, []);

  return { viewerRef, wasmApiRef };
}
