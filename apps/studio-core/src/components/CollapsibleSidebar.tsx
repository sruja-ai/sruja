// apps/studio-core/src/components/CollapsibleSidebar.tsx
import React, { useState } from 'react';
import { ChevronLeft, ChevronRight, Layout, Layers, Package } from 'lucide-react';

interface CollapsibleSidebarProps {
  children: React.ReactNode;
  defaultWidth?: number;
  minWidth?: number;
  maxWidth?: number;
}

export const CollapsibleSidebar: React.FC<CollapsibleSidebarProps> = ({
  children,
  defaultWidth = 240,
  minWidth = 48,
  maxWidth = 320,
}) => {
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [width, setWidth] = useState(defaultWidth);
  const [isResizing, setIsResizing] = useState(false);

  const handleToggle = () => {
    setIsCollapsed(!isCollapsed);
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  };

  React.useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const newWidth = e.clientX;
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

  return (
    <div
      className="relative bg-gray-900 border-r border-gray-800 flex flex-col transition-all duration-300 ease-in-out flex-shrink-0"
      style={{ width: isCollapsed ? minWidth : width, flexShrink: 0, flexGrow: 0 }}
    >
      {/* Toggle Button */}
      <button
        onClick={handleToggle}
        className="absolute -right-3 top-4 z-10 w-6 h-6 rounded-full bg-gray-800 border border-gray-700 flex items-center justify-center text-gray-400 hover:text-white hover:bg-gray-700 hover:border-gray-600 transition-all duration-200 shadow-lg hover:shadow-xl"
        title={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      >
        {isCollapsed ? <ChevronRight size={14} /> : <ChevronLeft size={14} />}
      </button>

      {/* Content */}
      {!isCollapsed && (
        <div className="flex-1 flex flex-col overflow-y-auto min-h-0">
          {children}
        </div>
      )}

      {/* Collapsed State: Icon Bar */}
      {isCollapsed && (
        <div className="flex flex-col items-center py-2 gap-2">
          <button
            className="p-2 rounded text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
            title="Steps"
          >
            <Layout size={20} />
          </button>
          <button
            className="p-2 rounded text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
            title="Palette"
          >
            <Layers size={20} />
          </button>
          <button
            className="p-2 rounded text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
            title="Assets"
          >
            <Package size={20} />
          </button>
        </div>
      )}

      {/* Resize Handle */}
      {!isCollapsed && (
        <div
          onMouseDown={handleMouseDown}
          className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-600/50 transition-colors"
          style={{ zIndex: 10 }}
        />
      )}
    </div>
  );
};



