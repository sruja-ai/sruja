// UI components for Sruja Viewer
// Handles Requirements, ADRs, Scenarios pages, tabs, and highlighting

import type { ArchitectureJSON } from './types';
import type { SrujaViewer } from './viewer';
import { Colors } from '@sruja/shared/utils/cssVars';

export interface UIOptions {
  container: string | HTMLElement;
  data: ArchitectureJSON;
  viewer: SrujaViewer;
  onSelect?: (id: string | null) => void;
}

export class SrujaViewerUI {
  private container: HTMLElement;
  private data: ArchitectureJSON;
  private viewer: SrujaViewer;
  constructor(options: UIOptions) {
    const containerElement =
      typeof options.container === 'string'
        ? document.querySelector(options.container)
        : options.container;

    if (!containerElement || !(containerElement instanceof HTMLElement)) {
      throw new Error(`Container not found: ${options.container}`);
    }

    this.container = containerElement;
    this.data = options.data;
    this.viewer = options.viewer;
  }

  /**
   * Initialize the full UI with tabs and pages
   */
  async init(): Promise<void> {
    this.createUI();
    this.setupTabs();
    this.renderRequirementsPage();
    this.renderADRsPage();
    this.renderScenariosPage();
  }

  /**
   * Create the UI structure
   */
  private createUI(): void {
    // Check if UI already exists
    if (this.container.querySelector('.sruja-ui-container')) {
      return;
    }

    const uiContainer = document.createElement('div');
    uiContainer.className = 'sruja-ui-container';
    uiContainer.style.cssText = 'display: flex; flex-direction: column; height: 100%;';

    // Create tabs
    const tabs = document.createElement('div');
    tabs.className = 'sruja-tabs';
    tabs.style.cssText = 'display: flex; gap: 8px; padding: 12px 16px; border-bottom: 1px solid var(--color-border); background: var(--color-background);';
    
    const tabDiagram = this.createTab('diagram', 'Diagram', true);
    const tabRequirements = this.createTab('requirements', 'Requirements', false);
    const tabADRs = this.createTab('adrs', 'ADRs', false);
    const tabScenarios = this.createTab('scenarios', 'Scenarios', false);

    tabs.appendChild(tabDiagram);
    tabs.appendChild(tabRequirements);
    tabs.appendChild(tabADRs);
    tabs.appendChild(tabScenarios);

    // Create content area
    const content = document.createElement('div');
    content.className = 'sruja-content';
    content.style.cssText = 'flex: 1; display: flex; flex-direction: column; overflow: hidden;';

    // Diagram tab content (existing viewer container)
    const diagramContent = document.createElement('div');
    diagramContent.className = 'sruja-tab-content active';
    diagramContent.id = 'sruja-tab-diagram';
    diagramContent.style.cssText = 'flex: 1; display: flex; flex-direction: column; overflow: hidden;';
    
    // Move existing viewer container into diagram tab
    const viewerContainer = this.container.querySelector('#sruja-app') || this.container;
    if (viewerContainer === this.container) {
      // Create new viewer container
      const newViewerContainer = document.createElement('div');
      newViewerContainer.id = 'sruja-app';
      newViewerContainer.style.cssText = 'flex: 1; position: relative; min-height: 0;';
      diagramContent.appendChild(newViewerContainer);
    } else {
      diagramContent.appendChild(viewerContainer);
    }

    // Requirements tab content
    const reqContent = document.createElement('div');
    reqContent.className = 'sruja-tab-content';
    reqContent.id = 'sruja-tab-requirements';
    reqContent.style.cssText = 'flex: 1; overflow-y: auto; padding: 24px; background: #fff; display: none;';

    // ADRs tab content
    const adrContent = document.createElement('div');
    adrContent.className = 'sruja-tab-content';
    adrContent.id = 'sruja-tab-adrs';
    adrContent.style.cssText = 'flex: 1; overflow-y: auto; padding: 24px; background: #fff; display: none;';

    // Scenarios tab content
    const scenarioContent = document.createElement('div');
    scenarioContent.className = 'sruja-tab-content';
    scenarioContent.id = 'sruja-tab-scenarios';
    scenarioContent.style.cssText = 'flex: 1; overflow-y: auto; padding: 24px; background: #fff; display: none;';

    content.appendChild(diagramContent);
    content.appendChild(reqContent);
    content.appendChild(adrContent);
    content.appendChild(scenarioContent);

    uiContainer.appendChild(tabs);
    uiContainer.appendChild(content);

    // Clear container and add UI
    this.container.innerHTML = '';
    this.container.appendChild(uiContainer);
  }

