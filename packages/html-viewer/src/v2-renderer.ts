// V2 Renderer - Mind-map style architecture diagram with expand/collapse
declare const lucide: {
    createIcons: (options: any) => void;
    createElement: (iconNode: any) => SVGElement;
    icons: any;
};

interface C4Colors {
    bg: string;
    border: string;
    text: string;
    headerBg: string;
}

const C4_COLORS: Record<string, C4Colors> = {
    person: {
        bg: '#08427b',
        border: '#073b6e',
        text: '#ffffff',
        headerBg: '#063264'
    },
    system: {
        bg: '#1168bd',
        border: '#0e5aa8',
        text: '#ffffff',
        headerBg: '#0d4f94'
    },
    container: {
        bg: '#438dd5',
        border: '#3a7fc0',
        text: '#ffffff',
        headerBg: '#3575ab'
    },
    component: {
        bg: '#85bbf0',
        border: '#6faee8',
        text: '#1a1a1a',
        headerBg: '#6faee8'
    },
    external: {
        bg: '#999999',
        border: '#888888',
        text: '#ffffff',
        headerBg: '#777777'
    },
    default: {
        bg: '#ffffff',
        border: '#cccccc',
        text: '#333333',
        headerBg: '#f0f0f0'
    }
};

const ICON_MAP: Record<string, string> = {
    'user': 'User',
    'server': 'Server',
    'box': 'Box',
    'code': 'Code',
    'database': 'Database',
    'layers': 'Layers',
    'help-circle': 'HelpCircle',
    'cloud': 'Cloud',
    'globe': 'Globe',
    'chevron-down': 'ChevronDown',
    'chevron-right': 'ChevronRight',
    'plus': 'Plus',
    'minus': 'Minus'
};

export class V2Renderer {
    private container: HTMLElement;
    private svgLayer: SVGSVGElement;
    private viewport: HTMLDivElement;
    private onNodeClick: ((nodeId: string) => void) | null = null;

    constructor(container: HTMLElement) {
        this.container = container;
        this.container.style.position = 'relative';
        this.container.style.overflow = 'auto';
        this.container.style.width = '100%';
        this.container.style.height = '100%';
        this.container.style.background = 'linear-gradient(135deg, #f5f7fa 0%, #e4e8ed 100%)';
        this.container.style.fontFamily = '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';

        // Viewport for panning/zooming
        this.viewport = document.createElement('div');
        this.viewport.style.position = 'relative';
        this.viewport.style.minWidth = '100%';
        this.viewport.style.minHeight = '100%';
        this.viewport.style.padding = '40px';
        this.viewport.style.boxSizing = 'border-box';
        this.container.appendChild(this.viewport);

        // SVG Layer for edges (underneath nodes)
        this.svgLayer = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        this.svgLayer.style.position = 'absolute';
        this.svgLayer.style.top = '0';
        this.svgLayer.style.left = '0';
        this.svgLayer.style.overflow = 'visible';
        this.svgLayer.style.pointerEvents = 'none';
        this.viewport.appendChild(this.svgLayer);

        // Add CSS animations
        this.addStyles();
    }

    setClickHandler(handler: (nodeId: string) => void) {
        this.onNodeClick = handler;
    }

    private addStyles() {
        const style = document.createElement('style');
        style.textContent = `
            @keyframes expandIn {
                from { opacity: 0; transform: scale(0.8); }
                to { opacity: 1; transform: scale(1); }
            }
            @keyframes fadeIn {
                from { opacity: 0; }
                to { opacity: 1; }
            }
            .v2-node {
                animation: expandIn 0.3s ease-out;
            }
            .v2-node:hover {
                z-index: 100;
            }
            .expand-btn {
                display: flex;
                align-items: center;
                justify-content: center;
                width: 24px;
                height: 24px;
                border-radius: 50%;
                border: 2px solid currentColor;
                background: white;
                cursor: pointer;
                transition: all 0.2s ease;
                font-weight: bold;
                font-size: 16px;
            }
            .expand-btn:hover {
                transform: scale(1.2);
                box-shadow: 0 2px 8px rgba(0,0,0,0.3);
            }
            .node-card {
                transition: all 0.2s ease;
            }
            .node-card:hover {
                transform: translateY(-2px);
                box-shadow: 0 8px 25px rgba(0,0,0,0.2) !important;
            }
            .children-container {
                animation: fadeIn 0.3s ease-out;
            }
        `;
        this.container.appendChild(style);
    }

