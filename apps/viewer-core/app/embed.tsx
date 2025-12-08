// apps/viewer-core/app/embed.tsx
// apps/viewer-core/app/embed.tsx
import { logger } from '@sruja/shared';
// This creates a UMD/IIFE bundle that includes React and all dependencies
// Implements proper C4 multi-view with separate graph builds per view level
import React, { useEffect, useRef, useImperativeHandle, forwardRef } from 'react';
import { createRoot, Root } from 'react-dom/client';
import { createViewer } from '@sruja/viewer';
import type { ArchitectureJSON, ViewerInstance } from '@sruja/viewer';
import '@sruja/ui/design-system/styles.css';

// Store active viewer instance and data for external access
let activeViewer: ViewerInstance | null = null;
let activeRoot: Root | null = null;
let archData: ArchitectureJSON | null = null;

// Minimal viewer component that exposes the viewer instance
const EmbedViewer = forwardRef<ViewerInstance | null, { data: ArchitectureJSON }>(
  ({ data }, ref) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const viewerRef = useRef<ViewerInstance | null>(null);

    useEffect(() => {
      if (!containerRef.current) return;
      const viewer = createViewer({ container: containerRef.current, data });
      viewerRef.current = viewer;
      activeViewer = viewer;
      archData = data;
      viewer.init();
      return () => {
        viewer.destroy();
        viewerRef.current = null;
        activeViewer = null;
      };
    }, []);

    useEffect(() => {
      if (viewerRef.current) {
        viewerRef.current.load(data);
        archData = data;
      }
    }, [data]);

    useImperativeHandle(ref, () => viewerRef.current as ViewerInstance);

    return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />;
  }
);

function mount(selector: string, data: ArchitectureJSON) {
  const container = document.querySelector(selector);
  if (!container) {
    console.error(`Container not found: ${selector}`);
    return;
  }

  // Store architecture data
  archData = data;

  // Cleanup previous instance if any
  if (activeRoot) {
    activeRoot.unmount();
  }

  activeRoot = createRoot(container);
  activeRoot.render(React.createElement(EmbedViewer, { data }));
}

// =====================================================
// C4 Multi-View: Build elements per view level
// =====================================================

interface CyElement {
  group?: 'nodes' | 'edges';
  data: {
    id: string;
    label?: string;
    type?: string;
    parent?: string;
    source?: string;
    target?: string;
    [key: string]: any;
  };
}

/**
 * Build Cytoscape elements for a specific C4 view level
 */
