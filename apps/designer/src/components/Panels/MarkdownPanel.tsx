// components/Panels/MarkdownPanel.tsx
// Markdown Panel - Shows the Markdown representation of the current architecture
// Uses shared MarkdownPreviewPanel component from @sruja/ui
import { MarkdownPreviewPanel } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";

export function MarkdownPanel() {
  const likec4Model = useArchitectureStore((s) => s.likec4Model);
  const convertedMarkdown = useArchitectureStore((s) => s.convertedMarkdown);
  const isConverting = useArchitectureStore((s) => s.isConverting);

  if (!likec4Model) {
    return (
      <div className="markdown-panel empty">
        <p>No architecture loaded</p>
      </div>
    );
  }

  return (
    <MarkdownPreviewPanel
      content={convertedMarkdown || ""}
      title="Markdown Representation"
      isLoading={isConverting}
      loadingMessage="Converting DSL to Markdown..."
      emptyMessage="Markdown representation not available"
      className="markdown-panel"
      defaultViewMode="preview"
    />
  );
}
