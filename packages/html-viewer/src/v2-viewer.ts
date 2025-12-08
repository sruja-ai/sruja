import { V2Layout } from './v2-layout';
import { V2Renderer } from './v2-renderer';
import { V2Interaction } from './v2-interaction';
import { SrujaNodeIndex } from './sruja-node-index';

export class SrujaV2Viewer extends HTMLElement {
    private layoutEngine: V2Layout;
    private renderer: V2Renderer;
    // @ts-expect-error - V2Interaction is initialized in constructor
    private interaction: V2Interaction;
    private nodeIndex: SrujaNodeIndex;

    private expandedNodes: Set<string> = new Set();

    constructor() {
        super();
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
        this.shadowRoot!.appendChild(container);

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
        this.shadowRoot!.appendChild(this.nodeIndex);

        
    }

    connectedCallback() {
        
    }

    async renderGraph() {
        const arch = this.nodeIndex.getArchitecture();
        if (!arch) return;

        // Transform simplified architecture to ELK graph
        const nodes: any[] = [];
        const edges: any[] = [];

        // Add Systems
        if (arch.systems) {
            arch.systems.forEach(sys => {
                const isSystemExpanded = this.expandedNodes.has(sys.id);
                const hasContainers = sys.containers && sys.containers.length > 0;

                const sysNode: any = {
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

                            const contNode: any = {
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
                                cont.components.forEach((comp: any) => {
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
        } catch (e) {
            console.error('Layout failed:', e);
            const container = this.shadowRoot!.getElementById('sruja-v2-container');
            if (container) {
                container.innerText = `Error rendering graph: ${e}`;
                container.style.color = 'red';
                container.style.display = 'flex';
                container.style.justifyContent = 'center';
                container.style.alignItems = 'center';
            }
        }
    }

    handleNodeClick(nodeId: string) {
        
        if (this.expandedNodes.has(nodeId)) {
            this.expandedNodes.delete(nodeId);
        } else {
            this.expandedNodes.add(nodeId);
        }
        this.renderGraph();
    }
}

customElements.define('sruja-v2-viewer', SrujaV2Viewer);
