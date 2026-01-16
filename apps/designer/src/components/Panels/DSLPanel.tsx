// DSL Panel - Shows the source DSL code for the current architecture
import { useState, useMemo, useCallback, useEffect, useRef } from "react";
import type * as monacoTypes from "monaco-editor";
import { useArchitectureStore } from "../../stores";
import { useUIStore } from "../../stores/uiStore";
import { SrujaMonacoEditor } from "@sruja/ui";
import { useDSLSync, useDSLEditor, useDSLDiff, useDslSelectionSync } from "../../hooks";
import { DSLPanelHeader } from "./DSLPanelHeader";
import "./DSLPanel.css";

export function DSLPanel() {
  const model = useArchitectureStore((s) => s.model);
  const { dslSource, error, isSaving, handleDSLChange } = useDSLSync();
  const { monacoTheme, handleEditorDidMount: baseEditorDidMount } = useDSLEditor(dslSource);
  const { showDiff, baselineDsl, setShowDiff } = useDSLDiff();
  const [copied, setCopied] = useState(false);
  const [monacoInstance, setMonacoInstance] = useState<typeof import("monaco-editor") | null>(null);
  const [editorInstance, setEditorInstance] =
    useState<monacoTypes.editor.IStandaloneCodeEditor | null>(null);
  const targetLine = useUIStore((s) => s.targetLine);
  const setTargetLine = useUIStore((s) => s.setTargetLine);

  // Memoize value to prevent unnecessary re-renders
  const editorValue = useMemo(() => dslSource || "", [dslSource]);

  const handleEditorDidMount = useCallback(
    (monaco: typeof import("monaco-editor"), editor: monacoTypes.editor.IStandaloneCodeEditor) => {
      setMonacoInstance(monaco);
      setEditorInstance(editor);
      baseEditorDidMount(monaco, editor);
    },
    [baseEditorDidMount]
  );

  useDslSelectionSync({
    editor: editorInstance,
    monaco: monacoInstance,
    dslSource,
    model,
  });

  useEffect(() => {
    if (!targetLine || !editorInstance) return;
    editorInstance.revealLineInCenterIfOutsideViewport(targetLine);
    editorInstance.setPosition({ lineNumber: targetLine, column: 1 });
    setTargetLine(null);
  }, [editorInstance, setTargetLine, targetLine]);

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
        <div className="dsl-empty-content">
          <p>No architecture loaded</p>
          <p className="dsl-empty-hint">
            💡 Go to the <strong>Builder</strong> tab to start creating your architecture, or load
            an example.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="dsl-panel" data-testid="dsl-panel-container">
      <DSLPanelHeader
        dslSource={dslSource}
        error={error}
        isSaving={isSaving}
        copied={copied}
        onCopy={handleCopy}
        showDiff={showDiff}
        onToggleDiff={setShowDiff}
      />

      <div className="dsl-panel-content" data-testid="dsl-editor-content">
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
