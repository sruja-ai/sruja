// apps/studio-core/src/components/ViewerPane.tsx
import React from 'react';
import { PropertiesPanel } from './PropertiesPanel';
import { X } from 'lucide-react';
import { ViewerInstance } from '@sruja/viewer';

interface ViewerPaneProps {
  containerRef: React.RefObject<HTMLDivElement | null>;
  viewerRef?: React.RefObject<ViewerInstance | null>; // Add viewerRef
  showProperties: boolean;
  onCloseProperties: () => void;
  selectedNodeId: string | null;
  archData: ArchitectureJSON | null;
  onPropertiesUpdate: (updates: any) => void;
  onAddNode: (type: 'person' | 'system' | 'container' | 'component' | 'datastore' | 'queue' | 'requirement' | 'adr' | 'deployment') => void;
  onToggleRelation: () => void;
  isAddingRelation: boolean;
  sourceNode: string | null;
  currentLevel?: number;
  onSetLevel: (level: number) => void;
  zoomLevel: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFitToScreen: () => void;
  onToggleCollapse: () => void;
  onDelete: () => void;
  onShare?: () => void;
  onUndo?: () => void;
  onRedo?: () => void;
  onExport?: (format: 'svg' | 'png') => void;
  onShowHelp?: () => void;
}

export function ViewerPane({
  containerRef,
  viewerRef,
  showProperties,
  onCloseProperties,
  selectedNodeId,
  archData,
  onPropertiesUpdate,
  onAddNode,
  onToggleRelation,
  isAddingRelation,
  sourceNode,
  currentLevel,
  onSetLevel,
  zoomLevel,
  onZoomIn,
  onZoomOut,
  onFitToScreen,
  onToggleCollapse,
  onDelete,
  onShare,
  onUndo,
  onRedo,
  onExport,
  onShowHelp,
}: ViewerPaneProps) {

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    if (!archData || !onPropertiesUpdate) return;

    try {
      const data = JSON.parse(e.dataTransfer.getData('application/json'));
      
      // Calculate position
      let position = undefined;
      if (viewerRef?.current?.cy) {
        const cy = viewerRef.current.cy;
        const pan = cy.pan();
        const zoom = cy.zoom();

        // Convert screen to model coordinates
        // Model = (Screen - Pan) / Zoom
        const rect = containerRef.current?.getBoundingClientRect();
        if (rect) {
          const x = (e.clientX - rect.left - pan.x) / zoom;
          const y = (e.clientY - rect.top - pan.y) / zoom;
          position = { x: Math.round(x), y: Math.round(y) };
        }
      }

      if (data.type === 'existing-node' && data.id) {
        // Update layout metadata for existing node
        if (position) {
          const newData = JSON.parse(JSON.stringify(archData));
          if (!newData.metadata) newData.metadata = {};
          if (!newData.metadata.layout) newData.metadata.layout = {};

          newData.metadata.layout[data.id] = position;
          onPropertiesUpdate(newData);
        }
      } else if (data.type === 'new-node' && data.nodeType) {
        // Add new node from palette
        if (onAddNode) {
          onAddNode(data.nodeType);
          // Position will be handled by the viewer's addNode method
        }
      }
    } catch (err) {
      console.error('Failed to handle drop:', err);
    }
  };

  return (
    <div className="flex flex-1 flex-col bg-[var(--color-background)] min-w-0 w-full h-full overflow-hidden">
      <div className="flex flex-1 relative overflow-hidden min-h-0 w-full h-full" style={{ minHeight: '400px', width: '100%', height: '100%' }}>
        <div className="flex-1 relative min-h-0 w-full h-full" style={{ minHeight: '400px', width: '100%', height: '100%', position: 'relative' }}>
          <div
            id="viewer"
            ref={containerRef}
            className="w-full h-full absolute inset-0 select-none z-[1]"
            draggable={false}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
            style={{
              backgroundColor: 'var(--color-background)',
              backgroundImage:
                'linear-gradient(to right, rgba(148, 163, 184, 0.1) 1px, transparent 1px), linear-gradient(to bottom, rgba(148, 163, 184, 0.1) 1px, transparent 1px)',
              backgroundSize: '24px 24px',
              backgroundPosition: '0 0',
              opacity: '1',
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              width: '100%',
              height: '100%',
              margin: 0,
              padding: 0,
            }}
          />
        </div>
        {showProperties && (
          <div className="w-80 border-l border-[var(--color-border)] bg-[var(--color-background)] flex flex-col z-10 transition-all duration-300 ease-in-out">
            <div className="px-4 py-2.5 border-b border-[var(--color-border)] bg-[var(--color-surface)] flex justify-between items-center flex-shrink-0">
              <h3 className="m-0 text-xs font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider">Properties</h3>
              <button
                onClick={onCloseProperties}
                className="bg-transparent border-none cursor-pointer p-1 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] flex items-center transition-colors"
                title="Close Properties"
              >
                <X size={16} />
              </button>
            </div>
            <PropertiesPanel
              selectedNodeId={selectedNodeId}
              archData={archData}
              onUpdate={onPropertiesUpdate}
            />
          </div>
        )}
      </div>
    </div>
  );
}