    render(layoutData: any) {
        // Clear previous
        this.viewport.innerHTML = '';

        // Re-add style
        this.addStyles();

        // Re-add SVG layer
        this.viewport.appendChild(this.svgLayer);
        this.svgLayer.innerHTML = '';

        // Calculate bounds for viewport sizing
        let maxX = 0, maxY = 0;
        const calculateBounds = (nodes: any[]) => {
            nodes.forEach((n: any) => {
                maxX = Math.max(maxX, (n.x || 0) + (n.width || 0) + 100);
                maxY = Math.max(maxY, (n.y || 0) + (n.height || 0) + 100);
                if (n.children) calculateBounds(n.children);
            });
        };
        if (layoutData.children) calculateBounds(layoutData.children);

        this.viewport.style.width = `${maxX + 80}px`;
        this.viewport.style.height = `${maxY + 80}px`;
        this.svgLayer.setAttribute('width', `${maxX + 80}`);
        this.svgLayer.setAttribute('height', `${maxY + 80}`);

        // Add arrow marker definition
        this.addArrowMarker();

        // Render edges first (so they appear behind nodes)
        if (layoutData.edges) {
            this.renderEdges(layoutData.edges);
        }

        // Render nodes recursively
        if (layoutData.children) {
            this.renderNodes(layoutData.children, this.viewport, 0, 0);
        }
    }

    private addArrowMarker() {
        const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');

        const marker = document.createElementNS('http://www.w3.org/2000/svg', 'marker');
        marker.setAttribute('id', 'arrowhead');
        marker.setAttribute('markerWidth', '12');
        marker.setAttribute('markerHeight', '8');
        marker.setAttribute('refX', '11');
        marker.setAttribute('refY', '4');
        marker.setAttribute('orient', 'auto');
        marker.setAttribute('markerUnits', 'strokeWidth');

        const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        path.setAttribute('d', 'M0,0 L12,4 L0,8 L3,4 Z');
        path.setAttribute('fill', '#666');

        marker.appendChild(path);
        defs.appendChild(marker);
        this.svgLayer.appendChild(defs);
    }

    private renderEdges(edges: any[]) {
        edges.forEach((edge: any) => {
            if (!edge.sections) return;

            const group = document.createElementNS('http://www.w3.org/2000/svg', 'g');

            edge.sections.forEach((section: any) => {
                // Create curved path for more organic look
                const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
                let d = `M ${section.startPoint.x} ${section.startPoint.y}`;

                if (section.bendPoints && section.bendPoints.length > 0) {
                    // Use quadratic curves for smoother lines
                    section.bendPoints.forEach((bp: any, i: number) => {
                        if (i === 0) {
                            d += ` Q ${bp.x} ${bp.y}`;
                        } else {
                            d += ` ${bp.x} ${bp.y}`;
                        }
                    });
                    d += ` ${section.endPoint.x} ${section.endPoint.y}`;
                } else {
                    d += ` L ${section.endPoint.x} ${section.endPoint.y}`;
                }

                path.setAttribute('d', d);
                path.setAttribute('stroke', '#888888');
                path.setAttribute('stroke-width', '2');
                path.setAttribute('fill', 'none');
                path.setAttribute('marker-end', 'url(#arrowhead)');
                path.setAttribute('stroke-linecap', 'round');
                path.setAttribute('stroke-linejoin', 'round');
                group.appendChild(path);

                // Add edge label if present
                if (edge.labels && edge.labels[0]?.text) {
                    const midX = (section.startPoint.x + section.endPoint.x) / 2;
                    const midY = (section.startPoint.y + section.endPoint.y) / 2;

                    // Label background
                    const labelText = edge.labels[0].text;
                    const labelBg = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
                    const textWidth = labelText.length * 6 + 16;
                    labelBg.setAttribute('x', `${midX - textWidth / 2}`);
                    labelBg.setAttribute('y', `${midY - 12}`);
                    labelBg.setAttribute('width', `${textWidth}`);
                    labelBg.setAttribute('height', '24');
                    labelBg.setAttribute('fill', '#ffffff');
                    labelBg.setAttribute('rx', '12');
                    labelBg.setAttribute('stroke', '#dddddd');
                    labelBg.setAttribute('stroke-width', '1');
                    labelBg.setAttribute('filter', 'drop-shadow(0 1px 2px rgba(0,0,0,0.1))');
                    group.appendChild(labelBg);

                    // Label text
                    const label = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                    label.setAttribute('x', `${midX}`);
                    label.setAttribute('y', `${midY + 4}`);
                    label.setAttribute('text-anchor', 'middle');
                    label.setAttribute('font-size', '11');
                    label.setAttribute('font-family', '-apple-system, BlinkMacSystemFont, sans-serif');
                    label.setAttribute('fill', '#555555');
                    label.textContent = labelText;
                    group.appendChild(label);
                }
            });

            this.svgLayer.appendChild(group);
        });
    }

