// packages/html-viewer/src/sruja-node-index.ts
// Node indexing and mapping service

interface NodeData {
  id: string;
  type: string;
  name: string;
  description: string;
}

interface Edge {
  from: string;
  to: string;
  label: string;
  type: string;
}

interface Architecture {
  persons?: Array<{ id: string; label?: string; name?: string; description?: string; relations?: Array<{ target: string; label?: string; type?: string }> }>;
  systems?: Array<{
    id: string;
    label?: string;
    name?: string;
    description?: string;
    containers?: Array<{
      id: string;
      label?: string;
      name?: string;
      description?: string;
      components?: Array<{ id: string; label?: string; name?: string; description?: string }>;
    }>;
    datastores?: Array<{ id: string; label?: string; name?: string; description?: string }>;
    queues?: Array<{ id: string; label?: string; name?: string; description?: string }>;
    relations?: Array<{ target: string; label?: string; type?: string }>;
  }>;
  requirements?: Array<{ id: string; title?: string; description?: string; type?: string }>;
  adrs?: Array<{ id: string; title?: string; decision?: string; context?: string; status?: string; consequences?: string }>;
  scenarios?: Array<{
    id: string;
    title?: string;
    label?: string;
    description?: string;
    steps?: Array<{ from: string; to: string; description?: string }>;
  }>;
  flows?: Array<{
    id: string;
    title?: string;
    label?: string;
    description?: string;
    steps?: Array<{ id?: string; description?: string }>;
  }>;
  deployment?: Array<{ id: string; label?: string }>;
}

export class SrujaNodeIndex extends HTMLElement {
  private nodeIndex: Map<string, NodeData> = new Map();
  private edges: Edge[] = [];
  private arch: Architecture | null = null;

  connectedCallback(): void {
    this.loadData();
  }

  private loadData(): void {
    const dataEl = document.getElementById('sruja-data');
    if (!dataEl) {
      console.error('SrujaNodeIndex: #sruja-data element not found!');
      return;
    }

    const jsonData = JSON.parse(dataEl.textContent || '{}');
    this.arch = jsonData.architecture || {};
    this.buildIndex();
  }

  private buildIndex(): void {
    this.nodeIndex.clear();
    this.edges = [];

    if (!this.arch) return;

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

  private indexNode(id: string, type: string, name: string, description: string): void {
    this.nodeIndex.set(id, { id, type, name, description });
  }

  private indexEdges(items: Array<{ id: string; relations?: Array<{ target: string; label?: string; type?: string }>; containers?: Array<{ id: string; relations?: Array<{ target: string; label?: string; type?: string }>; components?: Array<{ id: string; relations?: Array<{ target: string; label?: string; type?: string }> }> }>; components?: Array<{ id: string; relations?: Array<{ target: string; label?: string; type?: string }> }> }>, _parentType: string): void {
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
        this.indexEdges(item.containers as any, 'container');
      }
      if ('components' in item && item.components) {
        this.indexEdges(item.components as any, 'component');
      }
    });
  }

  getNode(id: string): NodeData | undefined {
    return this.nodeIndex.get(id);
  }

  getNodeType(nodeId: string): string | null {
    const node = this.nodeIndex.get(nodeId);
    return node ? node.type : null;
  }

  getEdges(): Edge[] {
    return this.edges;
  }

  getArchitecture(): Architecture | null {
    return this.arch;
  }

  mapSvgNodesToIds(targetSvg: SVGElement | null): void {
    if (!targetSvg) return;

    const allGroups = targetSvg.querySelectorAll('g[role="img"]');
    const nodeGroups = Array.from(allGroups).filter(group => {
      if (!group.classList.contains('node')) return false;
      if (group.querySelector('line') || group.querySelector('path[marker-end]')) return false;
      const ariaLabel = group.getAttribute('aria-label');
      if (ariaLabel && (ariaLabel.includes(' to ') || ariaLabel.includes(' -> '))) return false;
      return true;
    }) as Element[];

    nodeGroups.forEach(group => {
      const ariaLabel = group.getAttribute('aria-label');
      const titleEl = group.querySelector('title');
      const title = titleEl ? titleEl.textContent : '';
      const searchText = title || ariaLabel;

      if (!searchText) return;

      let bestMatch: string | null = null;
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
          } else if (normalizedSearch.replace(/\s+/g, '') === normalizedName.replace(/\s+/g, '') ||
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

  private normalizeString(str: string | null): string {
    if (!str || typeof str !== 'string') return '';
    return str.toLowerCase().replace(/\s+/g, '').replace(/^(the|a|an)\s+/i, '');
  }

  matchesSearch(nodeId: string, query: string): boolean {
    if (!query) return true;
    const node = this.nodeIndex.get(nodeId);
    if (!node) return false;

    const q = query.toLowerCase();
    const idMatch = node.id.toLowerCase().includes(q);
    const nameMatch = node.name.toLowerCase().includes(q);
    const descMatch = !!(node.description && node.description.toLowerCase().includes(q));

    return idMatch || nameMatch || descMatch;
  }
}

customElements.define('sruja-node-index', SrujaNodeIndex);
