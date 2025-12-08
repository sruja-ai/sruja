// Core viewer implementation

import cytoscape, { Core, ElementDefinition } from 'cytoscape';
import dagre from 'cytoscape-dagre';
import fcose from 'cytoscape-fcose';
import ELK from 'elkjs/lib/elk.bundled.js';
import cytoscapeElk from 'cytoscape-elk';

if (typeof window !== 'undefined') {
  cytoscape.use(dagre);
  cytoscape.use(fcose);
  window.ELK = window.ELK || (ELK as unknown);
  cytoscape.use(cytoscapeElk);
}

import type { ArchitectureJSON, ViewerOptions, ViewerInstance, LayoutData, MetadataJSON } from './types';
import type { LayoutOptions } from './types/layout';
import { buildStyles } from './style';
import { waitForContainerSize, computeLayoutOptions, createLayoutStopHandler, fitPresetWithRetry } from './layout';
import { exportPNG as doExportPNG, exportSVG as doExportSVG } from './export';
import { buildNodes, buildTopLevelRelations, buildSystemRelations } from './graph/builders';
import { resolveNodeId as resolveIdUtil } from './utils/id';
import { initEvents } from './events';
import { extractLayout } from './serialize/layout';
import { serializeNodes } from './serialize/nodes';
import { serializeRelations } from './serialize/relations';
import { logger } from '@sruja/shared';
import {
  collapseNode,
  expandNode,
  collapseNodesByType,
  expandNodesByType,
  applyFocusDimming,
  getNode,
  ANIMATION_DURATION_MS,
} from './utils/node-operations';

/**
 * Sruja Viewer - Renders architecture diagrams using Cytoscape.js
 */
export class SrujaViewer implements ViewerInstance {
  private container: string | HTMLElement;
  public cy: Core | null = null;
  private data?: ArchitectureJSON;
  private onSelect?: (id: string | null) => void;
  // @ts-expect-error - used in setFocus
  private focus: { systemId?: string; containerId?: string } | undefined;
  private layoutStopHandler: (() => void) | null = null;
  private eventCleanup: (() => void) | null = null;

  constructor(options: ViewerOptions) {
    logger.info('SrujaViewer initialized (@sruja/viewer)');
    this.container = options.container;
    this.data = options.data;
    this.onSelect = options.onSelect;
  }

  /**
   * Initialize the viewer
   */
  async init(): Promise<void> {
    if (this.cy) {
      this.destroy();
    }

    const containerElement =
      typeof this.container === 'string'
        ? document.querySelector(this.container)
        : this.container;

    if (!containerElement) {
      throw new Error(`Container not found: ${this.container}`);
    }

    // Ensure container is visible and has dimensions
    // Use ResizeObserver to wait for container to be properly sized
    const htmlElement = containerElement as HTMLElement;
    const rect = htmlElement.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) {
      logger.debug('Container has zero dimensions, waiting for resize...', {
        width: rect.width,
        height: rect.height,
        display: window.getComputedStyle(htmlElement).display,
        visibility: window.getComputedStyle(htmlElement).visibility
      });
      await waitForContainerSize(htmlElement);
    }

    const elements = this.data ? this.convertToCytoscape(this.data) : [];
    // Allow empty diagrams - don't create dummy elements
    // Empty architecture should show empty canvas

    // Check if we have layout metadata AND if any elements actually have positions
    const metadata = this.data?.metadata as MetadataJSON | undefined;
    const hasLayoutMetadata = !!(metadata && metadata.layout);
    const hasPositions = elements.some(e => e.position !== undefined);
    const usePreset = hasLayoutMetadata && hasPositions;

    const engine = metadata && typeof metadata.layoutEngine === 'string' ? metadata.layoutEngine : '';

    const layoutOptions = computeLayoutOptions(usePreset, engine);

    this.cy = cytoscape({
      container: containerElement as HTMLElement,
      elements,
      style: buildStyles(),
      layout: layoutOptions,
      minZoom: 0.1,
      maxZoom: 3,
      boxSelectionEnabled: true,
      autoungrabify: false,
      userZoomingEnabled: true,
      userPanningEnabled: true,
      // Enable node dragging for drag-to-connect feature
      // Removed wheelSensitivity to use default and avoid console warnings
    });

    // Initialize event handlers
    this.eventCleanup = initEvents(this.cy, {
      onSelect: this.onSelect
    }, this.data);

