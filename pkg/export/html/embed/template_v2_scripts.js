const SrujaViewer = {
    cy: null,
    container: null,

    init(selector) {
        this.container = document.querySelector(selector);
    },

    render(elements, layoutOptions = {}) {
        if (!this.container) return;

        const defaultLayout = {
            name: 'dagre',
            rankDir: 'TB',
            nodeSep: 50,
            rankSep: 100,
            padding: 50,
            animate: true,
            animationDuration: 500,
            fit: true
        };

        const config = {
            container: this.container,
            elements: elements,
            style: [
                // Base node style
                {
                    selector: 'node',
                    style: {
                        'label': 'data(label)',
                        'text-valign': 'center',
                        'text-halign': 'center',
                        'text-wrap': 'wrap',
                        'text-max-width': '120px',
                        'font-family': 'Inter, system-ui, -apple-system, sans-serif',
                        'font-size': '12px',
                        'font-weight': 500,
                        'border-width': 2,
                        'transition-property': 'background-color, border-color, width, height',
                        'transition-duration': '300ms'
                    }
                },
                // Person node (user/actor)
                {
                    selector: 'node[type="person"]',
                    style: {
                        'shape': 'ellipse',
                        'background-color': '#6366f1',
                        'border-color': '#4f46e5',
                        'color': '#ffffff',
                        'width': 110,
                        'height': 75,
                        'font-weight': 600,
                        'text-margin-y': 0
                    }
                },
                // System node
                {
                    selector: 'node[type="system"]',
                    style: {
                        'shape': 'round-rectangle',
                        'background-color': '#1e293b',
                        'border-color': '#334155',
                        'color': '#ffffff',
                        'width': 180,
                        'height': 90,
                        'font-size': '14px',
                        'font-weight': 700,
                        'text-transform': 'uppercase'
                    }
                },
                // External system
                {
                    selector: 'node[type="external"]',
                    style: {
                        'shape': 'round-rectangle',
                        'background-color': '#64748b',
                        'border-color': '#475569',
                        'border-style': 'dashed',
                        'border-width': 2,
                        'color': '#ffffff',
                        'width': 160,
                        'height': 80
                    }
                },
                // Container node
                {
                    selector: 'node[type="container"]',
                    style: {
                        'shape': 'round-rectangle',
                        'background-color': '#3b82f6',
                        'border-color': '#2563eb',
                        'color': '#ffffff',
                        'width': 150,
                        'height': 70
                    }
                },
                // Component node
                {
                    selector: 'node[type="component"]',
                    style: {
                        'shape': 'round-rectangle',
                        'background-color': '#93c5fd',
                        'border-color': '#60a5fa',
                        'color': '#1e293b',
                        'width': 130,
                        'height': 55
                    }
                },
                // Datastore node (database)
                {
                    selector: 'node[type="datastore"]',
                    style: {
                        'shape': 'barrel',
                        'background-color': '#fbbf24',
                        'border-color': '#f59e0b',
                        'color': '#1e293b',
                        'width': 110,
                        'height': 70
                    }
                },
                // Queue node
                {
                    selector: 'node[type="queue"]',
                    style: {
                        'shape': 'round-rectangle',
                        'background-color': '#10b981',
                        'border-color': '#059669',
                        'color': '#ffffff',
                        'width': 110,
                        'height': 55
                    }
                },
                // Parent/compound node style
                {
                    selector: ':parent',
                    style: {
                        'background-color': '#f8fafc',
                        'background-opacity': 0.9,
                        'border-width': 2,
                        'border-style': 'dashed',
                        'border-color': '#cbd5e1',
                        'text-valign': 'top',
                        'text-halign': 'center',
                        'text-margin-y': -8,
                        'color': '#64748b',
                        'font-size': '13px',
                        'font-weight': 600,
                        'padding': 35,
                        'text-transform': 'uppercase'
                    }
                },
                // Edge style
                {
                    selector: 'edge',
                    style: {
                        'curve-style': 'bezier',
                        'target-arrow-shape': 'triangle',
                        'target-arrow-color': '#64748b',
                        'line-color': '#94a3b8',
                        'width': 2,
                        'label': 'data(label)',
                        'font-size': '10px',
                        'color': '#475569',
                        'text-background-color': '#ffffff',
                        'text-background-opacity': 0.95,
                        'text-background-padding': '3px',
                        'text-background-shape': 'round-rectangle',
                        'text-rotation': 'autorotate',
                        'arrow-scale': 1.3,
                        'opacity': 0.9
                    }
                },
                // Selected node
                {
                    selector: 'node:selected',
                    style: {
                        'border-width': 4,
                        'border-color': '#3b82f6',
                        'border-opacity': 1
                    }
                },
                // Selected edge
                {
                    selector: 'edge:selected',
                    style: {
                        'line-color': '#3b82f6',
                        'target-arrow-color': '#3b82f6',
                        'width': 3,
                        'opacity': 1
                    }
                }
            ],
            layout: { ...defaultLayout, ...layoutOptions },
            minZoom: 0.2,
            maxZoom: 3
        };

        if (this.cy) {
            this.cy.destroy();
        }
        this.cy = cytoscape(config);

        // Add double-click handler for drill-down
        this.cy.on('dblclick', 'node', (evt) => {
            const node = evt.target;
            const type = node.data('type');
            if (type === 'system') {
                showContainerView(node.id());
            } else if (type === 'container') {
                showComponentView(node.id(), node.data('parentSystem'));
            }
        });
    },

    fit() { this.cy && this.cy.fit(void 0, 50) },
    reset() { this.cy && (this.cy.fit(void 0, 50), this.cy.center()) }
};

