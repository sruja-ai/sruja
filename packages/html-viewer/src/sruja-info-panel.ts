// packages/html-viewer/src/sruja-info-panel.ts
// Info panel component for displaying node details

interface NodeData {
  id: string;
  type: string;
  name: string;
  description: string;
}

interface Architecture {
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
}

interface SrujaNodeIndexElement extends HTMLElement {
  getNode(id: string): NodeData | undefined;
  getArchitecture(): Architecture | null;
}

class SrujaInfoPanel extends HTMLElement {
  private nodeIndex: SrujaNodeIndexElement | null = null;
  private arch: Architecture | null = null;
  private titleEl: HTMLElement | null = null;
  private descEl: HTMLElement | null = null;

  connectedCallback(): void {
    const indexEl = document.querySelector('sruja-node-index') as SrujaNodeIndexElement;
    if (indexEl) {
      this.nodeIndex = indexEl;
      this.arch = indexEl.getArchitecture();
    }
    
    this.titleEl = this.querySelector('#info-title');
    this.descEl = this.querySelector('#info-description');
  }

  showNode(nodeId: string): void {
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
    } else if (node.type === 'adr' && this.arch?.adrs) {
      const adr = this.arch.adrs.find(a => a.id === nodeId);
      if (adr) {
        title = `${adr.id}${adr.status ? ' [' + adr.status + ']' : ''}`;
        desc = adr.decision || adr.context || adr.title || '';
        if (adr.consequences) {
          desc += '\n\nConsequences: ' + adr.consequences;
        }
      }
    } else if (node.type === 'scenario' && this.arch?.scenarios) {
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
    } else if (node.type === 'flow' && this.arch?.flows) {
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
    
    if (this.titleEl) this.titleEl.textContent = title;
    if (this.descEl) this.descEl.textContent = desc;
    this.classList.add('visible');
  }

  hide(): void {
    this.classList.remove('visible');
  }
}

customElements.define('sruja-info-panel', SrujaInfoPanel);
