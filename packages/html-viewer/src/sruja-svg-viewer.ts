// packages/html-viewer/src/sruja-svg-viewer.ts
// SVG viewer with pan and zoom functionality

class SrujaSvgViewer extends HTMLElement {
  private scale = 1;
  private panX = 0;
  private panY = 0;
  private isDragging = false;
  private startX = 0;
  private startY = 0;
  private container: HTMLElement | null = null;

  connectedCallback(): void {
    this.container = this.closest('#container') as HTMLElement;
    if (!this.container) return;
    
    this.setupPanZoom();
    this.setupZoomControls();
  }

  private setupPanZoom(): void {
    if (!this.container) return;
    
    this.container.addEventListener('mousedown', (e) => {
      if (e.button === 0 && !(e.target as Element).closest('.sidebar') && !(e.target as Element).closest('.controls')) {
        this.isDragging = true;
        this.container!.classList.add('dragging');
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
      if (this.container) this.container.classList.remove('dragging');
    });
    
    this.container.addEventListener('mouseleave', () => {
      this.isDragging = false;
      if (this.container) this.container.classList.remove('dragging');
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

  private setupZoomControls(): void {
    (window as any).zoomIn = () => {
      this.scale = Math.min(5, this.scale * 1.2);
      this.updateTransform();
    };
    
    (window as any).zoomOut = () => {
      this.scale = Math.max(0.1, this.scale / 1.2);
      this.updateTransform();
    };
    
    (window as any).resetZoom = () => {
      this.scale = 1;
      this.panX = 0;
      this.panY = 0;
      this.updateTransform();
    };
  }

  private updateTransform(): void {
    const activeContainer = document.querySelector('.svg-container-level.active') as HTMLElement || 
                           document.getElementById('svg-container-all');
    const gridContainer = document.getElementById('svg-grid-container');
    
    if (activeContainer && !gridContainer) {
      activeContainer.style.transform = `translate(${this.panX}px, ${this.panY}px) scale(${this.scale})`;
    }
  }

  zoomToNode(nodeEl: Element, svg: SVGElement): void {
    if (!nodeEl || !svg || !this.container) return;
    
    const activeContainer = document.querySelector('.svg-container-level.active') as HTMLElement || 
                           document.getElementById('svg-container-all');
    if (!activeContainer) return;
    
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

  reset(): void {
    this.scale = 1;
    this.panX = 0;
    this.panY = 0;
    this.updateTransform();
  }
}

customElements.define('sruja-svg-viewer', SrujaSvgViewer);