// Global state
let archData = null;
let currentView = { level: 1, focusId: null };
let navigationTree = { systems: [], containers: {} };
let nodeParentMap = {};

// Initialize viewer when DOM is ready
document.addEventListener('DOMContentLoaded', async () => {
    const dataEl = document.getElementById('sruja-data');
    if (!dataEl) {
        console.error('No sruja-data element found');
        return;
    }

    try {
        archData = JSON.parse(dataEl.textContent);
    } catch (e) {
        console.error('Failed to parse architecture data:', e);
        return;
    }

    // Build navigation tree from architecture data
    buildNavigationTree();

    // Populate sidebar
    populateSidebar();

    // Populate documentation panel
    populateDocumentation();

    // Initialize SrujaViewer
    if (typeof SrujaViewer !== 'undefined') {
        const loadingOverlay = document.getElementById('loading-overlay');

        // Show initial view with a slight delay to allow rendering
        setTimeout(() => {
            SrujaViewer.init('#cy');
            showSystemContext();

            // Hide loading overlay
            if (loadingOverlay) {
                loadingOverlay.classList.add('hidden');
            }
        }, 100);
    } else {
        console.error('SrujaViewer not found');
        document.getElementById('cy').innerHTML = '<p style="padding:20px;color:red;">Viewer failed to load</p>';
    }
});

// Build navigation tree from architecture JSON
function buildNavigationTree() {
    const arch = archData.architecture || {};
    navigationTree = { systems: [], containers: {} };
    nodeParentMap = {};

    // Extract systems
    if (arch.systems) {
        arch.systems.forEach(sys => {
            navigationTree.systems.push({
                id: sys.id,
                label: sys.label || sys.id,
                type: 'system'
            });

            // Extract containers for this system
            if (sys.containers) {
                navigationTree.containers[sys.id] = sys.containers.map(cont => ({
                    id: cont.id,
                    label: cont.label || cont.id,
                    type: 'container',
                    parentSystem: sys.id
                }));

                // Populate parent map: Container -> System
                sys.containers.forEach(cont => {
                    nodeParentMap[cont.id] = sys.id;

                    // Populate parent map: Component -> Container
                    (cont.components || []).forEach(comp => {
                        nodeParentMap[comp.id] = cont.id;
                    });
                });
            }
        });
    }
}

// Populate sidebar with navigation items
function populateSidebar() {
    const systemsList = document.getElementById('systems-list');
    systemsList.innerHTML = '';

    navigationTree.systems.forEach(sys => {
        const item = document.createElement('div');
        item.className = 'nav-item';
        item.id = 'nav-' + sys.id;
        item.onclick = () => showContainerView(sys.id);
        item.innerHTML = `
                    <span class="icon">📦</span>
                    <span>${sys.label}</span>
                `;
        systemsList.appendChild(item);
    });
}

