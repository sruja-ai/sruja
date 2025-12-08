// apps/studio-core/src/hooks/useClickToConnect.ts
import { useEffect } from 'react';
import type React from 'react';
import type { ViewerInstance } from '@sruja/viewer';

interface UseClickToConnectOptions {
  viewerRef: React.RefObject<ViewerInstance | null>;
  isAddingRelation: boolean;
  sourceNode: string | null;
  setSourceNode: (node: string | null) => void;
  setIsAddingRelation: (adding: boolean) => void;
  setModalConfig: (config: any) => void;
}

/**
 * Hook for click-to-connect relation creation
 */
export function useClickToConnect({
  viewerRef,
  isAddingRelation,
  sourceNode,
  setSourceNode,
  setIsAddingRelation,
  setModalConfig,
}: UseClickToConnectOptions): void {
  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer || !viewer.cy) return;

    const cy = viewer.cy;

    const handleTap = (evt: any) => {
      if (!isAddingRelation) return;

      const target = evt.target;
      if (target === cy) {
        // Clicked on background, cancel
        if (sourceNode) {
          cy.getElementById(sourceNode).removeStyle();
        }
        setSourceNode(null);
        setIsAddingRelation(false);
        return;
      }

      if (target.isNode()) {
        if (!sourceNode) {
          setSourceNode(target.id());
          // Visual feedback
          target.style('border-color', (getComputedStyle(document.documentElement).getPropertyValue('--color-error-500') || '#ef4444').trim());
          target.style('border-width', 4);
        } else {
          // Complete relation - Open Modal
          if (sourceNode !== target.id()) {
            setModalConfig({
              isOpen: true,
              title: 'Add Relation',
              placeholder: 'Enter verb (e.g., Uses)',
              type: 'relation',
              data: { source: sourceNode, target: target.id() },
            });
          } else {
            // Clicked same node, reset
            cy.getElementById(sourceNode).removeStyle();
            setSourceNode(null);
            setIsAddingRelation(false);
          }
        }
      }
    };

    cy.on('tap', handleTap);

    return () => {
      cy.removeListener('tap', handleTap);
      // Cleanup: reset style if unmounting mid-operation
      if (sourceNode && viewer.cy) {
        viewer.cy.getElementById(sourceNode).removeStyle();
      }
    };
  }, [viewerRef, isAddingRelation, sourceNode, setSourceNode, setIsAddingRelation, setModalConfig]);
}