  private createTab(id: string, label: string, active: boolean): HTMLElement {
    const tab = document.createElement('button');
    tab.className = `sruja-tab ${active ? 'active' : ''}`;
    tab.setAttribute('data-tab', id);
    tab.textContent = label;
    tab.style.cssText = `
      padding: 8px 16px;
      border: none;
      background: transparent;
      color: ${active ? Colors.primary() : Colors.textSecondary()};
      cursor: pointer;
      font-size: 0.875rem;
      font-weight: 500;
      border-bottom: 2px solid ${active ? Colors.primary() : 'transparent'};
      transition: all 0.2s;
    `;
    return tab;
  }

  private setupTabs(): void {
    const tabs = this.container.querySelectorAll('.sruja-tab');
    const contents = this.container.querySelectorAll('.sruja-tab-content');

    tabs.forEach(tab => {
      tab.addEventListener('click', () => {
        const tabId = tab.getAttribute('data-tab');

        // Update active tab
        tabs.forEach(t => {
          t.classList.remove('active');
          (t as HTMLElement).style.color = Colors.textSecondary();
          (t as HTMLElement).style.borderBottomColor = 'transparent';
        });
        tab.classList.add('active');
        (tab as HTMLElement).style.color = Colors.primary();
        (tab as HTMLElement).style.borderBottomColor = Colors.primary();

        // Update active content
        contents.forEach(c => {
          c.classList.remove('active');
          (c as HTMLElement).style.display = 'none';
        });
        const content = this.container.querySelector(`#sruja-tab-${tabId}`);
        if (content) {
          content.classList.add('active');
          (content as HTMLElement).style.display = 'flex';
        }
      });
    });
  }

  private renderRequirementsPage(): void {
    const container = this.container.querySelector('#sruja-tab-requirements');
    if (!container) return;

    const arch = this.data.architecture || {};
    const reqs = arch.requirements || [];

    if (reqs.length === 0) {
      container.innerHTML = '<div style="padding: 48px; text-align: center; color: var(--color-text-tertiary);"><p>No requirements defined</p></div>';
      return;
    }

    let html = '<h2 style="margin: 0 0 24px 0; font-size: 1.5rem; font-weight: 600;">Requirements</h2>';

    reqs.forEach(req => {
      const typeClass = `type-${req.type || 'functional'}`;
      const title = req.title || req.description || '';
      html += `<div class="req-card" data-req-id="${req.id}" onclick="window.srujaUI?.highlightRequirement('${req.id}')">`;
      html += `<h3>${req.id}: ${this.escapeHtml(title)}</h3>`;
      html += `<div class="meta"><span class="type-badge ${typeClass}">${req.type || 'functional'}</span></div>`;
      html += `</div>`;
    });

    container.innerHTML = html;
  }

  private renderADRsPage(): void {
    const container = this.container.querySelector('#sruja-tab-adrs');
    if (!container) return;

    const arch = this.data.architecture || {};
    const adrs = arch.adrs || [];

    if (adrs.length === 0) {
      container.innerHTML = '<div style="padding: 48px; text-align: center; color: var(--color-text-tertiary);"><p>No ADRs defined</p></div>';
      return;
    }

    let html = '<h2 style="margin: 0 0 24px 0; font-size: 1.5rem; font-weight: 600;">Architecture Decision Records</h2>';

    adrs.forEach(adr => {
      const statusStr = (adr.status || 'proposed') as string;
      const status = statusStr.toLowerCase();
      html += `<div class="adr-card" data-adr-id="${adr.id}" onclick="window.srujaUI?.highlightADR('${adr.id}')">`;
      html += `<h3>${adr.id}: ${this.escapeHtml(adr.title || '')}</h3>`;
      if (adr.status) {
        html += `<span class="status ${status}">${this.escapeHtml(statusStr)}</span>`;
      }
      if (adr.context) {
        html += `<div class="section"><h4>Context</h4><p>${this.escapeHtml(adr.context as string)}</p></div>`;
      }
      if (adr.decision) {
        html += `<div class="section"><h4>Decision</h4><p>${this.escapeHtml(adr.decision as string)}</p></div>`;
      }
      if (adr.consequences) {
        html += `<div class="section"><h4>Consequences</h4><p>${this.escapeHtml(adr.consequences as string)}</p></div>`;
      }
      html += `</div>`;
    });

    container.innerHTML = html;
  }