// Update containers list for a specific system
function updateContainersList(systemId) {
    const containersSection = document.getElementById('containers-section');
    const containersList = document.getElementById('containers-list');

    const containers = navigationTree.containers[systemId] || [];

    if (containers.length === 0) {
        containersSection.style.display = 'none';
        return;
    }

    containersSection.style.display = 'block';
    containersList.innerHTML = '';

    containers.forEach(cont => {
        const item = document.createElement('div');
        item.className = 'nav-item nav-item-child';
        item.id = 'nav-' + cont.id;
        item.onclick = () => showComponentView(cont.id, systemId);
        item.innerHTML = `
                    <span class="icon">🔧</span>
                    <span>${cont.label}</span>
                `;
        containersList.appendChild(item);
    });
}

// Helper: Resolve a node ID to its visible ancestor in the current node set
// This matches viewer-core's resolveNodeId logic
function resolveNodeId(id, addedNodes, arch) {
    // If the ID is directly in our added nodes, return it
    if (addedNodes.has(id)) return id;

    // Check if it's a container/component that belongs to a system
    if (arch.systems) {
        for (const system of arch.systems) {
            // Check if ID is a container in this system
            if (system.containers) {
                for (const container of system.containers) {
                    if (container.id === id) {
                        // If the system is in added nodes, resolve to system
                        if (addedNodes.has(system.id)) return system.id;
                        // If qualified container ID is in added nodes, use that
                        const qualifiedId = `${system.id}.${id}`;
                        if (addedNodes.has(qualifiedId)) return qualifiedId;
                    }
                    // Check if ID is a component in this container
                    if (container.components) {
                        for (const component of container.components) {
                            if (component.id === id) {
                                // Resolve to container or system
                                const qualifiedContainerId = `${system.id}.${container.id}`;
                                if (addedNodes.has(qualifiedContainerId)) return qualifiedContainerId;
                                if (addedNodes.has(system.id)) return system.id;
                            }
                        }
                    }
                }
            }
            // Check datastores and queues
            if (system.datastores) {
                for (const ds of system.datastores) {
                    if (ds.id === id) {
                        const qualifiedId = `${system.id}.${id}`;
                        if (addedNodes.has(qualifiedId)) return qualifiedId;
                        if (addedNodes.has(system.id)) return system.id;
                    }
                }
            }
            if (system.queues) {
                for (const q of system.queues) {
                    if (q.id === id) {
                        const qualifiedId = `${system.id}.${id}`;
                        if (addedNodes.has(qualifiedId)) return qualifiedId;
                        if (addedNodes.has(system.id)) return system.id;
                    }
                }
            }
        }
    }

    return null;
}

// Helper: Add node/edge to elements array
const elementsBuilder = (arch) => {
    const e = [];
    const addedNodes = new Set();
    const addedEdges = new Set();

    return {
        addNode: (id, label, type, parent, data = {}) => {
            if (addedNodes.has(id)) return;
            e.push({ data: { id, label: label || id, type, parent, ...data } });
            addedNodes.add(id);
        },
        addEdge: (src, tgt, lbl) => {
            // Resolve source and target to visible nodes
            const resolvedSrc = resolveNodeId(src, addedNodes, arch);
            const resolvedTgt = resolveNodeId(tgt, addedNodes, arch);

            // Only add edge if both ends resolve and are different
            if (resolvedSrc && resolvedTgt && resolvedSrc !== resolvedTgt) {
                const edgeId = `${resolvedSrc}-${resolvedTgt}`;
                if (!addedEdges.has(edgeId)) {
                    e.push({ data: { source: resolvedSrc, target: resolvedTgt, label: lbl || '' } });
                    addedEdges.add(edgeId);
                }
            }
        },
        getElements: () => e,
        getAddedNodes: () => addedNodes
    };
};

