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


// === sruja-svg-viewer.js ===
"use strict";
// packages/html-viewer/src/sruja-svg-viewer.ts
// SVG viewer with pan and zoom functionality
class SrujaSvgViewer extends HTMLElement {
    constructor() {
        super(...arguments);
        Object.defineProperty(this, "scale", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 1
        });
        Object.defineProperty(this, "panX", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 0
        });
        Object.defineProperty(this, "panY", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 0
        });
        Object.defineProperty(this, "isDragging", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: false
        });
        Object.defineProperty(this, "startX", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 0
        });
        Object.defineProperty(this, "startY", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 0
        });
        Object.defineProperty(this, "container", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
    }
    connectedCallback() {
        this.container = this.closest('#container');
        if (!this.container)
            return;
        this.setupPanZoom();
        this.setupZoomControls();
    }
    setupPanZoom() {
        if (!this.container)
            return;
        this.container.addEventListener('mousedown', (e) => {
            if (e.button === 0 && !e.target.closest('.sidebar') && !e.target.closest('.controls')) {
                this.isDragging = true;
                this.container.classList.add('dragging');
                this.startX = e.clientX - this.panX;
                this.startY = e.clientY - this.panY;
            }
        });
        this.container.addEventListener('mousemove', (e) => {
            if (this.isDragging) {
                this.panX = e.clientX - this.startX;
                this.panY = e.clientY - this.startY;
                this.updateTransform();
            }
        });
        this.container.addEventListener('mouseup', () => {
            this.isDragging = false;
            if (this.container)
                this.container.classList.remove('dragging');
        });
        this.container.addEventListener('mouseleave', () => {
            this.isDragging = false;
            if (this.container)
                this.container.classList.remove('dragging');
        });
        this.container.addEventListener('wheel', (e) => {
            // Only zoom when Ctrl/Cmd is held, otherwise allow normal scrolling
            if (e.ctrlKey || e.metaKey) {
                e.preventDefault();
                const delta = e.deltaY > 0 ? 0.9 : 1.1;
                this.scale = Math.max(0.1, Math.min(5, this.scale * delta));
                this.updateTransform();
            }
            // If not holding Ctrl/Cmd, allow default scroll behavior
        });
    }
    setupZoomControls() {
        window.zoomIn = () => {
            this.scale = Math.min(5, this.scale * 1.2);
            this.updateTransform();
        };
        window.zoomOut = () => {
            this.scale = Math.max(0.1, this.scale / 1.2);
            this.updateTransform();
        };
        window.resetZoom = () => {
            this.scale = 1;
            this.panX = 0;
            this.panY = 0;
            this.updateTransform();
        };
    }
    updateTransform() {
        const activeContainer = document.querySelector('.svg-container-level.active') ||
            document.getElementById('svg-container-all');
        const gridContainer = document.getElementById('svg-grid-container');
        if (activeContainer && !gridContainer) {
            activeContainer.style.transform = `translate(${this.panX}px, ${this.panY}px) scale(${this.scale})`;
        }
    }
    zoomToNode(nodeEl, svg) {
        if (!nodeEl || !svg || !this.container)
            return;
        const activeContainer = document.querySelector('.svg-container-level.active') ||
            document.getElementById('svg-container-all');
        if (!activeContainer)
            return;
        // Get container viewport dimensions (accounting for padding-top)
        const containerRect = this.container.getBoundingClientRect();
        const viewportCenterX = containerRect.width / 2;
        const viewportCenterY = (containerRect.height - 48) / 2 + 48; // Account for 48px top padding
        // Get current transform
        const computedStyle = window.getComputedStyle(activeContainer);
        const transform = computedStyle.transform;
        let currentPanX = this.panX;
        let currentPanY = this.panY;
        if (transform && transform !== 'none') {
            const matrix = new DOMMatrix(transform);
            currentPanX = matrix.e;
            currentPanY = matrix.f;
        }
        // Calculate based on where the node currently appears on screen
        const nodeScreenRect = nodeEl.getBoundingClientRect();
        const nodeScreenCenterX = nodeScreenRect.left + nodeScreenRect.width / 2;
        const nodeScreenCenterY = nodeScreenRect.top + nodeScreenRect.height / 2;
        // Calculate offset needed to move node to viewport center
        const containerScreenCenterX = containerRect.left + viewportCenterX;
        const containerScreenCenterY = containerRect.top + viewportCenterY;
        const offsetX = containerScreenCenterX - nodeScreenCenterX;
        const offsetY = containerScreenCenterY - nodeScreenCenterY;
        // Convert screen offset to pan offset
        // The transform affects the container, so we add the offset directly
        this.panX = currentPanX + offsetX;
        this.panY = currentPanY + offsetY;
        this.updateTransform();
    }
    reset() {
        this.scale = 1;
        this.panX = 0;
        this.panY = 0;
        this.updateTransform();
    }
}
customElements.define('sruja-svg-viewer', SrujaSvgViewer);


