// apps/studio-core/src/components/ActivityBar.tsx
import React from 'react';
import { Sidebar, Settings, BookOpen } from 'lucide-react';
import { cn } from '@sruja/ui';
import { useStudioState } from '../context/StudioStateContext';

interface ActivityBarProps {
  onExplorerClick: () => void;
  onPropertiesClick: () => void;
  onDocumentationClick: () => void;
}

export function ActivityBar({
  onExplorerClick,
  onPropertiesClick,
  onDocumentationClick,
}: ActivityBarProps) {
  const { sidebar, properties } = useStudioState();

  return (
    <div className="w-12 h-full bg-[var(--color-neutral-900)] flex flex-col items-center pt-2 border-r border-[var(--color-neutral-800)] flex-shrink-0">
      <button
        onClick={onExplorerClick}
        className={cn(
          "w-10 h-10 mb-1 rounded flex items-center justify-center transition-all duration-200",
          sidebar.showSidebar && sidebar.activePanel === 'explorer'
            ? "bg-[var(--color-info-500)] text-[var(--color-background)]"
            : "bg-transparent text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)]"
        )}
        title="Explorer"
      >
        <Sidebar size={20} />
      </button>
      <button
        onClick={onPropertiesClick}
        className={cn(
          "w-10 h-10 mb-1 rounded flex items-center justify-center transition-all duration-200",
          properties.showProperties
            ? "bg-[var(--color-info-500)] text-[var(--color-background)]"
            : "bg-transparent text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)]"
        )}
        title="Properties"
      >
        <Settings size={20} />
      </button>
      <button
        onClick={onDocumentationClick}
        className={cn(
          "w-10 h-10 mb-1 rounded flex items-center justify-center transition-all duration-200",
          sidebar.showSidebar && sidebar.activePanel === 'documentation'
            ? "bg-[var(--color-info-500)] text-[var(--color-background)]"
            : "bg-transparent text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)]"
        )}
        title="Documentation"
      >
        <BookOpen size={20} />
      </button>
    </div>
  );
}

