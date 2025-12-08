// Core viewer implementation

import cytoscape, { Core, ElementDefinition } from 'cytoscape';
// @ts-ignore
import dagre from 'cytoscape-dagre';
// @ts-ignore
import fcose from 'cytoscape-fcose';
// @ts-ignore
import ELK from 'elkjs/lib/elk.bundled.js';
// @ts-ignore
import cytoscapeElk from 'cytoscape-elk';

cytoscape.use(dagre);
cytoscape.use(fcose);
(window as any).ELK = (window as any).ELK || ELK;
cytoscape.use(cytoscapeElk);

import type { ArchitectureJSON, ViewerOptions, ViewerInstance, LayoutData } from './types';
import { convertToCytoscape } from './converter';
import { Colors, getCssVar } from '@sruja/shared/utils/cssVars';
import { getDefaultStyle } from './styleHelpers';

/**
 * Sruja Viewer - Renders architecture diagrams using Cytoscape.js
 */
export class SrujaViewer implements ViewerInstance {
  private container: string | HTMLElement;
  public cy: Core | null = null;
  private data?: ArchitectureJSON;
  private onSelect?: (id: string | null) => void;

  constructor(options: ViewerOptions) {
    console.log('SrujaViewer initialized (@sruja/viewer)');
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

    let elements = this.data ? convertToCytoscape(this.data) : [];
    if (!elements || elements.length === 0) {
      elements = [
        { data: { id: 'node-a', label: 'A', type: 'container' } },
        { data: { id: 'node-b', label: 'B', type: 'container' } },
        { data: { id: 'node-a-node-b', source: 'node-a', target: 'node-b', label: 'connect' } },
      ];
    }

    // Check if we have layout metadata AND if any elements actually have positions
    const metaAny = this.data?.metadata as any;
    const hasLayoutMetadata = metaAny && metaAny.layout;
    const hasPositions = elements.some(e => e.position !== undefined);
    const usePreset = hasLayoutMetadata && hasPositions;

    const engine = metaAny && typeof metaAny.layoutEngine === 'string' ? metaAny.layoutEngine : '';

    const layoutOptions = usePreset ? {
      name: 'preset',
      fit: true,
      padding: 50,
      animate: true,
      animationDuration: 500
    } : {
      name: (engine && ['dagre', 'cose', 'fcose', 'elk'].includes(engine)) ? engine : 'dagre',
      // @ts-ignore
      rankDir: 'TB',
      nodeSep: 50,
      rankSep: 100,
      padding: 50,
      animate: true,
      animationDuration: 500,
      fit: true,
    };

    this.cy = cytoscape({
      container: containerElement as HTMLElement,
      elements,
      style: getDefaultStyle(),
      layout: layoutOptions as any,
      minZoom: 0.1,
      maxZoom: 3,
      boxSelectionEnabled: true,
      autoungrabify: false,
      userZoomingEnabled: true,
      userPanningEnabled: true,
      // Removed wheelSensitivity to use default and avoid console warnings
    });

    this.cy.on('select', 'node', (e) => {
      const node = e.target;
      if (this.onSelect) {
        this.onSelect(node.id());
      }
    });

    this.cy.on('unselect', 'node', () => {
      // Check if any other node is selected
      const selected = this.cy?.$('node:selected');
      if (selected && selected.length === 0 && this.onSelect) {
        this.onSelect(null);
      }
    });

    if (!usePreset) {
      this.cy.fit();
      this.cy.on('layoutstop', () => {
        this.cy && this.cy.fit(undefined, 80);
      });
    } else {
      this.cy.fit(undefined, 80);
    }
  }

  /**
   * Load new architecture data
   */
  async load(data: ArchitectureJSON): Promise<void> {
    this.data = data;
    if (this.cy) {
      this.cy.destroy();
      this.cy = null;
    }
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
    cy.edges().style('line-color', Colors.neutral500());
    cy.edges().style('target-arrow-color', Colors.neutral500());
    cy.edges().style('width', '2');
    cy.fit(undefined, 80);
  }