  private renderScenariosPage(): void {
    const container = this.container.querySelector('#sruja-tab-scenarios');
    if (!container) return;

    const arch = this.data.architecture || {};
    const scenarios = arch.scenarios || [];

    if (scenarios.length === 0) {
      container.innerHTML = '<div style="padding: 48px; text-align: center; color: var(--color-text-tertiary);"><p>No scenarios defined</p></div>';
      return;
    }

    let html = '<h2 style="margin: 0 0 24px 0; font-size: 1.5rem; font-weight: 600;">Scenarios</h2>';

    scenarios.forEach(scenario => {
      const title = scenario.title || scenario.label || scenario.id;
      html += `<div class="scenario-card">`;
      html += `<h3>${this.escapeHtml(title)}</h3>`;
      if (scenario.description) {
        html += `<p style="color: var(--color-text-secondary); margin-bottom: 16px;">${this.escapeHtml(scenario.description as string)}</p>`;
      }
      html += `<button onclick="window.srujaUI?.showScenarioInDiagram('${scenario.id}')" style="padding: 8px 16px; background: var(--color-primary); color: var(--color-background); border: none; border-radius: 6px; cursor: pointer; margin-bottom: 16px;">Show in Diagram</button>`;

      if (scenario.steps && Array.isArray(scenario.steps) && scenario.steps.length > 0) {
        html += `<div class="sequence-diagram">`;
        scenario.steps.forEach((step: { from: string; to: string; description?: string }) => {
          html += `<div class="sequence-step">`;
          html += `<span class="from">${this.escapeHtml(step.from)}</span>`;
          html += `<span class="arrow">→</span>`;
          html += `<span class="to">${this.escapeHtml(step.to)}</span>`;
          if (step.description) {
            html += `<span class="desc">${this.escapeHtml(step.description)}</span>`;
          }
          html += `</div>`;
        });
        html += `</div>`;
      }
      html += `</div>`;
    });

    container.innerHTML = html;
  }

  highlightRequirement(reqId: string): void {
    // Switch to diagram tab
    const diagramTab = this.container.querySelector('.sruja-tab[data-tab="diagram"]') as HTMLElement;
    if (diagramTab) diagramTab.click();

    this.clearHighlights();

    // TODO: Implement tag-based highlighting
    // For now, show message
    setTimeout(() => {
      console.log(`Requirement ${reqId} clicked. Component highlighting will be implemented based on tags/relationships.`);
    }, 100);
  }

  highlightADR(adrId: string): void {
    // Switch to diagram tab
    const diagramTab = this.container.querySelector('.sruja-tab[data-tab="diagram"]') as HTMLElement;
    if (diagramTab) diagramTab.click();

    this.clearHighlights();

    // TODO: Implement tag-based highlighting
    setTimeout(() => {
      console.log(`ADR ${adrId} clicked. Component highlighting will be implemented based on tags/relationships.`);
    }, 100);
  }

  showScenarioInDiagram(scenarioId: string): void {
    // Switch to diagram tab
    const diagramTab = this.container.querySelector('.sruja-tab[data-tab="diagram"]') as HTMLElement;
    if (diagramTab) diagramTab.click();

    this.clearHighlights();

    const scenario = (this.data.architecture.scenarios || []).find(s => s.id === scenarioId);

    if (scenario && scenario.steps && Array.isArray(scenario.steps) && this.viewer.cyInstance) {
      const cy = this.viewer.cyInstance;
      const nodeIds = new Set<string>();

      scenario.steps.forEach((step: { from: string; to: string }) => {
        nodeIds.add(step.from);
        nodeIds.add(step.to);
      });

      // Resolve and highlight nodes
      setTimeout(() => {
        nodeIds.forEach(nodeId => {
          // Try direct match first
          let node = cy.getElementById(nodeId);
          if (node.length === 0) {
            // Try to resolve (e.g., "WebApp" -> "ECommerce.WebApp")
            const arch = this.data.architecture;
            if (arch.systems) {
              for (const system of arch.systems) {
                const qualifiedId = `${system.id}.${nodeId}`;
                node = cy.getElementById(qualifiedId);
                if (node.length > 0) break;
              }
            }
          }

          if (node && node.length > 0) {
            node.addClass('highlight');
            node.style('background-color', Colors.primary50());
            node.style('border-color', Colors.primary());
            node.style('border-width', '3px');
          }
        });

        // Fit to highlighted nodes
        const highlighted = cy.elements('.highlight');
        if (highlighted.length > 0) {
          cy.fit(highlighted, 100);
        }
      }, 200);
    }
  }

  private clearHighlights(): void {
    if (this.viewer.cyInstance) {
      const cy = this.viewer.cyInstance;
      cy.elements().removeClass('highlight');
      cy.elements().style('background-color', '');
      cy.elements().style('border-color', '');
      cy.elements().style('border-width', '');
    }
  }

  private escapeHtml(text: string | undefined): string {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }
}

