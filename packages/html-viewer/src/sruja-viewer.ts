// packages/html-viewer/src/sruja-viewer.ts
// Main viewer component that orchestrates all functionality

interface SrujaNodeIndex extends HTMLElement {
  getNode(id: string): { id: string; type: string; name: string; description: string } | undefined;
  getNodeType(nodeId: string): string | null;
  getEdges(): Array<{ from: string; to: string; label: string; type: string }>;
  getArchitecture(): {
    systems?: Array<{ containers?: Array<{ components?: Array<unknown> }>; datastores?: Array<unknown>; queues?: Array<unknown> }>;
    persons?: Array<unknown>;
    requirements?: Array<unknown>;
    adrs?: Array<unknown>;
    scenarios?: Array<unknown>;
    flows?: Array<unknown>;
    deployment?: Array<unknown>;
  } | null;
  mapSvgNodesToIds(targetSvg: SVGElement | null): void;
  matchesSearch(nodeId: string, query: string): boolean;
}

interface SrujaInfoPanel extends HTMLElement {
  showNode(nodeId: string): void;
  hide(): void;
}

interface SrujaSvgViewer extends HTMLElement {
  zoomToNode(nodeEl: Element, svg: SVGElement): void;
}

class SrujaViewer extends HTMLElement {
  private nodeIndex: SrujaNodeIndex | null = null;
  private infoPanel: SrujaInfoPanel | null = null;
  private currentFilter = 'all';
  private searchQuery = '';
  private selectedNodeId: string | null = null;

  connectedCallback(): void {
    this.nodeIndex = document.querySelector('sruja-node-index') as SrujaNodeIndex;
    this.infoPanel = document.querySelector('sruja-info-panel') as SrujaInfoPanel;
    
    if (!this.nodeIndex) {
      const indexEl = document.querySelector('sruja-node-index');
      if (indexEl) {
        indexEl.addEventListener('index-ready', () => this.initialize());
      }
      return;
    }
    
    this.initialize();
  }

  private initialize(): void {
    this.setupSearch();
    this.setupFilters();
    this.setupViewTabs();
    this.setupKeyboardShortcuts();
    // Attach interactions will be called from updateVisibility
    this.updateVisibility();
  }

  private setupSearch(): void {
    const searchBox = document.getElementById('search-box') as HTMLInputElement;
    if (!searchBox) return;
    
    searchBox.addEventListener('input', (e) => {
      this.searchQuery = (e.target as HTMLInputElement).value;
      this.updateVisibility();
    });
  }

