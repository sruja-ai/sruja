export class V2Interaction {
    private container: HTMLElement;
    private onNodeClick: (nodeId: string) => void;

    constructor(container: HTMLElement, onNodeClick: (nodeId: string) => void) {
        this.container = container;
        this.onNodeClick = onNodeClick;
        this.attachListeners();
    }

    private attachListeners() {
        this.container.addEventListener('click', (e) => {
            const target = e.target as HTMLElement;
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