// Show Full Architecture View (level 0)
// Renders everything (systems, persons) but NO internals to avoid clutter in breadthfirst
function showFullArchitecture() {
    currentView = { level: 0, focusId: null };
    const arch = archData.architecture || {};
    const builder = elementsBuilder(arch);

    // Update UI
    document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
    document.getElementById('nav-full').classList.add('active');
    document.getElementById('containers-section').style.display = 'none';
    updateBreadcrumb([{ label: '🗺️ Full Architecture', level: 0 }]);

    // Add Persons
    (arch.persons || []).forEach(p => {
        builder.addNode(p.id, p.label || p.name, 'person');
    });

    // Add Systems
    (arch.systems || []).forEach(sys => {
        builder.addNode(sys.id, sys.label || sys.name, sys.external ? 'external' : 'system');
    });

    // Add Edges from top-level relations array
    (arch.relations || []).forEach(r => {
        builder.addEdge(r.from, r.to, r.verb || r.label || '');
    });

    SrujaViewer.render(builder.getElements());
}

// Show L1: System Context View
// Renders Systems + Persons + Relationships
function showSystemContext() {
    currentView = { level: 1, focusId: null };
    const arch = archData.architecture || {};
    const builder = elementsBuilder(arch);

    // Update UI
    document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
    document.getElementById('nav-context').classList.add('active');
    document.getElementById('containers-section').style.display = 'none';
    updateBreadcrumb([{ label: '🏛️ System Context', level: 1 }]);

    // Add Persons
    (arch.persons || []).forEach(p => {
        builder.addNode(p.id, p.label || p.name, 'person');
    });

    // Add Systems
    (arch.systems || []).forEach(sys => {
        builder.addNode(sys.id, sys.label || sys.name, sys.external ? 'external' : 'system');
    });

    // Add Edges from top-level relations array
    // resolveNodeId will map container IDs to their parent system
    (arch.relations || []).forEach(r => {
        builder.addEdge(r.from, r.to, r.verb || r.label || '');
    });

    SrujaViewer.render(builder.getElements());
}

// Show L2: Container View for a specific system
// Renders: Focused System (expanded with containers) + Connected External Systems/Persons
function showContainerView(systemId) {
    currentView = { level: 2, focusId: systemId };
    const arch = archData.architecture || {};
    const builder = elementsBuilder(arch);

    const system = (arch.systems || []).find(s => s.id === systemId);
    if (!system) return;

    // Update UI
    document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
    const navItem = document.getElementById('nav-' + systemId);
    if (navItem) navItem.classList.add('active');
    updateContainersList(systemId);
    updateBreadcrumb([
        { label: '🏛️ System Context', level: 1, action: 'showSystemContext()' },
        { label: '📦 ' + (system.label || system.name), level: 2 }
    ]);

    // 1. Add Focused System (node type 'system') - serving as parent
    builder.addNode(system.id, system.label || system.name, 'system');

    // 2. Add Containers inside the system (with qualified IDs)
    const containerIds = new Set();
    (system.containers || []).forEach(cont => {
        const qualifiedId = `${system.id}.${cont.id}`;
        containerIds.add(cont.id);
        containerIds.add(qualifiedId);
        builder.addNode(qualifiedId, cont.label || cont.name, 'container', system.id, { parentSystem: system.id });
    });

    // 3. Add datastores/queues in this system
    (system.datastores || []).forEach(ds => {
        const qualifiedId = `${system.id}.${ds.id}`;
        builder.addNode(qualifiedId, ds.label || ds.name, 'datastore', system.id);
    });
    (system.queues || []).forEach(q => {
        const qualifiedId = `${system.id}.${q.id}`;
        builder.addNode(qualifiedId, q.label || q.name, 'queue', system.id);
    });

    // 4. Add edges from top-level relations that involve this system's elements
    (arch.relations || []).forEach(r => {
        const fromId = r.from;
        const toId = r.to;

        // Check if this edge involves any container in this system
        const fromInSystem = containerIds.has(fromId) || fromId === systemId;
        const toInSystem = containerIds.has(toId) || toId === systemId;

        // Show edges where at least one end is in this system
        if (fromInSystem || toInSystem) {
            // Add external nodes if needed
            if (!fromInSystem) {
                // Find the external node
                const person = (arch.persons || []).find(p => p.id === fromId);
                const otherSys = (arch.systems || []).find(s => s.id === fromId);
                if (person) {
                    builder.addNode(fromId, person.label || person.name, 'person');
                } else if (otherSys) {
                    builder.addNode(fromId, otherSys.label || otherSys.name, otherSys.external ? 'external' : 'system');
                }
            }
            if (!toInSystem) {
                const person = (arch.persons || []).find(p => p.id === toId);
                const otherSys = (arch.systems || []).find(s => s.id === toId);
                if (person) {
                    builder.addNode(toId, person.label || person.name, 'person');
                } else if (otherSys) {
                    builder.addNode(toId, otherSys.label || otherSys.name, otherSys.external ? 'external' : 'system');
                }
            }

            // Qualify IDs if they're containers in this system
            const qualifiedFrom = containerIds.has(fromId) ? `${system.id}.${fromId}` : fromId;
            const qualifiedTo = containerIds.has(toId) ? `${system.id}.${toId}` : toId;
            builder.addEdge(qualifiedFrom, qualifiedTo, r.verb || r.label || '');
        }
    });

    SrujaViewer.render(builder.getElements(), {
        name: 'elk',
        elk: {
            'algorithm': 'layered',
            'elk.direction': 'DOWN',
            'elk.spacing.nodeNode': '60',
        }
    });
}

