// apps/studio-core/src/hooks/useDragToConnect.ts
import { useEffect, useRef } from 'react';
import type React from 'react';
import type { ViewerInstance } from '@sruja/viewer';

interface UseDragToConnectOptions {
  viewerRef: React.RefObject<ViewerInstance | null>;
  setModalConfig: (config: any) => void;
}

type ConnectionHandle = HTMLElement;
type HandleSide = 'right' | 'left' | 'top' | 'bottom';

/**
 * Hook for drag-to-connect relation creation (Draw.io-style)
 */
export function useDragToConnect({
  viewerRef,
  setModalConfig,
}: UseDragToConnectOptions): void {
  const connectionHandlesRef = useRef<Map<string, ConnectionHandle[]>>(new Map());
  const isConnectingRef = useRef(false);
  const connectSourceNodeRef = useRef<string | null>(null);
  const connectSourceHandleRef = useRef<HandleSide | null>(null);
  const tempEdgeRef = useRef<any>(null);

  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer || !viewer.cy) return;

    const cy = viewer.cy;
    const connectionHandles = connectionHandlesRef.current;
    let isConnecting = isConnectingRef.current;
    let connectSourceNode = connectSourceNodeRef.current;
    let connectSourceHandle = connectSourceHandleRef.current;
    let tempEdge = tempEdgeRef.current;

    // Create visible connection handles on nodes (like Draw.io)
    const createConnectionHandles = (node: any) => {
      if (!node.isNode() || isConnecting) return;

      const nodeId = node.id();
      if (connectionHandles.has(nodeId)) return; // Already created

      const handles: ConnectionHandle[] = [];
      const container = cy.container() as HTMLElement;

      // Get node position in rendered (screen) coordinates
      const nodePos = node.renderedPosition();
      const nodeWidth = node.renderedWidth();
      const nodeHeight = node.renderedHeight();

      // Create handles on all 4 sides
      const handlePositions: Array<{ side: HandleSide; x: number; y: number }> = [
        { side: 'right', x: nodePos.x + nodeWidth / 2, y: nodePos.y },
        { side: 'left', x: nodePos.x - nodeWidth / 2, y: nodePos.y },
        { side: 'top', x: nodePos.x, y: nodePos.y - nodeHeight / 2 },
        { side: 'bottom', x: nodePos.x, y: nodePos.y + nodeHeight / 2 },
      ];

      handlePositions.forEach(({ side, x, y }) => {
        const handle = document.createElement('div');
        handle.className = 'connection-handle';
        handle.style.cssText = `
          position: absolute;
          width: 12px;
          height: 12px;
          background: (getComputedStyle(document.documentElement).getPropertyValue('--color-primary') || '#3b82f6').trim();
          border: 2px solid white;
          border-radius: 50%;
          cursor: crosshair;
          pointer-events: auto;
          z-index: 1000;
          transform: translate(-50%, -50%);
          transition: all 0.2s ease;
          box-shadow: 0 2px 4px rgba(0,0,0,0.2);
        `;

        // Position handle - renderedPosition is already in screen coordinates
        const containerRect = container.getBoundingClientRect();

        handle.style.left = `${containerRect.left + x}px`;
        handle.style.top = `${containerRect.top + y}px`;

        // Store side info
        handle.setAttribute('data-side', side);
        handle.setAttribute('data-node-id', nodeId);

        // Add hover effect
        handle.addEventListener('mouseenter', () => {
          handle.style.transform = 'translate(-50%, -50%) scale(1.3)';
          handle.style.background = (getComputedStyle(document.documentElement).getPropertyValue('--color-primary-hover') || '#2563eb').trim();
        });
        handle.addEventListener('mouseleave', () => {
          if (!isConnecting) {
            handle.style.transform = 'translate(-50%, -50%) scale(1)';
            handle.style.background = '#3b82f6';
          }
        });

        // Drag start
        handle.addEventListener('mousedown', (e) => {
          e.stopPropagation();
          e.preventDefault();

          connectSourceNode = nodeId;
          connectSourceHandle = side;
          isConnecting = true;
          connectSourceNodeRef.current = connectSourceNode;
          connectSourceHandleRef.current = connectSourceHandle;
          isConnectingRef.current = isConnecting;

          // Visual feedback
          node.ungrabify();
          node.style('border-color', '#3b82f6');
          node.style('border-width', 3);
          handle.style.background = '#10b981';
          handle.style.transform = 'translate(-50%, -50%) scale(1.5)';

          container.style.cursor = 'crosshair';
        });

        container.appendChild(handle);
        handles.push(handle);
      });

      connectionHandles.set(nodeId, handles);
    };

    // Remove connection handles
    const removeConnectionHandles = (node: any) => {
      if (!node.isNode() || isConnecting) return;

      const nodeId = node.id();
      const handles = connectionHandles.get(nodeId);
      if (!handles) return;

      handles.forEach((handle) => {
        if (handle.parentNode) {
          handle.parentNode.removeChild(handle);
        }
      });

      connectionHandles.delete(nodeId);
    };

    // Update handle positions when view changes (pan/zoom)
    const updateHandlePositions = () => {
      if (isConnecting) return; // Don't update during connection

      connectionHandles.forEach((handles, nodeId) => {
        const node = cy.getElementById(nodeId);
        if (!node || !node.isNode()) return;

        const container = cy.container() as HTMLElement;
        const nodePos = node.renderedPosition();
        const nodeWidth = node.renderedWidth();
        const nodeHeight = node.renderedHeight();
        const containerRect = container.getBoundingClientRect();

        handles.forEach((handle) => {
          const side = handle.getAttribute('data-side');
          let x = nodePos.x;
          let y = nodePos.y;

          if (side === 'right') x += nodeWidth / 2;
          else if (side === 'left') x -= nodeWidth / 2;
          else if (side === 'top') y -= nodeHeight / 2;
          else if (side === 'bottom') y += nodeHeight / 2;

          // renderedPosition is already in screen coordinates
          handle.style.left = `${containerRect.left + x}px`;
          handle.style.top = `${containerRect.top + y}px`;
        });
      });
    };

    // Handle mouse move while connecting
    const handleMouseMove = (evt: MouseEvent | any) => {
      isConnecting = isConnectingRef.current;
      connectSourceNode = connectSourceNodeRef.current;

      if (!isConnecting || !connectSourceNode) return;

      // Get mouse position in graph coordinates
      const container = cy.container() as HTMLElement;
      const containerRect = container.getBoundingClientRect();
      const pan = cy.pan();
      const zoom = cy.zoom();

      // Convert screen coordinates to graph coordinates
      const clientX = evt.clientX || (evt.originalEvent && evt.originalEvent.clientX) || 0;
      const clientY = evt.clientY || (evt.originalEvent && evt.originalEvent.clientY) || 0;

      const graphX = (clientX - containerRect.left - pan.x) / zoom;
      const graphY = (clientY - containerRect.top - pan.y) / zoom;

      const sourceNode = cy.getElementById(connectSourceNode);
      if (!sourceNode) return;

      // Remove previous temp edge
      if (tempEdge) {
        tempEdge.remove();
      }

      // Check if mouse is over another node or its connection handle
      const nodesAtPos = cy.nodes().filter((n: any) => {
        if (n.id() === connectSourceNode) return false;
        const pos = n.position();
        const width = n.width();
        const height = n.height();
        const dist = Math.sqrt(Math.pow(graphX - pos.x, 2) + Math.pow(graphY - pos.y, 2));
        return dist < Math.max(width, height) / 2 + 30; // 30px tolerance to include handles
      });

      if (nodesAtPos.length > 0) {
        const targetNode = nodesAtPos[0];
        // Create temp edge to target node
        tempEdge = cy.add({
          group: 'edges',
          data: {
            id: `temp-edge-${Date.now()}`,
            source: connectSourceNode,
            target: targetNode.id(),
          },
          style: {
            'line-color': '#3b82f6',
            'line-style': 'dashed',
            'line-width': 2,
            'opacity': 0.7,
            'target-arrow-color': '#3b82f6',
            'target-arrow-shape': 'triangle',
          },
        });
        tempEdgeRef.current = tempEdge;
        // Highlight target node and show its handles
        targetNode.style('border-color', '#10b981');
        targetNode.style('border-width', 3);

        // Show target node's handles
        if (!connectionHandles.has(targetNode.id())) {
          createConnectionHandles(targetNode);
        }
      } else {
        // No target node - remove target highlights
        cy.nodes().forEach((n: any) => {
          if (n.id() !== connectSourceNode) {
            n.removeStyle();
          }
        });
      }
    };

    // Handle mouse up to complete connection
    const handleMouseUp = (evt: MouseEvent | any) => {
      isConnecting = isConnectingRef.current;
      connectSourceNode = connectSourceNodeRef.current;

      if (!isConnecting || !connectSourceNode) return;

      // Get mouse position in graph coordinates
      const container = cy.container() as HTMLElement;
      const containerRect = container.getBoundingClientRect();
      const pan = cy.pan();
      const zoom = cy.zoom();

      const clientX = evt.clientX || (evt.originalEvent && evt.originalEvent.clientX) || 0;
      const clientY = evt.clientY || (evt.originalEvent && evt.originalEvent.clientY) || 0;

      const graphX = (clientX - containerRect.left - pan.x) / zoom;
      const graphY = (clientY - containerRect.top - pan.y) / zoom;

      let targetNodeId: string | null = null;

      // Check if mouse is over another node or its connection handle
      const nodesAtPos = cy.nodes().filter((n: any) => {
        if (n.id() === connectSourceNode) return false;
        const pos = n.position();
        const width = n.width();
        const height = n.height();
        const dist = Math.sqrt(Math.pow(graphX - pos.x, 2) + Math.pow(graphY - pos.y, 2));
        return dist < Math.max(width, height) / 2 + 30; // 30px tolerance to include handles
      });

      if (nodesAtPos.length > 0) {
        targetNodeId = nodesAtPos[0].id();
      }

      // Reset cursor
      container.style.cursor = '';

      // Reset all node styles and handles
      cy.nodes().forEach((n: any) => {
        n.removeStyle();
        n.grabify(); // Re-enable dragging

        // Reset handle styles
        const handles = connectionHandles.get(n.id());
        if (handles) {
          handles.forEach((handle) => {
            handle.style.background = '#3b82f6';
            handle.style.transform = 'translate(-50%, -50%) scale(1)';
          });
        }
      });

      // Remove temp edge
      if (tempEdge) {
        tempEdge.remove();
        tempEdge = null;
        tempEdgeRef.current = null;
      }

      // Create relation if valid target
      if (targetNodeId) {
        setModalConfig({
          isOpen: true,
          title: 'Add Relation',
          placeholder: 'Enter verb (e.g., Uses)',
          type: 'relation',
          data: { source: connectSourceNode, target: targetNodeId },
        });
      }

      connectSourceNode = null;
      connectSourceHandle = null;
      isConnecting = false;
      connectSourceNodeRef.current = null;
      connectSourceHandleRef.current = null;
      isConnectingRef.current = false;
    };

    // Show connection handles on node hover
    cy.on('mouseover', 'node', (evt: any) => {
      createConnectionHandles(evt.target);
    });

    cy.on('mouseout', 'node', (evt: any) => {
      isConnecting = isConnectingRef.current;
      if (!isConnecting) {
        removeConnectionHandles(evt.target);
      }
    });

    // Update handle positions on pan/zoom
    cy.on('pan', updateHandlePositions);
    cy.on('zoom', updateHandlePositions);
    cy.on('layoutstop', updateHandlePositions);

    // Global mouse events for connection dragging
    const container = cy.container() as HTMLElement;
    container.addEventListener('mousemove', handleMouseMove);
    container.addEventListener('mouseup', handleMouseUp);

    return () => {
      cy.removeListener('mouseover', 'node', createConnectionHandles);
      cy.removeListener('mouseout', 'node', removeConnectionHandles);
      cy.removeListener('pan', updateHandlePositions);
      cy.removeListener('zoom', updateHandlePositions);
      cy.removeListener('layoutstop', updateHandlePositions);

      // Remove global mouse listeners
      if (container) {
        container.removeEventListener('mousemove', handleMouseMove);
        container.removeEventListener('mouseup', handleMouseUp);
      }

      // Cleanup connection handles
      connectionHandles.forEach((handles) => {
        handles.forEach((handle) => {
          if (handle.parentNode) {
            handle.parentNode.removeChild(handle);
          }
        });
      });
      connectionHandles.clear();

      // Cleanup temp edge
      if (tempEdge) {
        tempEdge.remove();
      }

      // Reset cursor
      if (cy.container()) {
        (cy.container() as HTMLElement).style.cursor = '';
      }

      // Reset node styles
      cy.nodes().forEach((n: any) => {
        n.removeStyle();
        n.grabify(); // Re-enable dragging
      });
    };
  }, [viewerRef, setModalConfig]);
}
