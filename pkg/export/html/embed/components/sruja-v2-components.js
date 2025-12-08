// packages/html-viewer - Bundled web components
// Auto-generated from TypeScript sources

// === sruja-node-index.js ===
// packages/html-viewer/src/sruja-node-index.ts
// Node indexing and mapping service
class SrujaNodeIndex extends HTMLElement {
    constructor() {
        super(...arguments);
        Object.defineProperty(this, "nodeIndex", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: new Map()
        });
        Object.defineProperty(this, "edges", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: []
        });
        Object.defineProperty(this, "arch", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
    }
    connectedCallback() {
        this.loadData();
    }
    loadData() {
        const dataEl = document.getElementById('sruja-data');
        if (!dataEl) {
            console.error('SrujaNodeIndex: #sruja-data element not found!');
            return;
        }
        const jsonData = JSON.parse(dataEl.textContent || '{}');
        this.arch = jsonData.architecture || {};
        this.buildIndex();
    }
    buildIndex() {
        this.nodeIndex.clear();
        this.edges = [];
        if (!this.arch)
            return;
        // Index all nodes
        (this.arch.persons || []).forEach(p => {
            const name = p.label || p.name || p.id;
            this.indexNode(p.id, 'person', name, p.description || '');
        });
        (this.arch.systems || []).forEach(s => {
            const sysName = s.label || s.name || s.id;
            this.indexNode(s.id, 'system', sysName, s.description || '');
            (s.containers || []).forEach(c => {
                const contName = c.label || c.name || c.id;
                this.indexNode(c.id, 'container', contName, c.description || '');
                (c.components || []).forEach(comp => {
                    const compName = comp.label || comp.name || comp.id;
                    this.indexNode(comp.id, 'component', compName, comp.description || '');
                });
            });
            (s.datastores || []).forEach(d => {
                const dsName = d.label || d.name || d.id;
                this.indexNode(d.id, 'datastore', dsName, d.description || '');
            });
            (s.queues || []).forEach(q => {
                const qName = q.label || q.name || q.id;
                this.indexNode(q.id, 'queue', qName, q.description || '');
            });
        });
        (this.arch.requirements || []).forEach(r => {
            const desc = r.description || r.title || '';
            this.indexNode(r.id, 'requirement', r.title || r.id, desc);
        });
        (this.arch.adrs || []).forEach(a => {
            const desc = a.decision || a.context || a.title || '';
            this.indexNode(a.id, 'adr', a.title || a.id, desc);
        });
        (this.arch.scenarios || []).forEach(s => {
            const desc = s.description || s.title || '';
            this.indexNode(s.id, 'scenario', s.title || s.label || s.id, desc);
        });
        (this.arch.flows || []).forEach(f => {
            const desc = f.description || f.title || '';
            this.indexNode(f.id, 'flow', f.title || f.label || f.id, desc);
        });
        (this.arch.deployment || []).forEach(d => {
            this.indexNode(d.id, 'deployment', d.label || d.id, '');
        });
        // Index edges
        this.indexEdges(this.arch.systems || [], 'system');
        if (this.arch.persons) {
            this.arch.persons.forEach(p => {
                if (p.relations) {
                    p.relations.forEach(rel => {
                        this.edges.push({
                            from: p.id,
                            to: rel.target,
                            label: rel.label || '',
                            type: rel.type || 'relation'
                        });
                    });
                }
            });
        }
        this.dispatchEvent(new CustomEvent('index-ready', {
            detail: { nodeIndex: this.nodeIndex, edges: this.edges }
        }));
    }
    indexNode(id, type, name, description) {
        this.nodeIndex.set(id, { id, type, name, description });
    }
    indexEdges(items, _parentType) {
        items.forEach(item => {
            if (item.relations) {
                item.relations.forEach(rel => {
                    this.edges.push({
                        from: item.id,
                        to: rel.target,
                        label: rel.label || '',
                        type: rel.type || 'relation'
                    });
                });
            }
            if ('containers' in item && item.containers) {
                this.indexEdges(item.containers, 'container');
            }
            if ('components' in item && item.components) {
                this.indexEdges(item.components, 'component');
            }
        });
    }
    getNode(id) {
        return this.nodeIndex.get(id);
    }
    getNodeType(nodeId) {
        const node = this.nodeIndex.get(nodeId);
        return node ? node.type : null;
    }
    getEdges() {
        return this.edges;
    }
    getArchitecture() {
        return this.arch;
    }
    mapSvgNodesToIds(targetSvg) {
        if (!targetSvg)
            return;
        const allGroups = targetSvg.querySelectorAll('g[role="img"]');
        const nodeGroups = Array.from(allGroups).filter(group => {
            if (!group.classList.contains('node'))
                return false;
            if (group.querySelector('line') || group.querySelector('path[marker-end]'))
                return false;
            const ariaLabel = group.getAttribute('aria-label');
            if (ariaLabel && (ariaLabel.includes(' to ') || ariaLabel.includes(' -> ')))
                return false;
            return true;
        });
        nodeGroups.forEach(group => {
            const ariaLabel = group.getAttribute('aria-label');
            const titleEl = group.querySelector('title');
            const title = titleEl ? titleEl.textContent : '';
            const searchText = title || ariaLabel;
            if (!searchText)
                return;
            let bestMatch = null;
            let bestScore = 0;
            // Exact matches
            for (const [nodeId, node] of this.nodeIndex.entries()) {
                const nodeName = node.name || '';
                const nodeIdStr = nodeId || '';
                if (searchText === nodeName || searchText === nodeIdStr) {
                    bestMatch = nodeId;
                    bestScore = 100;
                    break;
                }
            }
            // Normalized matching
            if (!bestMatch) {
                const normalizedSearch = this.normalizeString(searchText);
                for (const [nodeId, node] of this.nodeIndex.entries()) {
                    const nodeName = node.name || '';
                    const nodeIdStr = nodeId || '';
                    const normalizedName = this.normalizeString(nodeName);
                    const normalizedId = this.normalizeString(nodeIdStr);
                    let score = 0;
                    if (normalizedSearch === normalizedName || normalizedSearch === normalizedId) {
                        score = 95;
                    }
                    else if (normalizedSearch.replace(/\s+/g, '') === normalizedName.replace(/\s+/g, '') ||
                        normalizedSearch.replace(/\s+/g, '') === normalizedId.replace(/\s+/g, '')) {
                        score = 90;
                    }
                    if (score > bestScore || (score === bestScore && nodeIdStr.length > (bestMatch ? bestMatch.length : 0))) {
                        bestScore = score;
                        bestMatch = nodeId;
                    }
                }
            }
            if (bestMatch && bestScore >= 80) {
                group.setAttribute('data-node-id', bestMatch);
            }
        });
    }
    normalizeString(str) {
        if (!str || typeof str !== 'string')
            return '';
        return str.toLowerCase().replace(/\s+/g, '').replace(/^(the|a|an)\s+/i, '');
    }
    matchesSearch(nodeId, query) {
        if (!query)
            return true;
        const node = this.nodeIndex.get(nodeId);
        if (!node)
            return false;
        const q = query.toLowerCase();
        const idMatch = node.id.toLowerCase().includes(q);
        const nameMatch = node.name.toLowerCase().includes(q);
        const descMatch = !!(node.description && node.description.toLowerCase().includes(q));
        return idMatch || nameMatch || descMatch;
    }
}
customElements.define('sruja-node-index', SrujaNodeIndex);