// Show L3: Component View for a specific container
// Renders: Focused Container (expanded with components) + Connected elements
function showComponentView(containerId, systemId) {
    currentView = { level: 3, focusId: containerId, parentId: systemId };
    const arch = archData.architecture || {};
    const builder = elementsBuilder(arch);

    const system = (arch.systems || []).find(s => s.id === systemId);
    if (!system) return;

    const container = (system.containers || []).find(c => c.id === containerId);
    if (!container) return;

    // Update UI
    document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
    const navItem = document.getElementById('nav-' + containerId);
    if (navItem) navItem.classList.add('active');
    updateBreadcrumb([
        { label: '🏛️ System Context', level: 1, action: 'showSystemContext()' },
        { label: '📦 ' + (system.label || system.name), level: 2, action: `showContainerView('${systemId}')` },
        { label: '🔧 ' + (container.label || container.name), level: 3 }
    ]);

    // 1. Add Focused Container
    builder.addNode(container.id, container.label || container.name, 'container');

    // 2. Add Components inside container
    (container.components || []).forEach(comp => {
        builder.addNode(comp.id, comp.label || comp.name, 'component', container.id);
    });

    // 3. Add Edges (Component -> Component, Component -> Helper)
    (container.components || []).forEach(comp => {
        (comp.relations || []).forEach(r => builder.addEdge(comp.id, r.target, r.label));
    });

    // 4. Show connected peer containers (in same system)
    (system.containers || []).forEach(peer => {
        if (peer.id === containerId) return;

        // Check if any component talks to this peer, or peer talks to container
        // Simple heuristic: If raw container relations exist
        (container.relations || []).forEach(r => {
            if (r.target === peer.id) {
                builder.addNode(peer.id, peer.label || peer.name, 'container');
                builder.addEdge(container.id, peer.id, r.label);
            }
        });

        (peer.relations || []).forEach(r => {
            if (r.target === container.id) {
                builder.addNode(peer.id, peer.label || peer.name, 'container');
                builder.addEdge(peer.id, container.id, r.label);
            }
        });
    });

    // Also internal datastores/queues
    (system.datastores || []).forEach(ds => builder.addNode(ds.id, ds.label || ds.name, 'datastore', null)); // show outside container
    (system.queues || []).forEach(q => builder.addNode(q.id, q.label || q.name, 'queue', null));

    SrujaViewer.render(builder.getElements(), {
        name: 'breadthfirst',
        spacingFactor: 1.5
    });
}

// Update breadcrumb
function updateBreadcrumb(items) {
    const breadcrumb = document.getElementById('breadcrumb');
    breadcrumb.innerHTML = items.map((item, i) => {
        const isLast = i === items.length - 1;
        const separator = isLast ? '' : '<span class="breadcrumb-separator">›</span>';

        if (isLast) {
            return `<span class="breadcrumb-item active">${item.label}</span>`;
        } else {
            return `<span class="breadcrumb-item" onclick="${item.action}">${item.label}</span>${separator}`;
        }
    }).join('');
}