function buildElementsForView(
  data: ArchitectureJSON,
  level: number,
  focusId?: string
): CyElement[] {
  const arch = data.architecture || {};
  const persons = arch.persons || [];
  const systems = arch.systems || [];
  const relations = arch.relations || [];

  const nodes: CyElement[] = [];
  const edges: CyElement[] = [];
  const addedNodeIds = new Set<string>();

  // Helper to add a node only once
  const addNode = (node: CyElement) => {
    if (!addedNodeIds.has(node.data.id)) {
      nodes.push(node);
      addedNodeIds.add(node.data.id);
    }
  };

  // Helper to check if an edge connects two added nodes
  const canAddEdge = (from: string, to: string) => {
    return addedNodeIds.has(from) && addedNodeIds.has(to);
  };

  if (level === 0) {
    // ==========================================
    // FULL ARCHITECTURE VIEW
    // Shows: Everything - Persons, Systems, Containers, Components
    // ==========================================

    // Add all persons
    persons.forEach((p: any) => {
      addNode({
        group: 'nodes',
        data: {
          id: p.id,
          label: p.label || p.id,
          type: 'person',
          description: p.description
        }
      });
    });

    // Add all systems with their containers and components
    systems.forEach((s: any) => {
      addNode({
        group: 'nodes',
        data: {
          id: s.id,
          label: s.label || s.id,
          type: 'system',
          description: s.description,
          external: s.external
        }
      });

      // Add containers
      const containers = s.containers || [];
      containers.forEach((c: any) => {
        addNode({
          group: 'nodes',
          data: {
            id: c.id,
            label: c.label || c.id,
            type: 'container',
            parent: s.id,
            technology: c.technology,
            description: c.description
          }
        });

        // Add components
        const components = c.components || [];
        components.forEach((comp: any) => {
          addNode({
            group: 'nodes',
            data: {
              id: comp.id,
              label: comp.label || comp.id,
              type: 'component',
              parent: c.id,
              technology: comp.technology,
              description: comp.description
            }
          });
        });
      });
    });

    // Add all relations
    relations.forEach((r: any) => {
      if (canAddEdge(r.from, r.to)) {
        edges.push({
          group: 'edges',
          data: {
            id: `edge-${r.from}-${r.to}`,
            source: r.from,
            target: r.to,
            label: r.label || r.verb || ''
          }
        });
      }
    });
  }
  else if (level === 1) {
    // ==========================================
    // L1: SYSTEM CONTEXT VIEW
    // Shows: Persons + Systems (as simple nodes, no children)
    // ==========================================

    // Add all persons
    persons.forEach((p: any) => {
      addNode({
        group: 'nodes',
        data: {
          id: p.id,
          label: p.label || p.id,
          type: 'person',
          description: p.description
        }
      });
    });

    // Add all systems as simple nodes (NO containers)
    systems.forEach((s: any) => {
      addNode({
        group: 'nodes',
        data: {
          id: s.id,
          label: s.label || s.id,
          type: 'system',
          description: s.description,
          external: s.external
        }
      });
    });

    // Add top-level relations between persons and systems
    relations.forEach((r: any) => {
      // Only add if both endpoints are persons or systems (not containers/components)
      const fromIsTopLevel = persons.some((p: any) => p.id === r.from) ||
        systems.some((s: any) => s.id === r.from);
      const toIsTopLevel = persons.some((p: any) => p.id === r.to) ||
        systems.some((s: any) => s.id === r.to);

      if (fromIsTopLevel && toIsTopLevel && canAddEdge(r.from, r.to)) {
        edges.push({
          group: 'edges',
          data: {
            id: `edge-${r.from}-${r.to}`,
            source: r.from,
            target: r.to,
            label: r.label || r.verb || ''
          }
        });
      }
    });
  }
  else if (level === 2 && focusId) {
    // ==========================================
    // L2: CONTAINER VIEW (for specific system)
    // Shows: Focused system with containers + connected persons/systems
    // ==========================================

    const focusedSystem = systems.find((s: any) => s.id === focusId);
    if (!focusedSystem) {
      logger.warn('System not found', { component: 'viewer', action: 'switch_view', focusId, level: 'system' });
      return [];
    }

    // Add the focused system as parent node
    addNode({
      group: 'nodes',
      data: {
        id: focusedSystem.id,
        label: focusedSystem.label || focusedSystem.id,
        type: 'system',
        description: focusedSystem.description,
        focused: true
      }
    });

    // Add containers within the focused system
    const containers = focusedSystem.containers || [];
    containers.forEach((c: any) => {
      addNode({
        group: 'nodes',
        data: {
          id: c.id,
          label: c.label || c.id,
          type: 'container',
          parent: focusedSystem.id,
          technology: c.technology,
          description: c.description
        }
      });
    });

    // Add connected persons
    persons.forEach((p: any) => {
      // Check if this person has relation to focused system or its containers
      const hasConnection = relations.some((r: any) =>
        (r.from === p.id && (r.to === focusId || containers.some((c: any) => c.id === r.to))) ||
        (r.to === p.id && (r.from === focusId || containers.some((c: any) => c.id === r.from)))
      );
      if (hasConnection) {
        addNode({
          group: 'nodes',
          data: {
            id: p.id,
            label: p.label || p.id,
            type: 'person'
          }
        });
      }
    });

    // Add connected external systems (as simple collapsed nodes)
    systems.forEach((s: any) => {
      if (s.id === focusId) return; // Skip focused system

      const hasConnection = relations.some((r: any) =>
        (r.from === focusId && r.to === s.id) ||
        (r.to === focusId && r.from === s.id) ||
        containers.some((c: any) =>
          (r.from === c.id && r.to === s.id) ||
          (r.to === c.id && r.from === s.id)
        )
      );
      if (hasConnection) {
        addNode({
          group: 'nodes',
          data: {
            id: s.id,
            label: s.label || s.id,
            type: 'system',
            external: true,
            collapsed: true
          }
        });
      }
    });

    // Add relevant edges
    relations.forEach((r: any) => {
      if (canAddEdge(r.from, r.to)) {
        edges.push({
          group: 'edges',
          data: {
            id: `edge-${r.from}-${r.to}`,
            source: r.from,
            target: r.to,
            label: r.label || r.verb || ''
          }
        });
      }
    });
  }
  else if (level === 3 && focusId) {
    // ==========================================
    // L3: COMPONENT VIEW (for specific container)
    // Shows: Focused container with components + siblings + connections
    // ==========================================

    // Find the container and its parent system
    let focusedContainer: any = null;
    let parentSystem: any = null;

    for (const sys of systems) {
      const container = (sys.containers || []).find((c: any) => c.id === focusId);
      if (container) {
        focusedContainer = container;
        parentSystem = sys;
        break;
      }
    }

    if (!focusedContainer || !parentSystem) {
      logger.warn('Container not found', { component: 'viewer', action: 'switch_view', focusId, level: 'container' });
      return [];
    }

    // Add parent system
    addNode({
      group: 'nodes',
      data: {
        id: parentSystem.id,
        label: parentSystem.label || parentSystem.id,
        type: 'system'
      }
    });

    // Add focused container with its components
    addNode({
      group: 'nodes',
      data: {
        id: focusedContainer.id,
        label: focusedContainer.label || focusedContainer.id,
        type: 'container',
        parent: parentSystem.id,
        technology: focusedContainer.technology,
        focused: true
      }
    });

    // Add components within focused container
    const components = focusedContainer.components || [];
    components.forEach((comp: any) => {
      addNode({
        group: 'nodes',
        data: {
          id: comp.id,
          label: comp.label || comp.id,
          type: 'component',
          parent: focusedContainer.id,
          technology: comp.technology,
          description: comp.description
        }
      });
    });

    // Add sibling containers (collapsed, no children)
    const siblingContainers = (parentSystem.containers || []).filter(
      (c: any) => c.id !== focusId
    );
    siblingContainers.forEach((c: any) => {
      addNode({
        group: 'nodes',
        data: {
          id: c.id,
          label: c.label || c.id,
          type: 'container',
          parent: parentSystem.id,
          collapsed: true
        }
      });
    });

    // Add connected persons
    persons.forEach((p: any) => {
      const hasConnection = relations.some((r: any) =>
        (r.from === p.id && (r.to === focusId || components.some((c: any) => c.id === r.to))) ||
        (r.to === p.id && (r.from === focusId || components.some((c: any) => c.id === r.from)))
      );
      if (hasConnection) {
        addNode({
          group: 'nodes',
          data: {
            id: p.id,
            label: p.label || p.id,
            type: 'person'
          }
        });
      }
    });

    // Add relevant edges
    relations.forEach((r: any) => {
      if (canAddEdge(r.from, r.to)) {
        edges.push({
          group: 'edges',
          data: {
            id: `edge-${r.from}-${r.to}`,
            source: r.from,
            target: r.to,
            label: r.label || r.verb || ''
          }
        });
      }
    });
  }

  return [...nodes, ...edges];
}