    // Ensure container has dimensions before fitting
    const containerRect = (containerElement as HTMLElement).getBoundingClientRect();
    logger.debug('Container dimensions at init', { width: containerRect.width, height: containerRect.height });

    // Remove any existing layoutstop handler to prevent accumulation
    if (this.layoutStopHandler && this.cy) {
      this.cy.off('layoutstop', this.layoutStopHandler);
    }

    // retries handled inside layout.ts

    if (!usePreset) {
      this.cy.fit();

      const handler = createLayoutStopHandler(this.cy, containerElement as HTMLElement, () => {
        if (!this.cy) return;
        this.cy.off('layoutstop', handler);
        this.layoutStopHandler = null;
      });
      this.layoutStopHandler = handler;
      this.cy.on('layoutstop', handler);
    } else {
      fitPresetWithRetry(this.cy, containerElement as HTMLElement);
    }
  }

  /**
   * Load new architecture data
   */
  async load(data: ArchitectureJSON): Promise<void> {
    this.data = data;
    // Clean up layoutstop handler before destroying
    if (this.layoutStopHandler && this.cy) {
      this.cy.off('layoutstop', this.layoutStopHandler);
      this.layoutStopHandler = null;
    }
    if (this.cy) {
      this.cy.destroy();
      this.cy = null;
    }
    // retries handled inside layout.ts
    await this.init();
  }

  /**
   * Get the Cytoscape instance
   */
  get cyInstance(): Core | null {
    return this.cy;
  }

  /**
   * Destroy the viewer
   */
  destroy(): void {
    // Clean up event handlers
    if (this.eventCleanup) {
      this.eventCleanup();
      this.eventCleanup = null;
    }
    // Clean up layoutstop handler before destroying
    if (this.layoutStopHandler && this.cy) {
      this.cy.off('layoutstop', this.layoutStopHandler);
      this.layoutStopHandler = null;
    }
    if (this.cy) {
      this.cy.destroy();
      this.cy = null;
    }
  }

  reset(): void {
    if (!this.cy) return;
    const cy = this.cy;
    cy.elements().unselect();
    cy.elements().removeClass('highlight');
    cy.elements().removeClass('scenario-highlight');
    cy.elements().style('background-color', '');
    cy.elements().style('border-color', '');
    cy.elements().style('border-width', '');
    cy.edges().style('line-color', '#94a3b8');
    cy.edges().style('target-arrow-color', '#94a3b8');
    cy.edges().style('width', '2');
    // Clear focus dimming
    cy.elements().style('opacity', 1);
    cy.fit(undefined, 80);
  }

  /**
   * Convert ArchitectureJSON to Cytoscape elements
   */
  private convertToCytoscape(data: ArchitectureJSON): ElementDefinition[] {
    const arch2 = data.architecture || {};
    const layout2 = (data.metadata as MetadataJSON)?.layout || {};
    const nodeIds2 = new Set<string>();
    const getPos2 = (id: string) => (layout2[id] ? { x: layout2[id].x, y: layout2[id].y } : undefined);
    const builtElements = buildNodes(arch2, getPos2, nodeIds2);
    const resolveNodeId2 = (id: string) => resolveIdUtil(id, arch2, nodeIds2);
    const relationElems = [
      ...buildTopLevelRelations(arch2, resolveNodeId2),
      ...buildSystemRelations(arch2, nodeIds2),
    ];
    return [...builtElements, ...relationElems];
  }

  /**
   * Add a node to the graph
   */
  addNode(type: string, label: string, parentId?: string, extraData?: Record<string, unknown>): void {
    if (!this.cy) return;

    // Generate ID
    let id = label.replace(/\s+/g, '');
    if (parentId) {
      id = `${parentId}.${id}`;
    }

    // Ensure unique ID
    let uniqueId = id;
    let counter = 1;
    while (this.cy.getElementById(uniqueId).length > 0) {
      uniqueId = `${id}${counter}`;
      counter++;
    }

    const node: ElementDefinition = {
      group: 'nodes',
      data: {
        id: uniqueId,
        label,
        type,
        parent: parentId,
        ...extraData
      }
    };

    this.cy.add(node);

    // Layout to position new node
    try {
      const layoutOpts: LayoutOptions = {
        name: 'fcose',
        animate: true,
        animationDuration: ANIMATION_DURATION_MS,
        fit: false,
        randomize: false,
        nodeDimensionsIncludeLabels: true
      };
      const layout = this.cy.layout(layoutOpts);
      layout.run();
    } catch (e) {
      logger.warn('Layout failed in addNode', { error: e instanceof Error ? e.message : String(e) });
    }
  }

  /**
   * Remove selected elements
   */
  removeSelected(): void {
    if (!this.cy) return;
    const selected = this.cy.$(':selected');
    if (selected.length > 0) {
      selected.remove();
    }
  }

  /**
   * Select and zoom to a node
   */
  selectNode(id: string): void {
    if (!this.cy) return;
    const node = this.cy.getElementById(id);
    if (node.length > 0) {
      // Expand ancestors if needed
      node.ancestors().forEach(ancestor => {
        if (ancestor.data('collapsed')) {
          ancestor.data('collapsed', false);
          ancestor.removeClass('collapsed');
          ancestor.children().style('display', 'element');
        }
      });

      this.cy.elements().unselect();
      node.select();
      this.cy.animate({
        fit: {
          eles: node,
          padding: 50
        },
        duration: ANIMATION_DURATION_MS
      });
    }
  }

  /**
   * Toggle collapse state of a compound node
   */
  toggleCollapse(id?: string): void {
    if (!this.cy) return;

    const node = getNode(this.cy, id);
    if (!node || node.length === 0) return;

    // Check if it's a compound node (has children)
    const children = node.children();
    if (children.length === 0) return;

    const isCollapsed = node.data('collapsed');
    if (isCollapsed) {
      expandNode(this.cy, node.id());
    } else {
      collapseNode(this.cy, node.id());
    }
  }

  /**
   * Set visualization level (1=System Context, 2=Container, 3=Component)
   */
  setLevel(level: number): void {
    if (!this.cy) return;

    const cy = this.cy;
    cy.batch(() => {
      if (level === 1) {
        // Collapse all systems (System Context view)
        collapseNodesByType(cy, 'system');
      } else if (level === 2) {
        // Expand systems, collapse containers (Container view)
        expandNodesByType(cy, 'system');
        collapseNodesByType(cy, 'container');
      } else if (level === 3) {
        // Expand everything (Component view)
        expandNodesByType(cy, 'system');
        expandNodesByType(cy, 'container');
      }
    });
  }

  /**
   * Focus view on a specific system or container. Non-focused elements are dimmed.
   */
  setFocus(focus?: { systemId?: string; containerId?: string }): void {
    if (!this.cy) return;
    this.focus = focus;
    applyFocusDimming(this.cy, focus || {});
  }

  /**
   * Add an edge to the graph
   */
  addEdge(sourceId: string, targetId: string, label?: string): void {
    if (!this.cy) return;

    this.cy.add({
      group: 'edges',
      data: {
        source: sourceId,
        target: targetId,
        label
      }
    });
  }

  /**
   * Export diagram as PNG (synchronous version using Cytoscape's built-in method)
   */
  exportPNG(options?: { scale?: number }): string {
    if (!this.cy) return '';
    return doExportPNG(this.cy, options);
  }

  /**
   * Export diagram as SVG
   */
  exportSVG(options?: { scale?: number }): string {
    if (!this.cy) return '';
    return doExportSVG(this.cy, options);
  }


  /**
   * Export current graph to ArchitectureJSON
   */
  toJSON(): ArchitectureJSON {
    if (!this.cy) {
      return this.data || {} as ArchitectureJSON;
    }

    const json: ArchitectureJSON = {
      architecture: {},
      metadata: { name: '', version: '', generated: '', layout: {} } as MetadataJSON,
      navigation: { levels: [] }
    };

    // Extract layout
    const layout = extractLayout(this.cy);
    (json.metadata as MetadataJSON).layout = layout;

    // Serialize nodes
    serializeNodes(this.cy, json.architecture!);

    // Serialize relations
    serializeRelations(this.cy, json.architecture!);

    return json;
  }

  getLayout(): Record<string, LayoutData> {
    if (!this.cy) return {};
    return extractLayout(this.cy);
  }
}

/**
 * Factory function to create a viewer instance
 */
export function createViewer(options: ViewerOptions): ViewerInstance {
  return new SrujaViewer(options);
}