// === sruja-info-panel.js ===
"use strict";
// packages/html-viewer/src/sruja-info-panel.ts
// Info panel component for displaying node details
class SrujaInfoPanel extends HTMLElement {
    constructor() {
        super(...arguments);
        Object.defineProperty(this, "nodeIndex", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
        Object.defineProperty(this, "arch", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
        Object.defineProperty(this, "titleEl", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
        Object.defineProperty(this, "descEl", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
    }
    connectedCallback() {
        const indexEl = document.querySelector('sruja-node-index');
        if (indexEl) {
            this.nodeIndex = indexEl;
            this.arch = indexEl.getArchitecture();
        }
        this.titleEl = this.querySelector('#info-title');
        this.descEl = this.querySelector('#info-description');
    }
    showNode(nodeId) {
        if (!this.nodeIndex || !nodeId) {
            this.hide();
            return;
        }
        const node = this.nodeIndex.getNode(nodeId);
        if (!node) {
            this.hide();
            return;
        }
        let title = node.name;
        let desc = node.description || nodeId;
        // Enhanced info for different types
        if (node.type === 'requirement' && this.arch?.requirements) {
            const req = this.arch.requirements.find(r => r.id === nodeId);
            if (req) {
                title = `${req.id} (${req.type || 'requirement'})`;
                desc = req.title || req.description || '';
            }
        }
        else if (node.type === 'adr' && this.arch?.adrs) {
            const adr = this.arch.adrs.find(a => a.id === nodeId);
            if (adr) {
                title = `${adr.id}${adr.status ? ' [' + adr.status + ']' : ''}`;
                desc = adr.decision || adr.context || adr.title || '';
                if (adr.consequences) {
                    desc += '\n\nConsequences: ' + adr.consequences;
                }
            }
        }
        else if (node.type === 'scenario' && this.arch?.scenarios) {
            const scenario = this.arch.scenarios.find(s => s.id === nodeId);
            if (scenario) {
                title = scenario.title || scenario.label || scenario.id;
                desc = scenario.description || '';
                if (scenario.steps && scenario.steps.length > 0) {
                    desc += '\n\nSteps:\n';
                    scenario.steps.forEach((step, idx) => {
                        const stepDesc = step.description || '';
                        desc += `${idx + 1}. ${step.from} → ${step.to}${stepDesc ? ': ' + stepDesc : ''}\n`;
                    });
                }
            }
        }
        else if (node.type === 'flow' && this.arch?.flows) {
            const flow = this.arch.flows.find(f => f.id === nodeId);
            if (flow) {
                title = flow.title || flow.label || flow.id;
                desc = flow.description || '';
                if (flow.steps && flow.steps.length > 0) {
                    desc += '\n\nSteps:\n';
                    flow.steps.forEach((step, idx) => {
                        const stepDesc = step.description || '';
                        desc += `${idx + 1}. ${step.id || 'Step ' + (idx + 1)}${stepDesc ? ': ' + stepDesc : ''}\n`;
                    });
                }
            }
        }
        if (this.titleEl)
            this.titleEl.textContent = title;
        if (this.descEl)
            this.descEl.textContent = desc;
        this.classList.add('visible');
    }
    hide() {
        this.classList.remove('visible');
    }
}
customElements.define('sruja-info-panel', SrujaInfoPanel);


// === sruja-viewer.js ===
"use strict";
// packages/html-viewer/src/sruja-viewer.ts
// Main viewer component that orchestrates all functionality
class SrujaViewer extends HTMLElement {
    constructor() {
        super(...arguments);
        Object.defineProperty(this, "nodeIndex", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
        Object.defineProperty(this, "infoPanel", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
        Object.defineProperty(this, "currentFilter", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'all'
        });
        Object.defineProperty(this, "searchQuery", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: ''
        });
        Object.defineProperty(this, "selectedNodeId", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: null
        });
    }
    connectedCallback() {
        this.nodeIndex = document.querySelector('sruja-node-index');
        this.infoPanel = document.querySelector('sruja-info-panel');
        if (!this.nodeIndex) {
            const indexEl = document.querySelector('sruja-node-index');
            if (indexEl) {
                indexEl.addEventListener('index-ready', () => this.initialize());
            }
            return;
        }
        this.initialize();
    }
    initialize() {
        this.setupSearch();
        this.setupFilters();
        this.setupViewTabs();
        this.setupKeyboardShortcuts();
        // Attach interactions will be called from updateVisibility
        this.updateVisibility();
    }
    setupSearch() {
        const searchBox = document.getElementById('search-box');
        if (!searchBox)
            return;
        searchBox.addEventListener('input', (e) => {
            this.searchQuery = e.target.value;
            this.updateVisibility();
        });
    }
    setupFilters() {
        const indexEl = this.nodeIndex;
        if (!indexEl)
            return;
        const arch = indexEl.getArchitecture();
        if (!arch)
            return;
        const hasElements = {
            system: (arch.systems || []).length > 0,
            container: (arch.systems || []).some(s => (s.containers || []).length > 0),
            component: (arch.systems || []).some(s => (s.containers || []).some(c => (c.components || []).length > 0)),
            person: (arch.persons || []).length > 0,
            datastore: (arch.systems || []).some(s => (s.datastores || []).length > 0),
            queue: (arch.systems || []).some(s => (s.queues || []).length > 0),
            requirement: (arch.requirements || []).length > 0,
            adr: (arch.adrs || []).length > 0,
            scenario: (arch.scenarios || []).length > 0,
            flow: (arch.flows || []).length > 0,
            deployment: (arch.deployment || []).length > 0
        };
        Object.keys(hasElements).forEach(filterType => {
            if (!hasElements[filterType]) {
                const tab = document.querySelector(`.filter-tab[data-filter="${filterType}"]`);
                if (tab) {
                    tab.disabled = true;
                    tab.classList.add('disabled');
                    tab.title = `No ${filterType} elements in this architecture`;
                }
            }
        });
        document.querySelectorAll('.filter-tab').forEach(tab => {
            tab.addEventListener('click', (e) => {
                e.preventDefault();
                e.stopPropagation();
                const clickedTab = (e.currentTarget || e.target.closest('.filter-tab') || e.target);
                if (!clickedTab || !clickedTab.classList || clickedTab.disabled)
                    return;
                document.querySelectorAll('.filter-tab').forEach(t => t.classList.remove('active'));
                clickedTab.classList.add('active');
                const filter = clickedTab.getAttribute('data-filter');
                if (filter) {
                    this.currentFilter = filter;
                    this.updateVisibility();
                }
            });
        });
    }
    setupViewTabs() {
        document.querySelectorAll('.view-tab').forEach(tab => {
            tab.addEventListener('click', (e) => {
                e.preventDefault();
                e.stopPropagation();
                const clickedTab = (e.currentTarget || e.target.closest('.view-tab') || e.target);
                if (!clickedTab || !clickedTab.classList)
                    return;
                const view = clickedTab.getAttribute('data-view');
                if (!view)
                    return;
                document.querySelectorAll('.view-tab').forEach(t => t.classList.remove('active'));
                clickedTab.classList.add('active');
                document.querySelectorAll('.view-content').forEach(v => v.classList.remove('active'));
                const viewEl = document.getElementById(view + '-view');
                if (viewEl) {
                    viewEl.classList.add('active');
                }
                // Populate requirements/ADRs/scenarios/flows view when switching to it
                if (view === 'requirements') {
                    this.populateRequirementsView();
                }
            });
        });
    }
    populateRequirementsView() {
        if (!this.nodeIndex)
            return;
        const arch = this.nodeIndex.getArchitecture();
        if (!arch)
            return;
        const requirementsList = document.getElementById('requirements-list');
        const adrsList = document.getElementById('adrs-list');
        if (requirementsList) {
            requirementsList.innerHTML = '';
            (arch.requirements || []).forEach((req) => {
                const item = document.createElement('div');
                item.className = 'requirement-item';
                item.setAttribute('data-requirement-id', req.id);
                const header = document.createElement('div');
                header.className = 'requirement-header';
                const id = document.createElement('span');
                id.className = 'requirement-id';
                id.textContent = req.id;
                const type = document.createElement('span');
                type.className = 'requirement-type';
                type.textContent = req.type || 'requirement';
                header.appendChild(id);
                header.appendChild(type);
                const desc = document.createElement('div');
                desc.className = 'requirement-description';
                desc.textContent = req.title || req.description || '';
                item.appendChild(header);
                item.appendChild(desc);
                item.addEventListener('click', () => {
                    document.querySelectorAll('.requirement-item').forEach(i => i.classList.remove('selected'));
                    item.classList.add('selected');
                    if (this.infoPanel) {
                        this.infoPanel.showNode(req.id);
                    }
                });
                requirementsList.appendChild(item);
            });
        }
        if (adrsList) {
            adrsList.innerHTML = '';
            (arch.adrs || []).forEach((adr) => {
                const item = document.createElement('div');
                item.className = 'adr-item';
                item.setAttribute('data-adr-id', adr.id);
                const header = document.createElement('div');
                header.className = 'adr-header';
                const id = document.createElement('span');
                id.className = 'adr-id';
                id.textContent = adr.id;
                if (adr.status) {
                    const status = document.createElement('span');
                    status.className = 'adr-status';
                    status.textContent = adr.status;
                    header.appendChild(status);
                }
                header.insertBefore(id, header.firstChild);
                const decision = document.createElement('div');
                decision.className = 'adr-decision';
                decision.textContent = adr.decision || adr.context || adr.title || '';
                item.appendChild(header);
                item.appendChild(decision);
                if (adr.context && adr.decision) {
                    const context = document.createElement('div');
                    context.className = 'adr-context';
                    context.textContent = `Context: ${adr.context}`;
                    item.appendChild(context);
                }
                if (adr.consequences) {
                    const consequences = document.createElement('div');
                    consequences.className = 'adr-consequences';
                    consequences.textContent = `Consequences: ${adr.consequences}`;
                    item.appendChild(consequences);
                }
                item.addEventListener('click', () => {
                    document.querySelectorAll('.adr-item').forEach(i => i.classList.remove('selected'));
                    item.classList.add('selected');
                    if (this.infoPanel) {
                        this.infoPanel.showNode(adr.id);
                    }
                });
                adrsList.appendChild(item);
            });
        }
        const scenariosList = document.getElementById('scenarios-list');
        if (scenariosList) {
            scenariosList.innerHTML = '';
            (arch.scenarios || []).forEach((scenario) => {
                const item = document.createElement('div');
                item.className = 'scenario-item';
                item.setAttribute('data-scenario-id', scenario.id);
                const title = document.createElement('div');
                title.className = 'scenario-title';
                title.textContent = scenario.title || scenario.label || scenario.id;
                const desc = document.createElement('div');
                desc.className = 'scenario-description';
                desc.textContent = scenario.description || '';
                item.appendChild(title);
                if (scenario.description) {
                    item.appendChild(desc);
                }
                if (scenario.steps && scenario.steps.length > 0) {
                    const stepsContainer = document.createElement('div');
                    stepsContainer.className = 'scenario-steps';
                    const stepsTitle = document.createElement('div');
                    stepsTitle.className = 'scenario-steps-title';
                    stepsTitle.textContent = 'Steps:';
                    stepsContainer.appendChild(stepsTitle);
                    scenario.steps.forEach((step, idx) => {
                        const stepItem = document.createElement('div');
                        stepItem.className = 'step-item';
                        const stepDesc = step.description || '';
                        stepItem.textContent = `${idx + 1}. ${step.from} → ${step.to}${stepDesc ? ': ' + stepDesc : ''}`;
                        stepsContainer.appendChild(stepItem);
                    });
                    item.appendChild(stepsContainer);
                }
                item.addEventListener('click', () => {
                    document.querySelectorAll('.scenario-item').forEach(i => i.classList.remove('selected'));
                    item.classList.add('selected');
                    if (this.infoPanel) {
                        this.infoPanel.showNode(scenario.id);
                    }
                });
                scenariosList.appendChild(item);
            });
        }
        const flowsList = document.getElementById('flows-list');
        if (flowsList) {
            flowsList.innerHTML = '';
            (arch.flows || []).forEach((flow) => {
                const item = document.createElement('div');
                item.className = 'flow-item';
                item.setAttribute('data-flow-id', flow.id);
                const title = document.createElement('div');
                title.className = 'flow-title';
                title.textContent = `${flow.id}: ${flow.title || flow.label || flow.id}`;
                const desc = document.createElement('div');
                desc.className = 'flow-description';
                desc.textContent = flow.description || '';
                item.appendChild(title);
                if (flow.description) {
                    item.appendChild(desc);
                }
                if (flow.steps && flow.steps.length > 0) {
                    const stepsContainer = document.createElement('div');
                    stepsContainer.className = 'flow-steps';
                    const stepsTitle = document.createElement('div');
                    stepsTitle.className = 'flow-steps-title';
                    stepsTitle.textContent = 'Steps:';
                    stepsContainer.appendChild(stepsTitle);
                    flow.steps.forEach((step, idx) => {
                        const stepItem = document.createElement('div');
                        stepItem.className = 'step-item';
                        const stepDesc = step.description || '';
                        const stepId = step.id || `Step ${idx + 1}`;
                        stepItem.textContent = `${stepId}${stepDesc ? ': ' + stepDesc : ''}`;
                        stepsContainer.appendChild(stepItem);
                    });
                    item.appendChild(stepsContainer);
                }
                item.addEventListener('click', () => {
                    document.querySelectorAll('.flow-item').forEach(i => i.classList.remove('selected'));
                    item.classList.add('selected');
                    if (this.infoPanel) {
                        this.infoPanel.showNode(flow.id);
                    }
                });
                flowsList.appendChild(item);
            });
        }
    }
    setupKeyboardShortcuts() {
        const searchBox = document.getElementById('search-box');
        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
                e.preventDefault();
                if (searchBox)
                    searchBox.focus();
            }
            else if (e.key === 'Escape') {
                this.selectedNodeId = null;
                this.highlightNode(null);
                if (searchBox)
                    searchBox.value = '';
                this.searchQuery = '';
                this.updateVisibility();
            }
        });
    }
    matchesFilter(nodeId) {
        if (!nodeId)
            return false;
        if (this.currentFilter === 'all')
            return true;
        if (!this.nodeIndex)
            return false;
        const type = this.nodeIndex.getNodeType(nodeId);
        return type === this.currentFilter;
    }
    matchesSearch(nodeId) {
        if (!nodeId || !this.nodeIndex)
            return false;
        return this.nodeIndex.matchesSearch(nodeId, this.searchQuery);
    }
    updateVisibility() {
        // Hide all SVG containers and grid
        document.querySelectorAll('.svg-container-level').forEach(el => {
            el.classList.remove('active');
            el.style.display = 'none';
        });
        const gridContainer = document.getElementById('svg-grid-container');
        if (gridContainer) {
            gridContainer.style.display = 'none';
        }
        const svgContainerAll = document.getElementById('svg-container-all');
        const svg = svgContainerAll ? svgContainerAll.querySelector('svg') : null;
        if (this.currentFilter === 'all') {
            if (svgContainerAll) {
                svgContainerAll.classList.add('active');
                svgContainerAll.style.display = 'flex';
            }
            if (svg && svgContainerAll && this.nodeIndex) {
                this.nodeIndex.mapSvgNodesToIds(svg);
                this.attachNodeInteractions(svg);
                this.updateNodeVisibility(svg);
            }
        }
        else if (['system', 'container'].includes(this.currentFilter)) {
            this.showGrid(this.currentFilter);
        }
        else if (['scenario', 'flow'].includes(this.currentFilter)) {
            // Scenarios and flows have their own separate SVG diagrams
            // Show them in grid view
            this.showGrid(this.currentFilter);
        }
        else {
            if (svgContainerAll) {
                svgContainerAll.classList.add('active');
                svgContainerAll.style.display = 'flex';
            }
            if (svg && svgContainerAll && this.nodeIndex) {
                this.nodeIndex.mapSvgNodesToIds(svg);
                this.attachNodeInteractions(svg);
                this.updateNodeVisibility(svg, true);
            }
        }
    }
    showGrid(filterType) {
        if (!this.nodeIndex)
            return;
        const prefix = `svg-container-${filterType}-`;
        const containers = document.querySelectorAll(`[id^="${prefix}"]`);
        if (containers.length === 0)
            return;
        let gridContainer = document.getElementById('svg-grid-container');
        if (!gridContainer) {
            gridContainer = document.createElement('div');
            gridContainer.id = 'svg-grid-container';
            gridContainer.className = 'svg-grid';
            const svgContainerAll = document.getElementById('svg-container-all');
            if (svgContainerAll && svgContainerAll.parentNode) {
                svgContainerAll.parentNode.insertBefore(gridContainer, svgContainerAll);
            }
        }
        gridContainer.innerHTML = '';
        gridContainer.style.display = 'grid';
        const keyAttr = filterType === 'container' ? 'data-container-key' : `data-${filterType}-id`;
        containers.forEach(cont => {
            const id = cont.getAttribute(keyAttr);
            if (!id)
                return;
            const nodeId = filterType === 'container' ? id.split('.')[1] : id;
            if (!this.matchesSearch(nodeId))
                return;
            const svg = cont.querySelector('svg');
            if (!svg)
                return;
            const node = this.nodeIndex.getNode(nodeId);
            const gridItem = document.createElement('div');
            gridItem.className = 'svg-grid-item';
            const title = document.createElement('h4');
            title.textContent = node ? (node.name || nodeId) : nodeId;
            const wrapper = document.createElement('div');
            wrapper.className = 'svg-wrapper';
            const svgClone = svg.cloneNode(true);
            wrapper.appendChild(svgClone);
            gridItem.appendChild(title);
            gridItem.appendChild(wrapper);
            gridContainer.appendChild(gridItem);
            this.attachNodeInteractions(svgClone);
        });
    }
    updateNodeVisibility(svg, useFilter = false) {
        if (!this.nodeIndex)
            return;
        const nodes = svg.querySelectorAll('[data-node-id]');
        nodes.forEach(nodeEl => {
            const nodeId = nodeEl.getAttribute('data-node-id');
            const visible = useFilter
                ? this.matchesFilter(nodeId) && this.matchesSearch(nodeId)
                : this.matchesSearch(nodeId);
            if (visible) {
                nodeEl.classList.remove('node-filtered-out');
            }
            else {
                nodeEl.classList.add('node-filtered-out');
            }
        });
        const svgEdges = svg.querySelectorAll('line, path[marker-end]');
        svgEdges.forEach(edge => {
            const fromId = edge.getAttribute('data-from');
            const toId = edge.getAttribute('data-to');
            if (fromId && toId) {
                const fromVisible = useFilter
                    ? this.matchesFilter(fromId) && this.matchesSearch(fromId)
                    : this.matchesSearch(fromId);
                const toVisible = useFilter
                    ? this.matchesFilter(toId) && this.matchesSearch(toId)
                    : this.matchesSearch(toId);
                if (fromVisible && toVisible) {
                    edge.classList.remove('edge-dimmed');
                }
                else {
                    edge.classList.add('edge-dimmed');
                }
            }
        });
    }
    attachNodeInteractions(targetSvg) {
        if (!targetSvg || !this.nodeIndex)
            return;
        this.nodeIndex.mapSvgNodesToIds(targetSvg);
        const nodes = targetSvg.querySelectorAll('[data-node-id]');
        nodes.forEach(nodeEl => {
            const nodeId = nodeEl.getAttribute('data-node-id');
            const node = this.nodeIndex.getNode(nodeId || '');
            if (!node)
                return;
            // Remove existing listeners by cloning
            const newNodeEl = nodeEl.cloneNode(true);
            if (nodeEl.parentNode) {
                nodeEl.parentNode.replaceChild(newNodeEl, nodeEl);
            }
            newNodeEl.addEventListener('click', (e) => {
                e.stopPropagation();
                this.selectedNodeId = this.selectedNodeId === nodeId ? null : nodeId;
                this.highlightNode(this.selectedNodeId);
                if (this.selectedNodeId && this.infoPanel) {
                    this.infoPanel.showNode(this.selectedNodeId);
                }
                else if (this.infoPanel) {
                    this.infoPanel.hide();
                }
            });
            newNodeEl.addEventListener('mouseenter', () => {
                if (this.selectedNodeId !== nodeId && this.infoPanel) {
                    this.infoPanel.showNode(nodeId || '');
                    newNodeEl.classList.add('node-highlight');
                }
            });
            newNodeEl.addEventListener('mouseleave', () => {
                if (this.selectedNodeId !== nodeId && this.infoPanel) {
                    this.infoPanel.hide();
                    newNodeEl.classList.remove('node-highlight');
                }
            });
        });
    }
    highlightNode(nodeId) {
        const activeContainer = document.querySelector('.svg-container-level.active') ||
            document.getElementById('svg-container-all');
        if (!activeContainer)
            return;
        const activeSvg = activeContainer.querySelector('svg');
        if (!activeSvg || !this.nodeIndex)
            return;
        activeSvg.querySelectorAll('.node-selected').forEach(el => el.classList.remove('node-selected'));
        activeSvg.querySelectorAll('.edge-highlight').forEach(el => el.classList.remove('edge-highlight'));
        if (!nodeId)
            return;
        const nodeEl = activeSvg.querySelector(`[data-node-id="${nodeId}"]`);
        if (nodeEl) {
            nodeEl.classList.add('node-selected');
            const edges = this.nodeIndex.getEdges();
            edges.filter(e => e.from === nodeId || e.to === nodeId).forEach(edge => {
                const edgeEl = activeSvg.querySelector(`[data-from="${edge.from}"][data-to="${edge.to}"]`);
                if (edgeEl)
                    edgeEl.classList.add('edge-highlight');
            });
            // Scroll container to bring node into view
            const container = document.getElementById('container');
            if (nodeEl && container) {
                const nodeRect = nodeEl.getBoundingClientRect();
                const containerRect = container.getBoundingClientRect();
                const containerCenterY = containerRect.top + containerRect.height / 2;
                const containerCenterX = containerRect.left + containerRect.width / 2;
                const nodeCenterY = nodeRect.top + nodeRect.height / 2;
                const nodeCenterX = nodeRect.left + nodeRect.width / 2;
                // Calculate scroll needed to center the node
                const scrollY = container.scrollTop + (nodeCenterY - containerCenterY);
                const scrollX = container.scrollLeft + (nodeCenterX - containerCenterX);
                container.scrollTo({
                    left: scrollX,
                    top: scrollY,
                    behavior: 'smooth'
                });
            }
        }
    }
}
customElements.define('sruja-viewer', SrujaViewer);


