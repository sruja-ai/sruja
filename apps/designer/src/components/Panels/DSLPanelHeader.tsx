import { Copy, Check, GitCompare, Loader2 } from "lucide-react";
import { Button } from "@sruja/ui";
import "./DSLPanel.css";

interface DSLPanelHeaderProps {
  dslSource: string | null;
  error: string | null;
  isSaving: boolean;
  copied: boolean;
  onCopy: () => void;
  showDiff: boolean;
  onToggleDiff: (show: boolean) => void;
}

export function DSLPanelHeader({
  dslSource,
  error,
  isSaving,
  copied,
  onCopy,
  showDiff,
  onToggleDiff,
}: DSLPanelHeaderProps) {
  return (
    <div className="dsl-panel-header">
      <div className="dsl-panel-title">
        <span>DSL Source</span>
        {isSaving && <Loader2 size={14} className="spinner" style={{ marginLeft: 8 }} />}
        {error && (
          <span style={{ color: "var(--error-color)", fontSize: "12px", marginLeft: 8 }}>
            {error}
          </span>
        )}
      </div>
      <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onToggleDiff(!showDiff)}
          title={showDiff ? "Hide diff" : "Show diff"}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "6px",
          }}
        >
          <GitCompare size={14} />
          <span>{showDiff ? "Hide Diff" : "Show Diff"}</span>
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={onCopy}
          disabled={!dslSource}
          title="Copy to clipboard"
          className="dsl-copy-btn"
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
        </Button>
      </div>
    </div>
  );
}