  /**
   * Convert ArchitectureJSON to Cytoscape elements
   */
  private convertToCytoscape(data: ArchitectureJSON): ElementDefinition[] {
    const elements: ElementDefinition[] = [];
    const nodeIds = new Set<string>();
    const arch = data.architecture || {};
    const layout = (data.metadata as any)?.layout || {};

    // Helper to add position if available
    const getPos = (id: string) => {
      if (layout[id]) {
        return { x: layout[id].x, y: layout[id].y };
      }
      return undefined;
    };

    // Add persons
    if (arch.persons) {
      for (const person of arch.persons) {
        nodeIds.add(person.id);
        elements.push({
          data: {
            id: person.id,
            label: person.label || person.id,
            type: 'person',
            metadata: person.metadata,
          },
          position: getPos(person.id)
        });
      }
    }

    // Add systems
    if (arch.systems) {
      for (const system of arch.systems) {
        nodeIds.add(system.id);
        elements.push({
          data: {
            id: system.id,
            label: system.label || system.id,
            type: 'system',
            metadata: system.metadata,
          },
          position: getPos(system.id)
        });

        // Add containers within systems
        if (system.containers) {
          for (const container of system.containers) {
            const containerId = `${system.id}.${container.id}`;
            nodeIds.add(containerId);
            elements.push({
              data: {
                id: containerId,
                label: container.label || container.id,
                type: 'container',
                parent: system.id,
                metadata: container.metadata,
              },
              position: getPos(containerId)
            });
          }
        }

        // Add datastores within systems
        if (system.datastores) {
          for (const ds of system.datastores) {
            const dsId = `${system.id}.${ds.id}`;
            nodeIds.add(dsId);
            elements.push({
              data: {
                id: dsId,
                label: ds.label || ds.id,
                type: 'datastore',
                parent: system.id,
                metadata: ds.metadata,
              },
              position: getPos(dsId)
            });
          }
        }

        // Add queues within systems
        if (system.queues) {
          for (const queue of system.queues) {
            const qId = `${system.id}.${queue.id}`;
            nodeIds.add(qId);
            elements.push({
              data: {
                id: qId,
                label: queue.label || queue.id,
                type: 'queue',
                parent: system.id,
                metadata: queue.metadata,
              },
            });
          }
        }
      }
    }

    // Add top-level containers (if any)
    if (arch.containers) {
      for (const container of arch.containers) {
        nodeIds.add(container.id);
        elements.push({
          data: {
            id: container.id,
            label: container.label || container.id,
            type: 'container',
            metadata: container.metadata,
          },
        });
      }
    }

    // Add data stores
    if (arch.datastores) {
      for (const ds of arch.datastores) {
        nodeIds.add(ds.id);
        elements.push({
          data: {
            id: ds.id,
            label: ds.label || ds.id,
            type: 'datastore',
            metadata: ds.metadata,
          },
        });
      }
    }

    // Add queues
    if (arch.queues) {
      for (const queue of arch.queues) {
        nodeIds.add(queue.id);
        elements.push({
          data: {
            id: queue.id,
            label: queue.label || queue.id,
            type: 'queue',
            metadata: queue.metadata,
          },
        });
      }
    }

    // Add requirements
    if (arch.requirements) {
      for (const req of arch.requirements) {
        nodeIds.add(req.id);
        elements.push({
          data: {
            id: req.id,
            label: req.title || req.id,
            type: 'requirement',
            metadata: req.metadata as any,
          },
          position: getPos(req.id)
        });
      }
    }

    // Add ADRs
    if (arch.adrs) {
      for (const adr of arch.adrs) {
        nodeIds.add(adr.id);
        elements.push({
          data: {
            id: adr.id,
            label: adr.title || adr.id,
            type: 'adr',
            metadata: adr.metadata as any,
          },
          position: getPos(adr.id)
        });
      }
    }

    // Add deployment nodes
    if (arch.deployment) {
      for (const node of arch.deployment) {
        nodeIds.add(node.id);
        elements.push({
          data: {
            id: node.id,
            label: node.label || node.id,
            type: 'deployment',
            metadata: node.metadata as any,
          },
          position: getPos(node.id)
        });
      }
    }

    // Helper function to resolve node IDs (handles short names like "WebApp" -> "ECommerce.WebApp")
    const resolveNodeId = (id: string): string | null => {
      // First check if it exists as-is
      if (nodeIds.has(id)) {
        return id;
      }
      // Search through systems for containers/components/datastores/queues
      if (arch.systems) {
        for (const system of arch.systems) {
          const qualifiedId = `${system.id}.${id}`;
          if (nodeIds.has(qualifiedId)) {
            return qualifiedId;
          }
          // Also check nested containers
          if (system.containers) {
            for (const container of system.containers) {
              if (container.id === id) {
                return `${system.id}.${id}`;
              }
              // Check components within containers
              if (container.components) {
                for (const component of container.components) {
                  if (component.id === id) {
                    return `${system.id}.${container.id}.${id}`;
                  }
                }
              }
            }
          }
        }
      }
      return null;
    };

    // Add system-level relations
    if (arch.relations) {
      for (const relation of arch.relations) {
        const fromId = resolveNodeId(relation.from);
        const toId = resolveNodeId(relation.to);
        if (fromId && toId) {
          elements.push({
            data: {
              id: `${fromId}-${toId}`,
              source: fromId,
              target: toId,
              label: relation.label || relation.verb || '',
            },
          });
        }
      }
    }

    // Add relations from systems
    if (arch.systems) {
      for (const system of arch.systems) {
        if (system.relations) {
          for (const relation of system.relations) {
            const fromId = `${system.id}.${relation.from}`;
            const toId = `${system.id}.${relation.to}`;
            if (nodeIds.has(fromId) && nodeIds.has(toId)) {
              elements.push({
                data: {
                  id: `${fromId}-${toId}`,
                  source: fromId,
                  target: toId,
                  label: relation.label || relation.verb || '',
                },
              });
            }
          }
        }
      }
    }

    return elements;
  }

