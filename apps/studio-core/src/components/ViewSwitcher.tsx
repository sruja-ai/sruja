// apps/studio-core/src/components/ViewSwitcher.tsx
import React from 'react';
import { cn } from '@sruja/ui';

interface ViewSwitcherProps {
  activeView: 'editor' | 'split' | 'viewer';
  onViewChange: (view: 'editor' | 'split' | 'viewer') => void;
}

export function ViewSwitcher({ activeView, onViewChange }: ViewSwitcherProps) {
  return (
    <div className="flex items-center gap-1 border-b border-[var(--color-border)] bg-[var(--color-background)]">
      <button
        onClick={() => onViewChange('editor')}
        className={cn(
          "px-4 py-2 border-none bg-transparent cursor-pointer text-sm font-medium transition-colors",
          activeView === 'editor'
            ? "text-[var(--color-info-500)] border-b-2 border-[var(--color-info-500)]"
            : "text-[var(--color-text-secondary)] border-b-2 border-transparent hover:text-[var(--color-text-primary)]"
        )}
      >
        Editor
      </button>
      <button
        onClick={() => onViewChange('split')}
        className={cn(
          "px-4 py-2 border-none bg-transparent cursor-pointer text-sm font-medium transition-colors",
          activeView === 'split'
            ? "text-[var(--color-info-500)] border-b-2 border-[var(--color-info-500)]"
            : "text-[var(--color-text-secondary)] border-b-2 border-transparent hover:text-[var(--color-text-primary)]"
        )}
      >
        Split View
      </button>
      <button
        onClick={() => onViewChange('viewer')}
        className={cn(
          "px-4 py-2 border-none bg-transparent cursor-pointer text-sm font-medium transition-colors",
          activeView === 'viewer'
            ? "text-[var(--color-info-500)] border-b-2 border-[var(--color-info-500)]"
            : "text-[var(--color-text-secondary)] border-b-2 border-transparent hover:text-[var(--color-text-primary)]"
        )}
      >
        Diagram
      </button>
    </div>
  );
}









