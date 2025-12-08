// apps/studio-core/src/components/StudioSidebar.tsx
import React from 'react';
import { X } from 'lucide-react';
import { cn } from '@sruja/ui';
import { ModelExplorer } from './ModelExplorer';
import { DocumentationPanel } from './DocumentationPanel';
import { ShortcutsPanel } from './ShortcutsPanel';
import { GuidePanel } from './GuidePanel';
import { Sidebar } from 'lucide-react';
import { useStudioState } from '../context/StudioStateContext';
import type { ArchitectureJSON } from '@sruja/viewer';

interface StudioSidebarProps {
  archData: ArchitectureJSON | null;
  onExplorerSelect: (id: string) => void;
}

export function StudioSidebar({ archData, onExplorerSelect }: StudioSidebarProps) {
  const { sidebar, setSidebar, documentation } = useStudioState();

  const handleResize = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = sidebar.width;

    const handleMouseMove = (e: MouseEvent) => {
      const diff = e.clientX - startX;
      const newWidth = Math.max(200, Math.min(600, startWidth + diff));
      setSidebar((prev) => ({ ...prev, width: newWidth }));
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  if (!sidebar.showSidebar) return null;

  return (
    <div
      className="h-full bg-[var(--color-background)] border-r border-[var(--color-border)] flex flex-col relative transition-all duration-300 ease-in-out flex-shrink-0"
      style={{ width: `${sidebar.width}px` }}
    >
      {/* Sidebar Header with Tabs */}
      <div className="flex border-b border-[var(--color-border)] bg-[var(--color-surface)]">
        <button
          onClick={() => setSidebar((prev) => ({ ...prev, activePanel: 'explorer' }))}
          className={cn(
            "px-4 py-2 text-xs font-medium transition-colors border-b-2",
            sidebar.activePanel === 'explorer'
              ? "text-[var(--color-text-primary)] border-[var(--color-info-500)] bg-[var(--color-background)]"
              : "text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)]"
          )}
        >
          EXPLORER
        </button>
        <button
          onClick={() => setSidebar((prev) => ({ ...prev, activePanel: 'documentation' }))}
          className={cn(
            "px-4 py-2 text-xs font-medium transition-colors border-b-2",
            sidebar.activePanel === 'documentation'
              ? "text-[var(--color-text-primary)] border-[var(--color-info-500)] bg-[var(--color-background)]"
              : "text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)]"
          )}
        >
          DOCS
        </button>
        <button
          onClick={() => setSidebar((prev) => ({ ...prev, activePanel: 'shortcuts' }))}
          className={cn(
            "px-4 py-2 text-xs font-medium transition-colors border-b-2",
            sidebar.activePanel === 'shortcuts'
              ? "text-[var(--color-text-primary)] border-[var(--color-info-500)] bg-[var(--color-background)]"
              : "text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)]"
          )}
        >
          SHORTCUTS
        </button>
        <button
          onClick={() => setSidebar((prev) => ({ ...prev, activePanel: 'guide' }))}
          className={cn(
            "px-4 py-2 text-xs font-medium transition-colors border-b-2",
            sidebar.activePanel === 'guide'
              ? "text-[var(--color-text-primary)] border-[var(--color-info-500)] bg-[var(--color-background)]"
              : "text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)]"
          )}
        >
          GUIDE
        </button>
        <div className="flex-1" />
        <button
          onClick={() => setSidebar((prev) => ({ ...prev, showSidebar: false }))}
          className="p-2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors"
          title="Close Sidebar"
        >
          <X size={14} />
        </button>
      </div>

      {/* Panel Content */}
      <div className="flex-1 overflow-hidden flex flex-col">
        {sidebar.activePanel === 'explorer' ? (
          <div className="flex-1 overflow-auto">
            {archData ? (
              <ModelExplorer data={archData} onSelect={onExplorerSelect} />
            ) : (
              <div className="p-8 text-center text-[var(--color-text-tertiary)]">
                <Sidebar className="w-12 h-12 mx-auto mb-3 opacity-30" />
                <p className="text-sm font-medium mb-1">No architecture loaded</p>
                <p className="text-xs opacity-70">Parse your DSL to see the model explorer</p>
              </div>
            )}
          </div>
        ) : sidebar.activePanel === 'documentation' ? (
          <DocumentationPanel
            isOpen={true}
            selectedNodeType={documentation.selectedNodeType}
            selectedNodeId={documentation.selectedNodeId}
            selectedNodeLabel={documentation.selectedNodeLabel}
          />
        ) : sidebar.activePanel === 'shortcuts' ? (
          <ShortcutsPanel />
        ) : (
          <GuidePanel archData={archData} />
        )}
      </div>

      {/* Resize handle */}
      <div
        className="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize bg-transparent z-10 hover:bg-blue-200 transition-colors"
        onMouseDown={handleResize}
      />
    </div>
  );
}
