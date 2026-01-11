import { useState, useMemo } from "react";
import { ActionIcon, Group, Text, Stack, Alert, LoadingOverlay, Code } from "@mantine/core";
import { Eye, Copy, Check } from "lucide-react";
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
            <Group gap="xs">
              {showViewToggle && (
                <Group gap="xs" className="markdown-view-toggle">
                  <ActionIcon
                    variant={viewMode === "preview" ? "filled" : "subtle"}
                    onClick={() => setViewMode("preview")}
                    title="Preview mode"
                    aria-label="Preview mode"
                  >
                    <Eye size={14} />
                  </ActionIcon>
                  <ActionIcon
                    variant={viewMode === "raw" ? "filled" : "subtle"}
                    onClick={() => setViewMode("raw")}
                    title="Raw markdown"
                    aria-label="Raw markdown"
                  >
                    <Code />
                  </ActionIcon>
                </Group>
              )}
              {showCopyButton && (
                <ActionIcon
                  variant="default"
                  onClick={handleCopy}
                  title="Copy to clipboard"
                  aria-label="Copy to clipboard"
                  color={copied ? "green" : "blue"}
                >
                  {copied ? <Check size={14} /> : <Copy size={14} />}
                </ActionIcon>
              )}
            </Group>
          )}
        </div>
      )}

      <div className="markdown-preview-panel-content">
        <LoadingOverlay visible={isLoading} />

        {error && (
          <Alert color="red" className="markdown-error">
            {error}
          </Alert>
        )}

        {!isLoading && !error && hasContent && (
          <>
            {viewMode === "preview" ? (
              <div className={`markdown-preview-container ${previewClassName}`}>
                <MarkdownPreview content={markdownSource} onMermaidExpand={onMermaidExpand} />
              </div>
            ) : (
              <Stack className="markdown-raw" p="md">
                <Text component="pre" size="sm" style={{ whiteSpace: "pre-wrap" }}>
                  {markdownSource}
                </Text>
              </Stack>
            )}
          </>
        )}

        {!isLoading && !error && !hasContent && (
          <Stack p="xl" align="center" className="markdown-empty">
            <Text c="dimmed" size="sm">
              {emptyMessage}
            </Text>
          </Stack>
        )}
      </div>
    </div>
  );
}
