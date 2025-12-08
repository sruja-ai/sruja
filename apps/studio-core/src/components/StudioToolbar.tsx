// apps/studio-core/src/components/StudioToolbar.tsx
import React from 'react';
import { Button, Logo, ThemeToggle } from '@sruja/ui';
import { Copy, ExternalLink, Hammer, Share2, FilePlus } from 'lucide-react';
import type { ExampleKey } from '../examples';
interface StudioToolbarProps {
  selectedExample: ExampleKey;
  onExampleChange: (key: ExampleKey) => void;
  examples: Record<string, string>;
  onShare: () => void;
  onViewInViewer?: () => void;
  onNewDiagram?: () => void;
  isWasmLoading?: boolean;
}

export function StudioToolbar({
  selectedExample,
  onExampleChange,
  examples,
  onShare,
  onViewInViewer,
  onNewDiagram,
  isWasmLoading = false,
}: StudioToolbarProps) {
  return (
    <div className="flex items-center justify-between gap-4 px-4 py-2.5 border-b border-[var(--color-border)] bg-[var(--color-surface)] flex-shrink-0">
      {/* Left side: Logo and Example selector */}
      <div className="flex items-center gap-6">
        {/* Brand Logo */}
        <a
          href="/"
          className="flex items-center gap-2 no-underline hover:opacity-80 transition-opacity"
          title="Sruja Home"
        >
          <Logo size={24} />
          <span className="text-base font-semibold text-[var(--color-text-primary)]">Sruja</span>
          <span className="text-xs text-[var(--color-text-secondary)] font-normal">Studio</span>
        </a>

        {/* Divider */}
        <div className="w-px h-6 bg-[var(--color-border)]" />

        {/* Example selector */}
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-[var(--color-text-secondary)]">Example:</label>
          <select
            value={selectedExample}
            onChange={(e) => onExampleChange(e.target.value as ExampleKey)}
            className="px-3 py-1.5 rounded-md border border-[var(--color-border)] text-sm text-[var(--color-text-primary)] bg-[var(--color-background)] outline-none cursor-pointer hover:bg-[var(--color-surface)] transition-colors"
            disabled={isWasmLoading}
          >
            {Object.keys(examples).map((key) => (
              <option key={key} value={key}>{key}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Right side: Action buttons */}
      <div className="flex items-center gap-2">
        {onNewDiagram && (
          <Button
            variant="secondary"
            size="sm"
            onClick={onNewDiagram}
            title="Start New Diagram"
            disabled={isWasmLoading}
          >
            <FilePlus size={18} />
            <span>New</span>
          </Button>
        )}
        <a
          href="https://sruja.ai"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2 px-3 py-1.5 rounded-md border border-[var(--color-border)] text-sm text-[var(--color-text-primary)] bg-[var(--color-background)] no-underline hover:bg-[var(--color-surface)] transition-colors"
          title="Open Sruja Website"
        >
          <ExternalLink size={18} />
          <span>Website</span>
        </a>
        {onViewInViewer && (
          <Button
            variant="secondary"
            size="sm"
            onClick={onViewInViewer}
            title="View in Viewer App"
            disabled={isWasmLoading}
          >
            <ExternalLink size={18} />
            <span>View in Viewer</span>
          </Button>
        )}
        <Button
          variant="primary"
          size="sm"
          onClick={onShare}
          title="Copy link to current state"
          disabled={isWasmLoading}
        >
          <Share2 size={18} />
          <span>Share</span>
        </Button>
        <div className="w-px h-6 bg-[var(--color-border)] mx-1" />
        <ThemeToggle size="sm" />
      </div>
    </div>
  );
}