  /**
   * Generate SVG icon data URI
   */
  private getIconDataUri(svgContent: string): string {
    // Clean SVG content and URL encode
    const cleaned = svgContent.replace(/\s+/g, ' ').trim();
    return `data:image/svg+xml,${encodeURIComponent(cleaned)}`;
  }

  /**
   * Get C4 model icons as SVG data URIs
   */
  private getC4Icons() {
    // Person/User icon - simple person silhouette
    const personIcon = this.getIconDataUri(
      '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>'
    );

    // System icon - server/box with lines
    const systemIcon = this.getIconDataUri(
      '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line x1="8" y1="21" x2="16" y2="21"></line><line x1="12" y1="17" x2="12" y2="21"></line><line x1="7" y1="8" x2="7" y2="8.01"></line><line x1="12" y1="8" x2="12" y2="8.01"></line><line x1="17" y1="8" x2="17" y2="8.01"></line><line x1="7" y1="12" x2="17" y2="12"></line></svg>'
    );

    // Container icon - application/component box
    const containerIcon = this.getIconDataUri(
      '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#334155" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="3" y1="9" x2="21" y2="9"></line><line x1="9" y1="21" x2="9" y2="9"></line></svg>'
    );

    return { personIcon, systemIcon, containerIcon };
  }

  /**
   * Get style value from metadata
   */
  private getStyleValue(ele: any, key: string, defaultVal: any): any {
    try {
      const meta = ele.data('metadata');
      if (!meta || !Array.isArray(meta)) return defaultVal;
      const entry = meta.find((m: any) => m && m.key === key);
      if (!entry) return defaultVal;
      const value = entry.value;
      // Return defaultVal if value is null, undefined, or empty string (for required properties)
      return value != null && value !== '' ? value : defaultVal;
    } catch (e) {
      return defaultVal;
    }
  }

  /**
   * Get icon from metadata
   */
  private getIconFromMetadata(ele: any, defaultIcon: string): string {
    try {
      const iconName = this.getStyleValue(ele, 'style.icon', '');
      if (!iconName || typeof iconName !== 'string') {
        // Return 'none' if no default icon, otherwise return default
        return defaultIcon && defaultIcon !== '' ? defaultIcon : 'none';
      }

      const icons = this.getC4Icons();
      let result: string;
      switch (iconName) {
        case 'person': result = icons.personIcon || defaultIcon || 'none'; break;
        case 'system': result = icons.systemIcon || defaultIcon || 'none'; break;
        case 'container': result = icons.containerIcon || defaultIcon || 'none'; break;
        case 'database': result = this.getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path></svg>') || defaultIcon || 'none'; break;
        case 'cloud': result = this.getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"></path></svg>') || defaultIcon || 'none'; break;
        case 'server': result = this.getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>') || defaultIcon || 'none'; break;
        case 'mobile': result = this.getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="2" width="14" height="20" rx="2" ry="2"></rect><line x1="12" y1="18" x2="12.01" y2="18"></line></svg>') || defaultIcon || 'none'; break;
        case 'browser': result = this.getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="3" y1="9" x2="21" y2="9"></line><line x1="9" y1="21" x2="9" y2="9"></line></svg>') || defaultIcon || 'none'; break;
        default: result = defaultIcon && defaultIcon !== '' ? defaultIcon : 'none';
      }
      return result && result !== '' ? result : 'none';
    } catch (e) {
      return defaultIcon && defaultIcon !== '' ? defaultIcon : 'none';
    }
  }

