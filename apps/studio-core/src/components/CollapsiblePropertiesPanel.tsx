// apps/studio-core/src/components/CollapsiblePropertiesPanel.tsx
import React, { useState } from 'react';
import { ChevronLeft, ChevronRight, Settings } from 'lucide-react';
import { PropertiesPanel } from './PropertiesPanel';
import { ArchitectureJSON } from '@sruja/viewer';

interface CollapsiblePropertiesPanelProps {
  selectedNodeId: string | null;
  archData: ArchitectureJSON | null;
  onUpdate: (newData: ArchitectureJSON) => void;
  defaultWidth?: number;
  minWidth?: number;
  maxWidth?: number;
}

export const CollapsiblePropertiesPanel: React.FC<CollapsiblePropertiesPanelProps> = ({
  selectedNodeId,
  archData,
  onUpdate,
  defaultWidth = 320,
  minWidth = 48,
  maxWidth = 480,
}) => {
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [isHidden, setIsHidden] = useState(!selectedNodeId);
  const [width, setWidth] = useState(defaultWidth);
  const [isResizing, setIsResizing] = useState(false);

  // Show panel when node is selected
  React.useEffect(() => {
    if (selectedNodeId) {
      if (isHidden) {
        setIsHidden(false);
      }
      if (isCollapsed) {
        setIsCollapsed(false);
      }
    }
  }, [selectedNodeId, isHidden, isCollapsed]);

  const handleToggle = () => {
    setIsCollapsed(!isCollapsed);
  };

  const handleClose = () => {
    setIsHidden(true);
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  };

  React.useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const containerWidth = window.innerWidth;
      const newWidth = containerWidth - e.clientX;
      if (newWidth >= minWidth && newWidth <= maxWidth) {
        setWidth(newWidth);
      }
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, minWidth, maxWidth]);

  if (isHidden) {
    return (
      <button
        onClick={() => setIsHidden(false)}
        className="w-14 h-14 fixed right-4 bottom-4 rounded-full bg-blue-600 hover:bg-blue-700 text-white shadow-lg hover:shadow-xl flex items-center justify-center transition-all duration-200 z-50 group"
        title="Show Properties"
      >
        <Settings size={20} className="group-hover:rotate-90 transition-transform duration-300" />
      </button>
    );
  }

  return (
    <div
      className="relative bg-gray-900 border-l border-gray-800 flex flex-col transition-all duration-300 ease-in-out flex-shrink-0"
      style={{ width: isCollapsed ? minWidth : width, flexShrink: 0, flexGrow: 0 }}
    >
      {/* Toggle Button */}
      <button
        onClick={handleToggle}
        className="absolute -left-3 top-4 z-10 w-6 h-6 rounded-full bg-gray-800 border border-gray-700 flex items-center justify-center text-gray-400 hover:text-white hover:bg-gray-700 hover:border-gray-600 transition-all duration-200 shadow-lg hover:shadow-xl"
        title={isCollapsed ? 'Expand properties' : 'Collapse properties'}
      >
        {isCollapsed ? <ChevronLeft size={14} /> : <ChevronRight size={14} />}
      </button>

      {/* Content */}
      {!isCollapsed ? (
        <div className="flex-1 flex flex-col overflow-hidden">
          {selectedNodeId ? (
            <PropertiesPanel
              selectedNodeId={selectedNodeId}
              archData={archData}
              onUpdate={onUpdate}
              onClose={handleClose}
            />
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center p-8 text-center">
              <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-800/50 flex items-center justify-center">
                <Settings size={24} className="text-gray-600" />
              </div>
              <p className="text-sm font-medium text-gray-400 mb-1">No selection</p>
              <p className="text-xs text-gray-600">Select a node to edit properties</p>
            </div>
          )}
        </div>
      ) : (
        <div className="flex flex-col items-center py-2">
          <button
            onClick={handleClose}
            className="p-2 rounded text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
            title="Hide Properties"
          >
            <Settings size={20} />
          </button>
        </div>
      )}

      {/* Resize Handle */}
      {!isCollapsed && (
        <div
          onMouseDown={handleMouseDown}
          className="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-600/50 transition-colors"
          style={{ zIndex: 10 }}
        />
      )}
    </div>
  );
};