// === v2-layout.js ===
// import ELK from 'elkjs/lib/elk.bundled.js';
class V2Layout {
    constructor() {
        Object.defineProperty(this, "elk", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        // @ts-expect-error - ELK is loaded globally via script tag
        if (typeof ELK === 'undefined') {
            throw new Error("ELK is not defined. elkjs library not loaded?");
        }
        // @ts-expect-error - ELK is loaded globally via script tag
        this.elk = new ELK();
    }
    async layout(graph) {
        const elkGraph = {
            id: 'root',
            layoutOptions: {
                'elk.algorithm': 'layered',
                'elk.direction': 'DOWN',
                'elk.spacing.nodeNode': '50',
                'elk.layered.spacing.nodeNodeBetweenLayers': '50',
                'elk.padding': '[top=50,left=50,bottom=50,right=50]',
            },
            children: (graph.children || graph.nodes || []).map((n) => ({
                id: n.id,
                width: 250, // Default width
                height: 150, // Default height
                labels: [{ text: n.label }],
                type: n.type, // Pass type through
                layoutOptions: {
                    'elk.portConstraints': 'FIXED_SIDE'
                },
                // Recursively handle children if they exist (for containers)
                children: n.children ? n.children.map((c) => ({
                    id: c.id,
                    width: 150,
                    height: 100,
                    labels: [{ text: c.labels?.[0]?.text || c.id }],
                    type: c.type // Pass type through
                })) : []
            })),
            edges: (graph.edges || []).map((e) => ({
                id: e.id,
                sources: e.sources || [e.source],
                targets: e.targets || [e.target]
            }))
        };
        const layouted = await this.elk.layout(elkGraph);
        return layouted;
    }
}


// === v2-renderer.js ===
const C4_COLORS = {
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
const ICON_MAP = {
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
class V2Renderer {
    constructor(container) {
        Object.defineProperty(this, "container", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "svgLayer", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "viewport", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "onNodeClick", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
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
    setClickHandler(handler) {
        this.onNodeClick = handler;
    }
    addStyles() {
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
    render(layoutData) {
        // Clear previous
        this.viewport.innerHTML = '';
        // Re-add style
        this.addStyles();
        // Re-add SVG layer
        this.viewport.appendChild(this.svgLayer);
        this.svgLayer.innerHTML = '';
        // Calculate bounds for viewport sizing
        let maxX = 0, maxY = 0;
        const calculateBounds = (nodes) => {
            nodes.forEach((n) => {
                maxX = Math.max(maxX, (n.x || 0) + (n.width || 0) + 100);
                maxY = Math.max(maxY, (n.y || 0) + (n.height || 0) + 100);
                if (n.children)
                    calculateBounds(n.children);
            });
        };
        if (layoutData.children)
            calculateBounds(layoutData.children);
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
    addArrowMarker() {
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
    renderEdges(edges) {
        edges.forEach((edge) => {
            if (!edge.sections)
                return;
            const group = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            edge.sections.forEach((section) => {
                // Create curved path for more organic look
                const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
                let d = `M ${section.startPoint.x} ${section.startPoint.y}`;
                if (section.bendPoints && section.bendPoints.length > 0) {
                    // Use quadratic curves for smoother lines
                    section.bendPoints.forEach((bp, i) => {
                        if (i === 0) {
                            d += ` Q ${bp.x} ${bp.y}`;
                        }
                        else {
                            d += ` ${bp.x} ${bp.y}`;
                        }
                    });
                    d += ` ${section.endPoint.x} ${section.endPoint.y}`;
                }
                else {
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
    renderNodes(nodes, parent, offsetX, offsetY) {
        nodes.forEach((node) => {
            const el = this.createNodeElement(node, offsetX, offsetY);
            parent.appendChild(el);
            // Recursively render children inside this node
            if (node.children && node.children.length > 0 && node.expanded !== false) {
                const headerHeight = 48;
                this.renderNodes(node.children, el, (node.x || 0) + offsetX, (node.y || 0) + offsetY + headerHeight);
            }
        });
    }
    createNodeElement(node, offsetX, offsetY) {
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
        }
        else {
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
    createIcon(type, color) {
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
            return svgEl;
        }
        catch {
            return null;
        }
    }
    getIconName(type) {
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
    darkenColor(hex, percent) {
        const num = parseInt(hex.replace('#', ''), 16);
        const amt = Math.round(2.55 * percent);
        const R = Math.max(0, (num >> 16) - amt);
        const G = Math.max(0, ((num >> 8) & 0x00FF) - amt);
        const B = Math.max(0, (num & 0x0000FF) - amt);
        return `#${(1 << 24 | R << 16 | G << 8 | B).toString(16).slice(1)}`;
    }
}


// === v2-interaction.js ===
class V2Interaction {
    constructor(container, onNodeClick) {
        Object.defineProperty(this, "container", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "onNodeClick", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        this.container = container;
        this.onNodeClick = onNodeClick;
        this.attachListeners();
    }
    attachListeners() {
        this.container.addEventListener('click', (e) => {
            const target = e.target;
            const node = target.closest('.v2-node');
            if (node) {
                const nodeId = node.getAttribute('data-id');
                if (nodeId) {
                    this.onNodeClick(nodeId);
                }
            }
        });
    }
}


// === v2-viewer.js ===
class SrujaV2Viewer extends HTMLElement {
    constructor() {
        super();
        Object.defineProperty(this, "layoutEngine", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "renderer", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        // @ts-expect-error - V2Interaction is initialized in constructor
        Object.defineProperty(this, "interaction", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "nodeIndex", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "expandedNodes", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: new Set()
        });
        this.attachShadow({ mode: 'open' });
        // Create container
        const container = document.createElement('div');
        container.id = 'sruja-v2-container';
        container.style.width = '100vw'; // Use vw/vh to be sure
        container.style.height = '100vh';
        container.style.overflow = 'hidden';
        container.innerText = "Loading Architecture...";
        container.style.display = 'flex';
        container.style.justifyContent = 'center';
        container.style.alignItems = 'center';
        this.shadowRoot.appendChild(container);
        this.layoutEngine = new V2Layout();
        this.renderer = new V2Renderer(container);
        this.renderer.setClickHandler(this.handleNodeClick.bind(this)); // Connect click handler
        this.interaction = new V2Interaction(container, this.handleNodeClick.bind(this));
        this.nodeIndex = new SrujaNodeIndex();
        // Listen early to avoid race conditions
        this.nodeIndex.addEventListener('index-ready', () => {
            this.renderGraph();
        });
        // We don't append nodeIndex to DOM as it reads from document.body
        // But we need to trigger its data loading.
        // Since it uses connectedCallback, we might need to append it temporarily or calling loadData manually (if we made it public)
        // For now, let's append it to shadow root, it won't be visible.
        this.shadowRoot.appendChild(this.nodeIndex);
    }
    connectedCallback() {
    }
    async renderGraph() {
        const arch = this.nodeIndex.getArchitecture();
        if (!arch)
            return;
        // Transform simplified architecture to ELK graph
        const nodes = [];
        const edges = [];
        // Add Systems
        if (arch.systems) {
            arch.systems.forEach(sys => {
                const isSystemExpanded = this.expandedNodes.has(sys.id);
                const hasContainers = sys.containers && sys.containers.length > 0;
                const sysNode = {
                    id: sys.id,
                    width: isSystemExpanded ? 400 : 220,
                    height: isSystemExpanded ? 300 : 120,
                    labels: [{ text: sys.name || sys.label || sys.id }],
                    layoutOptions: { 'elk.padding': '[top=60,left=20,bottom=20,right=20]' },
                    children: [],
                    type: 'system',
                    expanded: isSystemExpanded,
                    hasChildren: hasContainers
                };
                // Add Containers (L2) - ONLY if expanded
                if (sys.containers && isSystemExpanded) {
                    sys.containers.forEach(cont => {
                        if (cont.id) {
                            const isContainerExpanded = this.expandedNodes.has(cont.id);
                            const hasComponents = cont.components && cont.components.length > 0;
                            const contNode = {
                                id: cont.id,
                                width: isContainerExpanded ? 300 : 180,
                                height: isContainerExpanded ? 200 : 100,
                                labels: [{ text: cont.name || cont.label || cont.id }],
                                type: 'container',
                                children: [],
                                expanded: isContainerExpanded,
                                hasChildren: hasComponents
                            };
                            // Add Components (L3) - ONLY if container expanded
                            if (cont.components && isContainerExpanded) {
                                cont.components.forEach((comp) => {
                                    if (comp.id) {
                                        contNode.children.push({
                                            id: comp.id,
                                            width: 140,
                                            height: 80,
                                            labels: [{ text: comp.name || comp.label || comp.id }],
                                            type: 'component',
                                            hasChildren: false
                                        });
                                    }
                                });
                            }
                            sysNode.children.push(contNode);
                        }
                    });
                }
                if (sysNode.id) {
                    nodes.push(sysNode);
                }
            });
        }
        // Add Persons
        if (arch.persons) {
            arch.persons.forEach(p => {
                if (p.id) {
                    nodes.push({
                        id: p.id,
                        width: 150,
                        height: 100,
                        labels: [{ text: p.name || p.label || p.id }],
                        type: 'person'
                    });
                }
            });
        }
        // Add Edges from NodeIndex
        const indexEdges = this.nodeIndex.getEdges();
        indexEdges.forEach(e => {
            if (e.from && e.to) {
                // Filter edges based on visibility of source/target
                // This is tricky. ELK might handle edges to hidden nodes if they are inside collapsed parents IF the parent is the port?
                // For now, let's just add all edges and let ELK/Renderer handle or ignore.
                // Better approach: Check if endpoints are visible or their parents are visible.
                // If a node is inside a collapsed system, the edge should point to the system?
                // For a first pass interactivity, simple visibility check might be enough.
                edges.push({
                    id: `e_${e.from}_${e.to}_${Math.random().toString(36).substr(2, 5)}`,
                    sources: [e.from],
                    targets: [e.to],
                    labels: [{ text: e.label }] // ELK supports edge labels
                });
            }
        });
        const graph = {
            id: 'root',
            layoutOptions: {
                'elk.algorithm': 'layered',
                'elk.direction': 'DOWN',
                'elk.spacing.nodeNode': '80',
                'elk.layered.spacing.nodeNodeBetweenLayers': '80',
            },
            children: nodes,
            edges: edges
        };
        try {
            const layoutedGraph = await this.layoutEngine.layout(graph);
            this.renderer.render(layoutedGraph);
            // Re-attach listeners because renderer clears HTML
            // Actually V2Interaction manages event delegation on the container, 
            // but the container content is wiped. Interaction listener is on 'container' which is persistent.
            // So clicks should still work if target elements exist.
        }
        catch (e) {
            console.error('Layout failed:', e);
            const container = this.shadowRoot.getElementById('sruja-v2-container');
            if (container) {
                container.innerText = `Error rendering graph: ${e}`;
                container.style.color = 'red';
                container.style.display = 'flex';
                container.style.justifyContent = 'center';
                container.style.alignItems = 'center';
            }
        }
    }
    handleNodeClick(nodeId) {
        if (this.expandedNodes.has(nodeId)) {
            this.expandedNodes.delete(nodeId);
        }
        else {
            this.expandedNodes.add(nodeId);
        }
        this.renderGraph();
    }
}
customElements.define('sruja-v2-viewer', SrujaV2Viewer);