/**
 * Switch to a different C4 view by rebuilding the graph
 */
function switchView(level: number, focusId?: string) {
  if (!activeViewer || !activeViewer.cy || !archData) {
    logger.warn('Viewer not ready for switchView', { component: 'viewer', action: 'switch_view' });
    return;
  }

  const cy = activeViewer.cy;

  // Clear any existing layoutstop handlers to prevent interference
  cy.off('layoutstop');

  // Build new elements for this view
  const elements = buildElementsForView(archData, level, focusId);

  if (elements.length === 0) {
    logger.warn('No elements for view', { component: 'viewer', action: 'switch_view', level, focusId });
    return;
  }

  

  // Remove all existing elements
  cy.elements().remove();

  // Add new elements
  cy.add(elements);

  // Try dagre layout, fallback to breadthfirst or grid
  // Note: fit: true in layout options handles fitting automatically
  let layoutOptions: any = {
    name: 'dagre',
    rankDir: 'TB',
    nodeSep: 80,
    rankSep: 100,
    edgeSep: 50,
    padding: 50,
    fit: true,
    animate: false
  };

  // Check if dagre is available
  try {
    const layout = cy.layout(layoutOptions);
    layout.one('layoutstop', () => {
      
      // Don't call cy.fit() here - it overrides dagre's positioning
    });
    layout.run();
  } catch (e) {
    logger.warn('Dagre layout failed, falling back to breadthfirst', {
      component: 'viewer',
      action: 'layout',
      errorType: e instanceof Error ? e.constructor.name : 'unknown',
      error: e instanceof Error ? e.message : String(e),
    });
    // Fallback to breadthfirst layout
    const fallbackLayout = cy.layout({
      name: 'breadthfirst',
      directed: true,
      padding: 50,
      spacingFactor: 1.5,
      fit: true
    });
    fallbackLayout.run();
    // Don't call cy.fit() - layout already fits
  }
}

// Legacy functions for compatibility
function setLevel(level: number) {
  switchView(level);
}

function fit() {
  if (activeViewer && activeViewer.cy) {
    activeViewer.cy.fit(undefined, 50);
  }
}

function toggleCollapse(nodeId?: string) {
  if (activeViewer) {
    (activeViewer as any).toggleCollapse(nodeId);
  }
}

function reset() {
  if (activeViewer) {
    activeViewer.reset();
  }
}

// Export to global window object for UMD/IIFE bundle
if (typeof window !== 'undefined') {
  (window as any).SrujaViewer = {
    mount,
    switchView,
    setLevel,
    fit,
    toggleCollapse,
    reset,
    buildElementsForView,
    getViewer: () => activeViewer,
    getData: () => archData
  };
}

export { mount, switchView, setLevel, fit, toggleCollapse, reset, buildElementsForView };