    private renderNodes(nodes: any[], parent: HTMLElement, offsetX: number, offsetY: number) {
        nodes.forEach((node: any) => {
            const el = this.createNodeElement(node, offsetX, offsetY);
            parent.appendChild(el);

            // Recursively render children inside this node
            if (node.children && node.children.length > 0 && node.expanded !== false) {
                const headerHeight = 48;
                this.renderNodes(node.children, el, (node.x || 0) + offsetX, (node.y || 0) + offsetY + headerHeight);
            }
        });
    }

    private createNodeElement(node: any, offsetX: number, offsetY: number): HTMLElement {
        const type = node.type || 'default';
        const colors = C4_COLORS[type] || C4_COLORS.default;
        const hasChildren = node.hasChildren || (node.children && node.children.length > 0);
        const isExpanded = node.expanded !== false;

        const el = document.createElement('div');
        el.className = `v2-node v2-node-${type} node-card`;
        el.setAttribute('data-id', node.id);
        el.setAttribute('data-type', type);
        el.setAttribute('data-has-children', hasChildren ? 'true' : 'false');
        el.setAttribute('data-expanded', isExpanded ? 'true' : 'false');

        // Positioning
        el.style.position = 'absolute';
        el.style.left = `${(node.x || 0) + offsetX}px`;
        el.style.top = `${(node.y || 0) + offsetY}px`;
        el.style.width = `${node.width || 200}px`;
        el.style.height = `${node.height || 120}px`;
        el.style.zIndex = type === 'person' ? '10' : (type === 'system' ? '5' : '1');

        // Base styling
        el.style.borderRadius = type === 'person' ? '60px 60px 16px 16px' : '16px';
        el.style.overflow = 'hidden';
        el.style.boxShadow = '0 4px 15px rgba(0,0,0,0.15)';

        if (hasChildren && isExpanded) {
            // Expanded container
            el.style.background = '#f8f9fa';
            el.style.border = `3px solid ${colors.border}`;
        } else {
            // Leaf node or collapsed
            el.style.background = `linear-gradient(180deg, ${colors.bg} 0%, ${this.darkenColor(colors.bg, 15)} 100%)`;
            el.style.border = `2px solid ${colors.border}`;
        }

        // Create header
        const header = document.createElement('div');
        header.style.display = 'flex';
        header.style.alignItems = 'center';
        header.style.gap = '10px';
        header.style.padding = '12px 16px';
        header.style.background = hasChildren && isExpanded ? colors.headerBg : 'transparent';
        header.style.color = hasChildren && isExpanded ? colors.text : (type === 'component' ? colors.text : colors.text);
        header.style.cursor = hasChildren ? 'pointer' : 'default';

        // Expand/collapse button for nodes with children
        if (hasChildren) {
            const expandBtn = document.createElement('div');
            expandBtn.className = 'expand-btn';
            expandBtn.style.color = hasChildren && isExpanded ? colors.text : colors.bg;
            expandBtn.style.background = hasChildren && isExpanded ? 'rgba(255,255,255,0.2)' : 'white';
            expandBtn.style.borderColor = hasChildren && isExpanded ? colors.text : colors.bg;
            expandBtn.style.flexShrink = '0';
            expandBtn.innerHTML = isExpanded ? '−' : '+';
            expandBtn.title = isExpanded ? 'Collapse' : 'Expand';

            expandBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                if (this.onNodeClick) {
                    this.onNodeClick(node.id);
                }
            });