  /**
   * Calculate node width based on label and type
   */
  private calculateNodeWidth(ele: any): number {
    const label = ele.data('label') || '';
    const type = ele.data('type');
    const charWidth = 7.5; // Approx for 12px Inter
    const textMaxWidth = type === 'person' ? 100 : (type === 'system' ? 110 : 120);
    const labelWidth = label.length * charWidth;
    const textWidth = Math.min(labelWidth, textMaxWidth);

    // Padding calculations based on style definitions
    let paddingX = 0;
    switch (type) {
      case 'person':
        paddingX = 24; // 12 + 12
        break;
      case 'system':
        paddingX = 66; // 50 (left) + 16 (right)
        break;
      case 'container':
        // Check for icon
        const iconName = this.getStyleValue(ele, 'style.icon', '');
        const hasIcon = iconName && typeof iconName === 'string' && iconName !== '';
        paddingX = hasIcon ? 52 : 24; // (40+12) or (12+12)
        break;
      case 'datastore':
      case 'queue':
        paddingX = 24; // 12 + 12
        break;
      default:
        paddingX = 24; // Default padding
    }

    const minWidth = type === 'system' ? 120 : 80;
    return Math.max(minWidth, textWidth + paddingX);
  }

  /**
   * Calculate node height based on label and type
   */
  private calculateNodeHeight(ele: any): number {
    const label = ele.data('label') || '';
    const type = ele.data('type');
    const charWidth = 7.5;
    const textMaxWidth = type === 'person' ? 100 : (type === 'system' ? 110 : 120);
    const lineHeight = 16;

    // Estimate lines
    const words = label.split(' ');
    let lineCount = 1;
    let currentLineLen = 0;

    // Simple word wrap estimation
    for (const word of words) {
      const wordLen = word.length * charWidth;
      if (currentLineLen + wordLen > textMaxWidth) {
        lineCount++;
        currentLineLen = wordLen;
      } else {
        currentLineLen += wordLen + charWidth; // + space
      }
    }

    // Padding calculations
    let paddingY = 0;
    switch (type) {
      case 'person':
        paddingY = 56; // 44 (top) + 12 (bottom)
        break;
      case 'system':
        paddingY = 32; // 16 + 16
        break;
      case 'container':
      case 'datastore':
      case 'queue':
        paddingY = 24; // 12 + 12
        break;
      default:
        paddingY = 24;
    }

    const minHeight = type === 'person' ? 70 : (type === 'system' ? 50 : 40);
    return Math.max(minHeight, (lineCount * lineHeight) + paddingY);
  }

