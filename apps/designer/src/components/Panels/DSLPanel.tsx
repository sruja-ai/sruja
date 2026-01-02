// DSL Panel - Shows the source DSL code for the current architecture
import { useState, useMemo } from "react";
import { useArchitectureStore } from "../../stores";
import { SrujaMonacoEditor } from "@sruja/ui";
import { useDSLSync, useDSLEditor, useDSLDiff } from "../../hooks";
import { DSLPanelHeader } from "./DSLPanelHeader";
import "./DSLPanel.css";

export function DSLPanel() {
  const model = useArchitectureStore((s) => s.model);
  const { dslSource, error, isSaving, handleDSLChange } = useDSLSync();
  const { monacoTheme, handleEditorDidMount } = useDSLEditor(dslSource);
  const { showDiff, baselineDsl, setShowDiff } = useDSLDiff();
  const [copied, setCopied] = useState(false);

  // Memoize value to prevent unnecessary re-renders
  const editorValue = useMemo(() => dslSource || "", [dslSource]);

  const handleCopy = async () => {
    if (!dslSource) return;
    try {
      await navigator.clipboard.writeText(dslSource);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Copy failed silently
    }
  };

  if (!model) {
    return (
      <div className="dsl-panel empty">
        <p>No architecture loaded</p>
      </div>
    );
  }

  return (
    <div className="dsl-panel">
      <DSLPanelHeader
        dslSource={dslSource}
        error={error}
        isSaving={isSaving}
        copied={copied}
        onCopy={handleCopy}
        showDiff={showDiff}
        onToggleDiff={setShowDiff}
      />

      <div className="dsl-panel-content">
        {isSaving && <div className="dsl-loading">Saving DSL changes...</div>}
        {error && <div className="dsl-error">{error}</div>}
        {/* Always render editor to prevent unmount/remount flicker */}
        <SrujaMonacoEditor
          value={editorValue}
          originalValue={showDiff && baselineDsl ? baselineDsl : undefined}
          onChange={handleDSLChange}
          onReady={handleEditorDidMount}
          theme={monacoTheme}
          height="100%"
          enableLsp={true}
          options={{ readOnly: false }}
        />
      </div>
    </div>
  );
}