function fit() {
    if (typeof SrujaViewer !== 'undefined' && SrujaViewer.fit) {
        SrujaViewer.fit();
    }
}

function reset() {
    if (typeof SrujaViewer !== 'undefined' && SrujaViewer.reset) {
        SrujaViewer.reset();
    }
    showSystemContext();
}

// ============================================
// Export Functions
// ============================================

function exportPNG() {
    if (!SrujaViewer.cy) {
        alert('Diagram not ready');
        return;
    }
    const png = SrujaViewer.cy.png({ output: 'blob', bg: 'white', full: true, scale: 2 });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(png);
    link.download = (archData?.metadata?.name || 'architecture') + '.png';
    link.click();
    URL.revokeObjectURL(link.href);
}

function exportSVG() {
    if (!SrujaViewer.cy) {
        alert('Diagram not ready');
        return;
    }
    const svg = SrujaViewer.cy.svg({ full: true, bg: 'white' });
    const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = (archData?.metadata?.name || 'architecture') + '.svg';
    link.click();
    URL.revokeObjectURL(link.href);
}

function exportJSON() {
    const jsonStr = JSON.stringify(archData, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json;charset=utf-8' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = (archData?.metadata?.name || 'architecture') + '.json';
    link.click();
    URL.revokeObjectURL(link.href);
}

// ============================================
// Theme Toggle
// ============================================

let isDarkTheme = false;

function toggleTheme() {
    isDarkTheme = !isDarkTheme;
    document.body.classList.toggle('dark-theme', isDarkTheme);
    const btn = document.getElementById('theme-toggle');
    if (btn) {
        btn.textContent = isDarkTheme ? '☀️ Light' : '🌙 Dark';
    }
    // Update Cytoscape background if available
    if (SrujaViewer.cy) {
        const container = SrujaViewer.cy.container();
        if (container) {
            container.style.background = isDarkTheme ? '#0f172a' : '#f8fafc';
        }
    }
}

// ============================================
// Documentation Panel Functions
// ============================================

// Toggle documentation panel open/closed
function toggleDocPanel() {
    const panel = document.getElementById('doc-panel');
    const toggle = document.getElementById('doc-toggle');
    const isOpen = panel.classList.toggle('open');
    toggle.classList.toggle('open', isOpen);
    toggle.textContent = isOpen ? '✕ Close' : '📋 Documentation';
}

// Switch between documentation tabs
function showDocTab(tabName) {
    // Update tab buttons
    document.querySelectorAll('.doc-tab').forEach(tab => {
        tab.classList.toggle('active', tab.dataset.tab === tabName);
    });
    // Update sections
    document.querySelectorAll('.doc-section').forEach(section => {
        section.classList.toggle('active', section.id === 'doc-' + tabName);
    });
}

// Toggle ADR card expand/collapse
function toggleADR(element) {
    element.closest('.adr-card').classList.toggle('expanded');
}

// Populate documentation panel from architecture data
function populateDocumentation() {
    const arch = archData.architecture || {};

    // Populate ADRs
    populateADRs(arch.adrs || []);

    // Populate Requirements
    populateRequirements(arch.requirements || []);

    // Populate Policies
    populatePolicies(arch.policies || []);

    // Populate Metadata
    populateMetadata(arch.metadata || {});
}

// Populate ADRs section
function populateADRs(adrs) {
    const container = document.getElementById('adrs-content');
    const countBadge = document.getElementById('adr-count');
    countBadge.textContent = adrs.length;

    if (adrs.length === 0) {
        container.innerHTML = `
            <div class="doc-empty">
                <div class="doc-empty-icon">📝</div>
                <p>No Architecture Decision Records defined</p>
            </div>
        `;
        return;
    }

    container.innerHTML = adrs.map(adr => {
        const statusClass = (adr.status || 'proposed').toLowerCase().replace(/\s+/g, '-');
        return `
            <div class="adr-card">
                <div class="adr-header" onclick="toggleADR(this)">
                    <span class="adr-id">${adr.id || 'ADR'}</span>
                    <span class="adr-title">${escapeHtml(adr.title || adr.label || 'Untitled')}</span>
                    <span class="adr-status ${statusClass}">${adr.status || 'Proposed'}</span>
                </div>
                <div class="adr-body">
                    ${adr.context ? `<div class="adr-section"><div class="adr-section-label">Context</div>${escapeHtml(adr.context)}</div>` : ''}
                    ${adr.decision ? `<div class="adr-section"><div class="adr-section-label">Decision</div>${escapeHtml(adr.decision)}</div>` : ''}
                    ${adr.consequences ? `<div class="adr-section"><div class="adr-section-label">Consequences</div>${escapeHtml(adr.consequences)}</div>` : ''}
                </div>
            </div>
        `;
    }).join('');
}

// Populate Requirements section (grouped by type)
function populateRequirements(requirements) {
    const container = document.getElementById('requirements-content');
    const countBadge = document.getElementById('req-count');
    countBadge.textContent = requirements.length;

    if (requirements.length === 0) {
        container.innerHTML = `
            <div class="doc-empty">
                <div class="doc-empty-icon">✅</div>
                <p>No requirements defined</p>
            </div>
        `;
        return;
    }

    // Group by type
    const grouped = {};
    const typeIcons = {
        functional: '⚙️',
        performance: '⚡',
        security: '🔐',
        reliability: '🛡️',
        constraint: '📐'
    };

    requirements.forEach(req => {
        const type = req.type || 'functional';
        if (!grouped[type]) grouped[type] = [];
        grouped[type].push(req);
    });

    container.innerHTML = Object.entries(grouped).map(([type, reqs]) => `
        <div class="req-group">
            <div class="req-group-title">
                ${typeIcons[type] || '📋'} ${type.charAt(0).toUpperCase() + type.slice(1)} (${reqs.length})
            </div>
            ${reqs.map(req => `
                <div class="req-item">
                    <span class="req-id">${req.id || ''}</span>
                    <span class="req-text">${escapeHtml(req.description || req.text || '')}</span>
                </div>
            `).join('')}
        </div>
    `).join('');
}

// Populate Policies section
function populatePolicies(policies) {
    const container = document.getElementById('policies-content');
    const countBadge = document.getElementById('policy-count');
    countBadge.textContent = policies.length;

    if (policies.length === 0) {
        container.innerHTML = `
            <div class="doc-empty">
                <div class="doc-empty-icon">🔒</div>
                <p>No policies defined</p>
            </div>
        `;
        return;
    }

    container.innerHTML = policies.map(policy => `
        <div class="policy-item">
            <span class="policy-category ${(policy.category || '').toLowerCase()}">${policy.category || 'Policy'}</span>
            <span class="policy-text">${escapeHtml(policy.description || policy.text || policy.id || '')}</span>
            ${policy.enforcement ? `<span class="policy-enforcement">Enforcement: ${policy.enforcement}</span>` : ''}
        </div>
    `).join('');
}

// Populate Metadata section
function populateMetadata(metadata) {
    const container = document.getElementById('metadata-content');

    // Convert object to entries, handling various formats
    let entries = [];
    if (Array.isArray(metadata)) {
        entries = metadata.map(m => [m.key, m.value]);
    } else if (typeof metadata === 'object') {
        entries = Object.entries(metadata);
    }

    if (entries.length === 0) {
        container.innerHTML = `
            <div class="doc-empty">
                <div class="doc-empty-icon">ℹ️</div>
                <p>No metadata defined</p>
            </div>
        `;
        return;
    }

    container.innerHTML = `
        <div class="metadata-grid">
            ${entries.map(([key, value]) => `
                <div class="metadata-item">
                    <div class="metadata-label">${escapeHtml(key)}</div>
                    <div class="metadata-value">${escapeHtml(Array.isArray(value) ? value.join(', ') : String(value))}</div>
                </div>
            `).join('')}
        </div>
    `;
}

// Helper: Escape HTML to prevent XSS
function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
