// packages/ui/src/components/MarkdownPreviewPanel.tsx
// Enhanced markdown preview component with preview/raw toggle and copy functionality
// Can be used across all apps

import { useState, useMemo } from "react";
import { Eye, Code, Copy, Check } from "lucide-react";
import { MarkdownPreview } from "./MarkdownPreview";
import type { MarkdownPreviewProps } from "./MarkdownPreview";
import "./MarkdownPreviewPanel.css";

export interface MarkdownPreviewPanelProps extends Omit<MarkdownPreviewProps, "className"> {
  /** Title to display in the header */
  title?: string;
  /** Show preview/raw toggle buttons */
  showViewToggle?: boolean;
  /** Show copy to clipboard button */
  showCopyButton?: boolean;
  /** Custom header content */
  headerContent?: React.ReactNode;
  /** Custom className for the panel container */
  className?: string;
  /** Custom className for the preview container */
  previewClassName?: string;
  /** Loading state */
  isLoading?: boolean;
  /** Loading message */
  loadingMessage?: string;
  /** Empty state message */
  emptyMessage?: string;
  /** Error state */
  error?: string | null;
  /** Initial view mode */
  defaultViewMode?: "preview" | "raw";
  /** Callback when copy is successful */
  onCopy?: () => void;
  /** Callback when mermaid diagram is expanded */
  onMermaidExpand?: (svg: string, code: string) => void;
}

export function MarkdownPreviewPanel({
  content,
  title = "Markdown Preview",
  showViewToggle = true,
  showCopyButton = true,
  headerContent,
  className = "",
  previewClassName = "",
  isLoading = false,
  loadingMessage = "Loading...",
  emptyMessage = "No content available",
  error = null,
  defaultViewMode = "preview",
  onMermaidExpand,
  onCopy,
}: MarkdownPreviewPanelProps) {
  const [viewMode, setViewMode] = useState<"preview" | "raw">(defaultViewMode);
  const [copied, setCopied] = useState(false);

  const markdownSource = useMemo(() => {
    return content || "";
  }, [content]);

  const handleCopy = async () => {
    if (!markdownSource) return;

    try {
      await navigator.clipboard.writeText(markdownSource);
      setCopied(true);
      onCopy?.();
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  const hasContent = markdownSource && markdownSource.trim().length > 0;
  const showActions = (showViewToggle || showCopyButton) && hasContent && !isLoading && !error;

  return (
    <div className={`markdown-preview-panel ${className}`}>
      {(title || headerContent || showActions) && (
        <div className="markdown-preview-panel-header">
          {title && (
            <div className="markdown-preview-panel-title">
              <span>{title}</span>
            </div>
          )}
          {headerContent}
          {showActions && (
            <div className="markdown-preview-panel-actions">
              {showViewToggle && (
                <div className="markdown-view-toggle">
                  <button
                    type="button"
                    className={`view-toggle-btn ${viewMode === "preview" ? "active" : ""}`}
                    onClick={() => setViewMode("preview")}
                    title="Preview mode"
                    aria-label="Preview mode"
                  >
                    <Eye size={14} />
                    <span>Preview</span>
                  </button>
                  <button
                    type="button"
                    className={`view-toggle-btn ${viewMode === "raw" ? "active" : ""}`}
                    onClick={() => setViewMode("raw")}
                    title="Raw markdown"
                    aria-label="Raw markdown"
                  >
                    <Code size={14} />
                    <span>Raw</span>
                  </button>
                </div>
              )}
              {showCopyButton && (
                <button
                  type="button"
                  className="markdown-copy-btn"
                  onClick={handleCopy}
                  title="Copy to clipboard"
                  aria-label="Copy to clipboard"
                >
                  {copied ? (
                    <>
                      <Check size={14} />
                      <span>Copied!</span>
                    </>
                  ) : (
                    <>
                      <Copy size={14} />
                      <span>Copy</span>
                    </>
                  )}
                </button>
              )}
            </div>
          )}
        </div>
      )}

      <div className="markdown-preview-panel-content">
        {isLoading && <div className="markdown-loading">{loadingMessage}</div>}

        {error && <div className="markdown-error">{error}</div>}

        {!isLoading && !error && hasContent && (
          <>
            {viewMode === "preview" ? (
              <div className={`markdown-preview-container ${previewClassName}`}>
                <MarkdownPreview content={markdownSource} onMermaidExpand={onMermaidExpand} />
              </div>
            ) : (
              <pre className="markdown-raw">
                <code>{markdownSource}</code>
              </pre>
            )}
          </>
        )}

        {!isLoading && !error && !hasContent && (
          <div className="markdown-empty">
            <p>{emptyMessage}</p>
          </div>
        )}
      </div>
    </div>
  );
}
