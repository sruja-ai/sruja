import { BookOpen, ExternalLink, Share2, Check, CheckCircle2, AlertCircle } from 'lucide-react';
import { SrujaLoader } from '@sruja/ui';
import { Logo } from '@sruja/ui';
import type { Example, ValidationStatus } from '../types';

interface TopNavBarProps {
  examples: Example[];
  selectedExample: string;
  isLoadingExample: boolean;
  isWasmLoading: boolean;
  onExampleChange: (exampleFile: string) => void;
  onOpenStudio: () => void;
  onShare: () => void;
  shareCopied: boolean;
  validationStatus: ValidationStatus;
  onErrorClick: () => void;
}

export function TopNavBar({
  examples,
  selectedExample,
  isLoadingExample,
  isWasmLoading,
  onExampleChange,
  onOpenStudio,
  onShare,
  shareCopied,
  validationStatus,
  onErrorClick,
}: TopNavBarProps) {
  return (
    <div className="flex items-center justify-between vscode-toolbar">
      <div className="flex items-center gap-4">
        <a href="/" className="flex items-center gap-2 no-underline" title="Sruja Home">
          <Logo size={28} />
          <span className="font-semibold text-lg" style={{ color: 'var(--color-text-primary)' }}>Sruja</span>
        </a>
        <div className="w-px h-6 bg-border" />
        {/* Examples */}
        {examples.length > 0 && (
          <>
            <div className="flex items-center gap-2">
              <BookOpen className="vscode-icon" />
              <label className="text-sm whitespace-nowrap" style={{ color: 'var(--vscode-descriptionForeground)' }}>Examples:</label>
              <select
                value={selectedExample}
                onChange={(e) => onExampleChange(e.target.value)}
                className="vscode-select min-w-[220px]"
                disabled={isWasmLoading || isLoadingExample}
                title={isLoadingExample ? 'Loading example...' : 'Select an example to load'}
              >
                <option value="">Select an example...</option>
                {examples.map((ex) => (
                  <option key={ex.file} value={ex.file} title={ex.description}>
                    {ex.name} {ex.category ? `(${ex.category})` : ''}
                  </option>
                ))}
              </select>
              {isLoadingExample && (
                <span className="inline-flex items-center gap-1 text-xs" style={{ color: 'var(--vscode-descriptionForeground)' }}>
                  <SrujaLoader size={16} />
                  Loading...
                </span>
              )}
            </div>
            <div className="vscode-toolbar-separator" />
          </>
        )}
        {/* Open in Studio */}
        <button
          className="vscode-toolbar-button"
          onClick={onOpenStudio}
          title="Open current architecture in Studio"
        >
          <ExternalLink className="vscode-icon" />
          <span>Open in Studio</span>
        </button>
        <div className="vscode-toolbar-separator" />
        {/* Share Button */}
        <button
          className="vscode-toolbar-button"
          onClick={onShare}
          title="Copy shareable URL"
        >
          {shareCopied ? (
            <>
              <Check className="vscode-icon" />
              <span>Copied!</span>
            </>
          ) : (
            <>
              <Share2 className="vscode-icon" />
              <span>Share</span>
            </>
          )}
        </button>
        <div className="vscode-toolbar-separator" />
        {/* Website Link */}
        <a 
          href="/" 
          className="vscode-toolbar-button"
          style={{ textDecoration: 'none', color: 'var(--vscode-textLink-foreground)' }}
          title="Go to Sruja website"
        >
          Website
        </a>
      </div>
      {/* Health Status Indicator */}
      <div className="flex items-center gap-2">
        {validationStatus.isValid ? (
          <div className="flex items-center gap-1.5" style={{ color: 'var(--vscode-successForeground)' }} title="Code is valid">
            <CheckCircle2 className="vscode-icon" style={{ width: '18px', height: '18px' }} />
          </div>
        ) : validationStatus.lastError ? (
          <button
            onClick={onErrorClick}
            className="vscode-toolbar-button"
            style={{ color: 'var(--vscode-errorForeground)' }}
            title="Click to view errors"
          >
            <AlertCircle className="vscode-icon" style={{ width: '18px', height: '18px' }} />
          </button>
        ) : null}
      </div>
    </div>
  );
}