  /**
   * Get default Cytoscape style
   */
  private getDefaultStyle(): any[] {
    // Premium C4 Model Color Palette (Modern Slate/Indigo Theme)
    const colors = {
      person: getCssVar('--color-primary-500'),
      personBorder: getCssVar('--color-primary-600'),
      system: getCssVar('--color-neutral-900'),
      systemBorder: getCssVar('--color-neutral-700'),
      container: Colors.background(),
      containerBorder: getCssVar('--color-neutral-700'),
      component: getCssVar('--color-neutral-100'),
      componentBorder: getCssVar('--color-neutral-400'),
      database: Colors.background(),
      databaseBorder: getCssVar('--color-neutral-600'),
      edge: Colors.neutral500(),
      edgeActive: Colors.info(),
      textDark: Colors.textPrimary(),
      textLight: '#ffffff',
    };

    const icons = this.getC4Icons();

    return [
      // Core Node Styling
      {
        selector: 'node',
        style: {
          'text-valign': 'center',
          'text-halign': 'center',
          'text-wrap': 'wrap',
          'text-max-width': 120,
          'font-family': 'Inter, system-ui, sans-serif',
          'font-size': 12,
          'font-weight': 500,
          'border-width': 2,
          'overlay-opacity': 0,
          // Removed invalid shadow CSS properties (shadow-blur, shadow-color, etc.)
          // Use filter: drop-shadow() in CSS if shadows are needed
          'transition-property': 'background-color, border-color, text-rotation',
          'transition-duration': 300,
          'width': (ele: any) => this.calculateNodeWidth(ele),
          'height': (ele: any) => this.calculateNodeHeight(ele),
          'label': 'data(label)',
          'text-overflow-wrap': 'anywhere',
        },
      },

      // Person - Icon at top, text below
      {
        selector: 'node[type="person"]',
        style: {
          'background-color': (ele: any) => this.getStyleValue(ele, 'style.color', colors.person) || colors.person,
          'border-color': colors.personBorder,
          'color': colors.textLight,
          'label': 'data(label)',
          'shape': (ele: any) => this.getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
          'padding': 12,
          'padding-top': 44, // Icon (20px) + icon margin (12px) + gap (8px) + text start (4px) = 44px
          'padding-bottom': 12,
          'padding-left': 12,
          'padding-right': 12,
          'text-max-width': 100,
          'background-image': (ele: any) => this.getIconFromMetadata(ele, icons.personIcon) || icons.personIcon,
          'background-width': '20px',
          'background-height': '20px',
          'background-position-x': '50%',
          'background-position-y': '12px',
          'background-repeat': 'no-repeat',
          'text-valign': 'bottom', // Text at bottom, icon at top
          'text-halign': 'center',
          'text-margin-y': 12, // Push text significantly down from icon
          'text-margin-x': 0,
          'text-background-padding': '2px',
          'font-weight': 600,
          'font-size': 11,
        },
      },

      // System - Icon on left, text on right (icon always shown for systems)
      {
        selector: 'node[type="system"]',
        style: {
          'background-color': (ele: any) => this.getStyleValue(ele, 'style.color', colors.system) || colors.system,
          'border-color': colors.systemBorder,
          'color': colors.textLight,
          'label': 'data(label)',
          'shape': (ele: any) => this.getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
          'padding': 16,
          'padding-left': 50, // Icon at 16px, icon width 18px = ends at 34px, + 16px gap = 50px
          'padding-right': 16,
          'padding-top': 16,
          'padding-bottom': 16,
          'text-max-width': 110,
          'background-image': (ele: any) => this.getIconFromMetadata(ele, icons.systemIcon) || icons.systemIcon,
          'background-width': '18px',
          'background-height': '18px',
          'background-position-x': '16px',
          'background-position-y': '50%',
          'background-repeat': 'no-repeat',
          'text-halign': 'left',
          'text-valign': 'center',
          'text-margin-x': 0, // No margin needed, padding-left handles spacing
          'text-margin-y': 0,
          'font-size': 14,
          'font-weight': 700,
          'text-transform': 'uppercase',
        },
      },

      // Container - No default icon, only show if set in metadata
      {
        selector: 'node[type="container"]',
        style: {
          'background-color': (ele: any) => this.getStyleValue(ele, 'style.color', colors.container) || colors.container,
          'border-color': colors.containerBorder,
          'color': colors.textDark,
          'label': 'data(label)',
          'shape': (ele: any) => this.getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
          'padding': 12,
          'padding-left': (ele: any) => {
            // Check if icon will be shown
            const iconName = this.getStyleValue(ele, 'style.icon', '');
            const hasIcon = iconName && typeof iconName === 'string' && iconName !== '';
            return hasIcon ? 40 : 12; // Extra padding only if icon present
          },
          'padding-right': 12,
          'padding-top': 12,
          'padding-bottom': 12,
          'text-max-width': 120,
          'background-image': (ele: any) => {
            // Only show icon if explicitly set in metadata, NO default icon
            const iconName = this.getStyleValue(ele, 'style.icon', '');
            if (!iconName || typeof iconName !== 'string') {
              return 'none'; // No default icon for containers
            }
            return this.getIconFromMetadata(ele, '');
          },
          'background-width': '16px',
          'background-height': '16px',
          'background-position-x': '12px',
          'background-position-y': '50%',
          'background-repeat': 'no-repeat',
          'text-halign': (ele: any) => {
            // Left align if icon present, center if no icon
            const iconName = this.getStyleValue(ele, 'style.icon', '');
            const hasIcon = iconName && typeof iconName === 'string' && iconName !== '';
            return hasIcon ? 'left' : 'center';
          },
          'text-valign': 'center',
          'text-margin-x': (ele: any) => {
            // Add margin only if icon present
            const iconName = this.getStyleValue(ele, 'style.icon', '');
            const hasIcon = iconName && typeof iconName === 'string' && iconName !== '';
            return hasIcon ? 8 : 0;
          },
          'text-margin-y': 0,
        },
      },

      // Datastore - Icon centered (if present), text centered
      {
        selector: 'node[type="datastore"]',
        style: {
          'background-color': (ele: any) => this.getStyleValue(ele, 'style.color', colors.database) || colors.database,
          'border-color': colors.databaseBorder,
          'color': colors.textDark,
          'label': 'data(label)',
          'shape': (ele: any) => this.getStyleValue(ele, 'style.shape', 'barrel') || 'barrel',
          'padding': 12,
          'padding-left': 12,
          'padding-right': 12,
          'padding-top': 12,
          'padding-bottom': 12,
          'text-max-width': 120,
          'border-width': 2,
          'background-image': (ele: any) => {
            const icon = this.getIconFromMetadata(ele, '');
            return icon && icon !== '' ? icon : 'none';
          },
          'background-width': '16px',
          'background-height': '16px',
          'background-position-x': '50%',
          'background-position-y': '30%', // Move icon up to make room for text
          'background-repeat': 'no-repeat',
          'text-halign': 'center',
          'text-valign': 'center',
          'text-margin-y': 8, // Push text down if icon present
          'text-margin-x': 0,
        },
      },

      // Queue - Icon centered (if present), text centered
      {
        selector: 'node[type="queue"]',
        style: {
          'background-color': (ele: any) => this.getStyleValue(ele, 'style.color', colors.container) || colors.container,
          'border-color': colors.containerBorder,
          'color': colors.textDark,
          'label': 'data(label)',
          'shape': (ele: any) => this.getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
          'padding': 12,
          'padding-left': 12,
          'padding-right': 12,
          'padding-top': 12,
          'padding-bottom': 12,
          'text-max-width': 120,
          'background-image': (ele: any) => {
            const icon = this.getIconFromMetadata(ele, '');
            return icon && icon !== '' ? icon : 'none';
          },
          'background-width': '16px',
          'background-height': '16px',
          'background-position-x': '50%',
          'background-position-y': '30%', // Move icon up to make room for text
          'background-repeat': 'no-repeat',
          'text-halign': 'center',
          'text-valign': 'center',
          'text-margin-y': 8, // Push text down if icon present
          'text-margin-x': 0,
        },
      },

      // Requirement
      {
        selector: 'node[type="requirement"]',
        style: {
          'display': 'none'
        },
      },

      // ADR
      {
        selector: 'node[type="adr"]',
        style: {
          'display': 'none'
        },
      },

      // Deployment Node
      {
        selector: 'node[type="deployment"]',
        style: {
          'background-color': '#f8fafc', // Slate 50
          'border-color': '#64748b', // Slate 500
          'color': '#334155', // Slate 700
          'label': 'data(label)',
          'shape': 'cut-rectangle',
          'padding': 16,
          'border-width': 2,
          'border-style': 'dashed',
        },
      },

      // Compound Nodes (Parent Systems)
      {
        selector: ':parent',
        style: {
          'background-color': '#f8fafc', // Slate 50
          'border-color': '#cbd5e1', // Slate 300
          'border-width': 1,
          'border-style': 'dashed',
          'label': 'data(label)',
          'text-valign': 'top',
          'text-halign': 'center',
          'text-margin-y': -10,
          'color': '#64748b', // Slate 500
          'font-size': 12,
          'font-weight': 600,
          'padding': 24,
          'text-transform': 'uppercase',
        },
      },

      {
        selector: '.collapsed',
        style: {
          'background-color': '#cbd5e1',
          'border-style': 'dashed',
          'label': 'data(label)',
          'text-valign': 'center',
          'text-halign': 'center',
          'width': 60,
          'height': 60
        }
      },
      // Edges
      {
        selector: 'edge',
        style: {
          'width': 2,
          'line-color': Colors.neutral500(),
          'target-arrow-color': Colors.neutral500(),
          'target-arrow-shape': 'triangle',
          'curve-style': 'bezier',
          'label': 'data(label)',
          'font-size': '10px',
          'color': colors.edge, // Kept original color for text
          'text-background-color': Colors.background(),
          'text-background-opacity': 1,
          'text-background-padding': '2px',
          'text-background-shape': 'round-rectangle', // Kept original
          'text-border-opacity': 0,
          'text-rotation': 'autorotate',
          'arrow-scale': 1.2,
          'opacity': 0.95,
        },
      },

      // Interaction States
      {
        selector: 'node:selected',
        style: {
          'border-width': 4,
          'border-color': colors.edgeActive,
          // Removed invalid shadow CSS properties
        },
      },
      {
        selector: 'edge:selected',
        style: {
          'line-color': colors.edgeActive,
          'target-arrow-color': colors.edgeActive,
          'color': colors.edgeActive,
          'width': 3,
        },
      },
      {
        selector: 'node:active',
        style: {
          'overlay-opacity': 0,
        },
      },
    ];
  }
  /**
   * Add a node to the graph
   */
  /**
   * Add a node to the graph
   */
  addNode(type: string, label: string, parentId?: string, extraData?: any): void {
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

    const node = {
      group: 'nodes',
      data: {
        id: uniqueId,
        label,
        type,
        parent: parentId,
        ...extraData
      }
    };

    this.cy.add(node as any);

    // Layout to position new node
    const layout = this.cy.layout({
      name: 'fcose',
      animate: true,
      animationDuration: 500,
      fit: false,
      randomize: false,
      nodeDimensionsIncludeLabels: true
    } as any);
    layout.run();
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
        duration: 500
      } as any);
    }
  }

  /**
   * Toggle collapse state of a compound node
   */
  toggleCollapse(id?: string): void {
    if (!this.cy) return;

    // Use provided ID or selected node
    let node;
    if (id) {
      node = this.cy.getElementById(id);
    } else {
      const selected = this.cy.$('node:selected');
      if (selected.length === 1) {
        node = selected[0];
      }
    }

    if (!node || node.length === 0) return;

    // Check if it's a compound node (has children)
    const children = node.children();
    if (children.length === 0) return;

    const isCollapsed = node.data('collapsed');

    if (isCollapsed) {
      // Expand
      node.data('collapsed', false);
      node.removeClass('collapsed');
      children.style('display', 'element');
      // Recursively show descendants if they were not explicitly collapsed?
      // For simplicity, just show direct children. 
      // If we want deep expand, we'd need more logic.
      // Let's just show all descendants for now to be safe, or just direct children.
      // If we just show direct children, their children might still be hidden if they were collapsed.
      // Let's just set display element for all descendants.
      node.descendants().style('display', 'element');
    } else {
      // Collapse
      node.data('collapsed', true);
      node.addClass('collapsed');
      node.descendants().style('display', 'none');
    }
  }

  /**
   * Set visualization level (1=System Context, 2=Container, 3=Component)
   */
  setLevel(level: number): void {
    if (!this.cy) return;

    const cy = this.cy;
    cy.batch(() => {
      const systems = cy.$('node[type="system"]');
      const containers = cy.$('node[type="container"]');

      if (level === 1) {
        // Collapse all systems
        systems.forEach(node => {
          node.data('collapsed', true);
          node.addClass('collapsed');
          node.descendants().style('display', 'none');
        });
      } else if (level === 2) {
        // Expand systems, collapse containers
        systems.forEach(node => {
          node.data('collapsed', false);
          node.removeClass('collapsed');
          node.children().style('display', 'element');
        });
        containers.forEach(node => {
          node.data('collapsed', true);
          node.addClass('collapsed');
          node.descendants().style('display', 'none');
        });
      } else if (level === 3) {
        // Expand everything
        systems.forEach(node => {
          node.data('collapsed', false);
          node.removeClass('collapsed');
          node.children().style('display', 'element');
        });
        containers.forEach(node => {
          node.data('collapsed', false);
          node.removeClass('collapsed');
          node.children().style('display', 'element');
        });
      }
    });
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
  exportPNG(options?: any): string {
    if (!this.cy) return '';

    return this.cy.png({
      output: 'base64uri',
      full: true,
      bg: Colors.background(),
      scale: options?.scale || 2
    });
  }

  /**
   * Export diagram as SVG
   */
  exportSVG(options?: any): string {
    return exportSVGUtil(this.cy, options);

    const scale = options?.scale || 1;
    // Always compute based on element bounding box and normalize to 0,0
    const bbox = this.cy.elements().boundingBox();
    const width = bbox.w * scale;
    const height = bbox.h * scale;
    const originX = bbox.x1;
    const originY = bbox.y1;

    // Start building SVG
    let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${bbox.w} ${bbox.h}">`;

    // Add defs section with styles and markers
    svg += '<defs>';
    svg += '<style type="text/css"><![CDATA[';
    svg += `.node-label { font-family: system-ui, sans-serif; font-size: 12px; fill: ${Colors.textPrimary()}; }`;
    svg += `.edge-label { font-family: system-ui, sans-serif; font-size: 10px; fill: ${Colors.neutral500()}; }`;
    svg += ']]></style>';
    svg += `<marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto"><polygon points="0 0, 10 3, 0 6" fill="${Colors.neutral500()}"/></marker>`;
    svg += '</defs>';

    // Background
    svg += `<rect x="0" y="0" width="${bbox.w}" height="${bbox.h}" fill="${Colors.background()}"/>`;

    // Export edges first (so they appear behind nodes)
    const edges = this.cy.edges();
    edges.forEach(edge => {
      const source = edge.source();
      const target = edge.target();
      const sourcePosRaw = source.position();
      const targetPosRaw = target.position();
      const sourcePos = { x: sourcePosRaw.x - originX, y: sourcePosRaw.y - originY };
      const targetPos = { x: targetPosRaw.x - originX, y: targetPosRaw.y - originY };

      // Calculate control points for bezier curve
      const dx = targetPos.x - sourcePos.x;
      const dy = targetPos.y - sourcePos.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const curvature = Math.min(dist * 0.3, 50);

      const controlX1 = sourcePos.x + dx * 0.5;
      const controlY1 = sourcePos.y - curvature;
      const controlX2 = targetPos.x - dx * 0.5;
      const controlY2 = targetPos.y - curvature;

      const path = `M ${sourcePos.x} ${sourcePos.y} C ${controlX1} ${controlY1}, ${controlX2} ${controlY2}, ${targetPos.x} ${targetPos.y}`;

      svg += `<path d="${path}" stroke="${Colors.neutral500()}" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>`;

      // Edge label
      const label = edge.data('label');
      if (label) {
        const midX = (sourcePos.x + targetPos.x) / 2;
        const midY = (sourcePos.y + targetPos.y) / 2 - curvature / 2;
        svg += `<text x="${midX}" y="${midY}" text-anchor="middle" class="edge-label">${this.escapeXml(label)}</text>`;
      }
    });

    // Export nodes
    const nodes = this.cy.nodes();
    nodes.forEach(node => {
      const posRaw = node.position();
      const pos = { x: posRaw.x - originX, y: posRaw.y - originY };
      const data = node.data();
      const width = node.width();
      const height = node.height();
      const type = data.type || 'node';

      // Get colors based on type
      const palette: Record<string, { bg: string; border: string; text: string }> = {
        person: { bg: getCssVar('--color-primary-500'), border: getCssVar('--color-primary-600'), text: Colors.background() },
        system: { bg: getCssVar('--color-neutral-900'), border: getCssVar('--color-neutral-700'), text: Colors.background() },
        container: { bg: Colors.background(), border: getCssVar('--color-neutral-700'), text: Colors.textPrimary() },
        datastore: { bg: Colors.background(), border: getCssVar('--color-neutral-600'), text: Colors.textPrimary() },
        queue: { bg: Colors.background(), border: getCssVar('--color-neutral-700'), text: Colors.textPrimary() }
      };

      const color = palette[type] || { bg: Colors.background(), border: Colors.neutral500(), text: Colors.textPrimary() };

      // Draw node shape (rounded rectangle)
      const rx = type === 'person' ? width / 2 : 8;
      svg += `<rect x="${pos.x - width / 2}" y="${pos.y - height / 2}" width="${width}" height="${height}" rx="${rx}" fill="${color.bg}" stroke="${color.border}" stroke-width="2"/>`;

      // Node label
      const label = data.label || data.id;
      if (label) {
        svg += `<text x="${pos.x}" y="${pos.y}" text-anchor="middle" dominant-baseline="middle" class="node-label" fill="${color.text}">${this.escapeXml(label)}</text>`;
      }
    });

    svg += '</svg>';
    return svg;
  }

  /**
   * Escape XML special characters
   */
  escapeXml(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&apos;');
  }

  /**
   * Export current graph to ArchitectureJSON
   */
  toJSON(): ArchitectureJSON {
    if (!this.data) return {} as ArchitectureJSON;

    // Clone the original data to avoid mutating it directly
    const json = JSON.parse(JSON.stringify(this.data));

    // Ensure metadata exists
    if (!json.metadata) {
      json.metadata = {};
    }

    // Add layout information to metadata
    (json.metadata as any).layout = this.getLayout();

    return json;
  }

  getLayout(): Record<string, LayoutData> {
    if (!this.cy) return {};

    const layout: Record<string, LayoutData> = {};

    this.cy.nodes().forEach(node => {
      const id = node.data('id');
      const pos = node.position();
      const width = node.width();
      const height = node.height();

      layout[id] = {
        x: Math.round(pos.x),
        y: Math.round(pos.y),
        width: Math.round(width),
        height: Math.round(height)
      };
    });

    return layout;
  }
}

/**
 * Factory function to create a viewer instance
 */
export function createViewer(options: ViewerOptions): ViewerInstance {
  return new SrujaViewer(options);
}