            header.appendChild(expandBtn);
        }

        // Icon
        const icon = this.createIcon(type, hasChildren && isExpanded ? colors.text : colors.text);
        if (icon) {
            icon.style.flexShrink = '0';
            header.appendChild(icon);
        }

        // Title
        const title = document.createElement('div');
        title.style.flex = '1';
        title.style.fontWeight = '600';
        title.style.fontSize = '14px';
        title.style.overflow = 'hidden';
        title.style.textOverflow = 'ellipsis';
        title.style.whiteSpace = 'nowrap';
        title.textContent = node.labels?.[0]?.text || node.id;
        header.appendChild(title);

        el.appendChild(header);

        // For leaf/collapsed nodes, add centered content
        if (!hasChildren || !isExpanded) {
            const content = document.createElement('div');
            content.style.flex = '1';
            content.style.display = 'flex';
            content.style.flexDirection = 'column';
            content.style.alignItems = 'center';
            content.style.justifyContent = 'center';
            content.style.padding = '8px 16px';
            content.style.color = colors.text;

            // Type badge
            const badge = document.createElement('div');
            badge.style.fontSize = '10px';
            badge.style.opacity = '0.8';
            badge.style.textTransform = 'uppercase';
            badge.style.letterSpacing = '1px';
            badge.style.marginTop = '4px';
            badge.style.padding = '2px 8px';
            badge.style.background = 'rgba(255,255,255,0.15)';
            badge.style.borderRadius = '4px';
            badge.textContent = hasChildren && !isExpanded ? `${type} ▸` : type;
            content.appendChild(badge);

            el.appendChild(content);
            el.style.display = 'flex';
            el.style.flexDirection = 'column';
        }

        // Make header clickable for expand/collapse
        if (hasChildren) {
            header.addEventListener('click', () => {
                if (this.onNodeClick) {
                    this.onNodeClick(node.id);
                }
            });
        }

        return el;
    }

    private createIcon(type: string, color: string): HTMLElement | null {
        const iconName = this.getIconName(type);
        const lucideName = ICON_MAP[iconName] || 'Box';

        if (typeof lucide === 'undefined' || !lucide.icons || !lucide.icons[lucideName]) {
            return null;
        }

        try {
            const svgEl = lucide.createElement(lucide.icons[lucideName]);
            svgEl.setAttribute('width', '20');
            svgEl.setAttribute('height', '20');
            svgEl.style.color = color;
            return svgEl as unknown as HTMLElement;
        } catch {
            return null;
        }
    }

    private getIconName(type: string): string {
        switch (type) {
            case 'person': return 'user';
            case 'system': return 'server';
            case 'container': return 'box';
            case 'component': return 'code';
            case 'database':
            case 'datastore': return 'database';
            case 'external': return 'globe';
            default: return 'box';
        }
    }

    private darkenColor(hex: string, percent: number): string {
        const num = parseInt(hex.replace('#', ''), 16);
        const amt = Math.round(2.55 * percent);
        const R = Math.max(0, (num >> 16) - amt);
        const G = Math.max(0, ((num >> 8) & 0x00FF) - amt);
        const B = Math.max(0, (num & 0x0000FF) - amt);
        return `#${(1 << 24 | R << 16 | G << 8 | B).toString(16).slice(1)}`;
    }
}