  private setupFilters(): void {
    const indexEl = this.nodeIndex;
    if (!indexEl) return;
    
    const arch = indexEl.getArchitecture();
    if (!arch) return;
    
    const hasElements: Record<string, boolean> = {
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
        const tab = document.querySelector(`.filter-tab[data-filter="${filterType}"]`) as HTMLButtonElement;
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
        const clickedTab = (e.currentTarget || (e.target as Element).closest('.filter-tab') || e.target) as HTMLButtonElement;
        if (!clickedTab || !clickedTab.classList || clickedTab.disabled) return;
        
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

  private setupViewTabs(): void {
    document.querySelectorAll('.view-tab').forEach(tab => {
      tab.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        const clickedTab = (e.currentTarget || (e.target as Element).closest('.view-tab') || e.target) as HTMLElement;
        if (!clickedTab || !clickedTab.classList) return;
        
        const view = clickedTab.getAttribute('data-view');
        if (!view) return;
        
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

  private populateRequirementsView(): void {
    if (!this.nodeIndex) return;
    const arch = this.nodeIndex.getArchitecture();
    if (!arch) return;
    
    const requirementsList = document.getElementById('requirements-list');
    const adrsList = document.getElementById('adrs-list');
    
    if (requirementsList) {
      requirementsList.innerHTML = '';
      (arch.requirements || []).forEach((req: any) => {
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
      (arch.adrs || []).forEach((adr: any) => {
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
      (arch.scenarios || []).forEach((scenario: any) => {
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
          
          scenario.steps.forEach((step: any, idx: number) => {
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
      (arch.flows || []).forEach((flow: any) => {
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
          
          flow.steps.forEach((step: any, idx: number) => {
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

  private setupKeyboardShortcuts(): void {
    const searchBox = document.getElementById('search-box') as HTMLInputElement;
    document.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
        e.preventDefault();
        if (searchBox) searchBox.focus();
      } else if (e.key === 'Escape') {
        this.selectedNodeId = null;
        this.highlightNode(null);
        if (searchBox) searchBox.value = '';
        this.searchQuery = '';
        this.updateVisibility();
      }
    });
  }

  private matchesFilter(nodeId: string | null): boolean {
    if (!nodeId) return false;
    if (this.currentFilter === 'all') return true;
    if (!this.nodeIndex) return false;
    const type = this.nodeIndex.getNodeType(nodeId);
    return type === this.currentFilter;
  }

  private matchesSearch(nodeId: string | null): boolean {
    if (!nodeId || !this.nodeIndex) return false;
    return this.nodeIndex.matchesSearch(nodeId, this.searchQuery);
  }

  private updateVisibility(): void {
    // Hide all SVG containers and grid
    document.querySelectorAll('.svg-container-level').forEach(el => {
      el.classList.remove('active');
      (el as HTMLElement).style.display = 'none';
    });
    const gridContainer = document.getElementById('svg-grid-container');
    if (gridContainer) {
      gridContainer.style.display = 'none';
    }
    
    const svgContainerAll = document.getElementById('svg-container-all');
    const svg = svgContainerAll ? svgContainerAll.querySelector('svg') as SVGElement : null;
    
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
    } else if (['system', 'container'].includes(this.currentFilter)) {
      this.showGrid(this.currentFilter);
    } else if (['scenario', 'flow'].includes(this.currentFilter)) {
      // Scenarios and flows have their own separate SVG diagrams
      // Show them in grid view
      this.showGrid(this.currentFilter);
    } else {
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

  private showGrid(filterType: string): void {
    if (!this.nodeIndex) return;
    
    const prefix = `svg-container-${filterType}-`;
    const containers = document.querySelectorAll(`[id^="${prefix}"]`);
    if (containers.length === 0) return;
    
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
      if (!id) return;
      
      const nodeId = filterType === 'container' ? id.split('.')[1] : id;
      if (!this.matchesSearch(nodeId)) return;
      
      const svg = cont.querySelector('svg') as SVGElement;
      if (!svg) return;
      
      const node = this.nodeIndex!.getNode(nodeId);
      const gridItem = document.createElement('div');
      gridItem.className = 'svg-grid-item';
      const title = document.createElement('h4');
      title.textContent = node ? (node.name || nodeId) : nodeId;
      const wrapper = document.createElement('div');
      wrapper.className = 'svg-wrapper';
      const svgClone = svg.cloneNode(true) as SVGElement;
      wrapper.appendChild(svgClone);
      gridItem.appendChild(title);
      gridItem.appendChild(wrapper);
      gridContainer.appendChild(gridItem);
      
      this.attachNodeInteractions(svgClone);
    });
  }

  private updateNodeVisibility(svg: SVGElement, useFilter = false): void {
    if (!this.nodeIndex) return;
    
    const nodes = svg.querySelectorAll('[data-node-id]');
    nodes.forEach(nodeEl => {
      const nodeId = nodeEl.getAttribute('data-node-id');
      const visible = useFilter 
        ? this.matchesFilter(nodeId) && this.matchesSearch(nodeId)
        : this.matchesSearch(nodeId);
      
      if (visible) {
        nodeEl.classList.remove('node-filtered-out');
      } else {
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
        } else {
          edge.classList.add('edge-dimmed');
        }
      }
    });
  }

  private attachNodeInteractions(targetSvg: SVGElement | null): void {
    if (!targetSvg || !this.nodeIndex) return;
    
    this.nodeIndex.mapSvgNodesToIds(targetSvg);
    const nodes = targetSvg.querySelectorAll('[data-node-id]');
    
    nodes.forEach(nodeEl => {
      const nodeId = nodeEl.getAttribute('data-node-id');
      const node = this.nodeIndex!.getNode(nodeId || '');
      if (!node) return;
      
      // Remove existing listeners by cloning
      const newNodeEl = nodeEl.cloneNode(true) as Element;
      if (nodeEl.parentNode) {
        nodeEl.parentNode.replaceChild(newNodeEl, nodeEl);
      }
      
      newNodeEl.addEventListener('click', (e) => {
        e.stopPropagation();
        this.selectedNodeId = this.selectedNodeId === nodeId ? null : nodeId;
        this.highlightNode(this.selectedNodeId);
        
        if (this.selectedNodeId && this.infoPanel) {
          this.infoPanel.showNode(this.selectedNodeId);
        } else if (this.infoPanel) {
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

  private highlightNode(nodeId: string | null): void {
    const activeContainer = document.querySelector('.svg-container-level.active') as HTMLElement || 
                           document.getElementById('svg-container-all');
    if (!activeContainer) return;
    
    const activeSvg = activeContainer.querySelector('svg') as SVGElement;
    if (!activeSvg || !this.nodeIndex) return;
    
    activeSvg.querySelectorAll('.node-selected').forEach(el => el.classList.remove('node-selected'));
    activeSvg.querySelectorAll('.edge-highlight').forEach(el => el.classList.remove('edge-highlight'));
    
    if (!nodeId) return;
    
    const nodeEl = activeSvg.querySelector(`[data-node-id="${nodeId}"]`);
    if (nodeEl) {
      nodeEl.classList.add('node-selected');
      
      const edges = this.nodeIndex.getEdges();
      edges.filter(e => e.from === nodeId || e.to === nodeId).forEach(edge => {
        const edgeEl = activeSvg.querySelector(`[data-from="${edge.from}"][data-to="${edge.to}"]`);
        if (edgeEl) edgeEl.classList.add('edge-highlight');
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
