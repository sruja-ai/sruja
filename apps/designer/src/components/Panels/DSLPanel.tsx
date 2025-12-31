// DSL Panel - Shows the source DSL code for the current architecture
import { useState, useEffect, useCallback } from "react";
import { FileCode, Copy, Check, AlertCircle } from "lucide-react";
import { useArchitectureStore, useSelectionStore, useUIStore } from "../../stores";
import { SrujaMonacoEditor, useTheme, Button, SrujaLoader } from "@sruja/ui";
import { convertDslToModel } from "../../wasm";
import type { SrujaModelDump } from "@sruja/shared";
import type * as monacoTypes from "monaco-editor";
import "./DSLPanel.css";

export function DSLPanel() {
  const model = useArchitectureStore((s) => s.model);
  const storeDslSource = useArchitectureStore((s) => s.dslSource);
  const setDslSourceStore = useArchitectureStore((s) => s.setDslSource);
  const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);
  const { mode } = useTheme();
  // Initialize local state from store DSL source
  const [dslSource, setDslSourceLocal] = useState<string>(() => {
    return useArchitectureStore.getState().dslSource || "";
  });
  const [loading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [monacoTheme, setMonacoTheme] = useState<"vs" | "vs-dark">("vs");

  // Detect theme for Monaco Editor
  useEffect(() => {
    const updateTheme = () => {
      const isDark =
        mode === "dark" ||
        (mode === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches) ||
        document.documentElement.getAttribute("data-theme") === "dark";
      setMonacoTheme(isDark ? "vs-dark" : "vs");
    };
    updateTheme();
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaQuery.addEventListener("change", updateTheme);
    return () => mediaQuery.removeEventListener("change", updateTheme);
  }, [mode]);

  // Sync local state with store DSL source - always sync when store changes or component mounts
  // This handles: Builder changes → Model → DSL → DSL Panel updates
  useEffect(() => {
    // Always sync from store when it changes, regardless of current local state
    // This ensures that when component remounts (e.g., after tab switch), it gets the latest value
    // Also handles Builder → DSL sync: when Builder updates model, updateArchitecture converts to DSL
    // and updates store, which triggers this effect to update the editor
    if (storeDslSource !== undefined) {
      if (storeDslSource !== null) {
        // Only update if different to avoid unnecessary re-renders
        // This prevents circular updates when DSL is edited manually
        if (storeDslSource !== dslSource) {
          /*
          // Syncing local DSL from store 
            storeLength: storeDslSource.length, 
            localLength: dslSource.length 
          });
          */
          setDslSourceLocal(storeDslSource);
        }
      } else if (
        storeDslSource === null &&
        dslSource &&
        dslSource.trim() &&
        !dslSource.includes("DSL source not available")
      ) {
        // If store DSL is cleared but we have local DSL, clear local too
        setDslSourceLocal("");
      }
    }
  }, [storeDslSource, dslSource]); // Include dslSource to fix exhaustive-deps

  // Initial load and fallback logic - runs when model or store DSL changes
  useEffect(() => {
    // If we have DSL source in store, use it (handled by sync effect above)
    if (storeDslSource) {
      return;
    }

    // If no model and no DSL, clear
    if (!model && !storeDslSource) {
      setDslSourceLocal("");
      return;
    }

    // If we have a model but no DSL source, provide a fallback message
    if (model && !storeDslSource && !loading) {
      const archName = model._metadata?.name || model.project?.name || "Architecture";
      setDslSourceLocal(
        `// Architecture: ${archName}\n// DSL source not available for this architecture.\n// This architecture may have been loaded from JSON or a custom file.`
      );
    }
  }, [model, storeDslSource, loading]);

  // Debounced DSL update handler
  const handleDSLChange = useCallback(
    (newDsl: string) => {
      setDslSourceLocal(newDsl);
      // Update store DSL source immediately (for share functionality)
      setDslSourceStore(newDsl, null);
    },
    [setDslSourceStore]
  );

  // Save DSL changes (debounced conversion)
  // This handles: DSL edits → Model → Diagram updates
  // Only convert if DSL was manually edited (not synced from store)
  useEffect(() => {
    // Skip if DSL is empty, same as store (synced from Builder), or just whitespace
    if (!dslSource || dslSource === storeDslSource || !dslSource.trim()) return;

    // This means DSL was manually edited - convert to model
    // This will update the model, which will trigger Diagram to update
    const timeoutId = setTimeout(async () => {
      setIsSaving(true);
      setError(null);
      try {
        // console.log("[DSLPanel] Converting DSL to model (DSL → Model → Diagram sync)");
        const json = await convertDslToModel(dslSource);
        if (json) {
          // loadFromDSL updates model, which triggers Diagram to recompute
          loadFromDSL(json as SrujaModelDump, dslSource, "");
        } else {
          setError("Failed to parse DSL");
        }
      } catch (err) {
        // Try to extract line number and error details from error message
        const message = err instanceof Error ? err.message : String(err);

        // Parse error message for line numbers (format: "line 42: syntax error")
        const lineMatch = message.match(/line (\d+)/i);
        const line = lineMatch ? lineMatch[1] : null;

        // Extract specific error (after colon)
        const errorMatch = message.match(/:\s*(.+?)$/);
        const specificError = errorMatch ? errorMatch[1] : message;

        // Build user-friendly error message
        const friendlyError = line
          ? `Syntax error at line ${line}: ${specificError}`
          : `Parse error: ${specificError}`;

        setError(friendlyError);
      } finally {
        setIsSaving(false);
      }
    }, 1500); // 1.5 second debounce

    return () => clearTimeout(timeoutId);
  }, [dslSource, storeDslSource, loadFromDSL]);

  const handleCopy = async () => {
    if (!dslSource) return;

    try {
      await navigator.clipboard.writeText(dslSource);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // console.error("Failed to copy:", err);
    }
  };

  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);

  // Scroll to selected node definition when selectedNodeId changes
  useEffect(() => {
    if (!selectedNodeId || !dslSource) return;

    // Simple regex to match: type "id" || type id
    // We look for common C4 types: person, system, container, component
    // Note: This is a heuristic. A proper LSP would be better, but this works for 90% of cases provided by the generator.
    const types = ["person", "system", "container", "component", "database", "queue"];
    const lines = dslSource.split("\n");

    let targetLine = -1;

    // Strategy 1: Look for explicit definition: type "id" or type id
    // e.g. system "web"
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const match = line.match(
        new RegExp(`\\b(${types.join("|")})\\s+["']?${selectedNodeId}["']?`)
      );
      if (match) {
        targetLine = i + 1;
        break;
      }
    }

    // Strategy 2: Look for nested definition: id "Label"
    // This is harder without a full parser, as we need context. Skipped for now to avoid false positives.

    if (targetLine > 0) {
      // We need access to the editor instance to scroll.
      // Since SrujaMonacoEditor doesn't expose refs easily, we might need to rely on
      // simply passing a 'selection' prop if it supported it, or use a custom event/hack.
      // However, SrujaMonacoEditor likely doesn't expose the instance.
      // Let's assume we can't scroll UNLESS SrujaMonacoEditor accepts an 'onMount' or exposes a ref.
      // If SrujaMonacoEditor is a wrapper around @monaco-editor/react, it might bubble up the ref.
      // Looking at SrujaMonacoEditor usage in other files might help, but for now,
      // since I can't change SrujaMonacoEditor easily, I will implement a text search
      // logic that *tries* to find it.
      // WAIT, SrujaMonacoEditor is likely just a wrapper.
      // Let's check if we can verify SrujaMonacoEditor definition.
      // For this iteration, I'll attempt to use a standard DOM search as a fallback if I can't access the editor?
      // No, Monaco virtualizes.
      // Let's try to add a 'selection' or 'revealLine' functionality if possible,
      // or just log for now if I can't control the editor.
      // Actually, I should check if SrujaMonacoEditor accepts an `editorRef` or similar.
      // But since I cannot check that file easily (it's in @sruja/ui package which might be outside or inside monorepo),
      // I will assume for now that I can't control the scroll without modifying the component.
      // ALTERNATIVE: Just highlight the text? No.
      // Let's try to pass `options` with `line`? No standard monaco prop for that.
      // RE-STRATEGY:
      // If I can't control the scroll, this feature is dead.
      // I'll add a comment that this requires Editor refinement.
      // actually, I can try to use `activeLine` prop if it exists?
      // Let's look at `SrujaMonacoEditor` imports. It is from `@sruja/ui`.
      // I'll assumme it forwards props to `MonacoEditor`.
      // `MonacoEditor` from `@monaco-editor/react` has `onMount`.
    }
  }, [selectedNodeId, dslSource]);

  const [editorInstance, setEditorInstance] =
    useState<monacoTypes.editor.IStandaloneCodeEditor | null>(null);

  const handleEditorDidMount = (
    _monaco: typeof monacoTypes,
    editor: monacoTypes.editor.IStandaloneCodeEditor
  ) => {
    setEditorInstance(editor);
  };

  // Actual scroll effect
  useEffect(() => {
    if (editorInstance && selectedNodeId && dslSource) {
      const types = [
        "person",
        "system",
        "container",
        "component",
        "database",
        "queue",
        "deployment",
        "node",
      ];
      const lines = dslSource.split("\n");
      let targetLine = -1;

      // 1. Exact ID match: type "id"
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        // Match: type "id" or type id (with word boundaries)
        const regex = new RegExp(`\\b(${types.join("|")})\\s+["']?${selectedNodeId}["']?\\b`);
        if (regex.test(line)) {
          targetLine = i + 1;
          break;
        }
      }

      // 2. Fallback: just finding the ID string definition (e.g. nested component)
      if (targetLine === -1) {
        for (let i = 0; i < lines.length; i++) {
          // Look for: id "Label" pattern common in nested elements
          // or just "id"
          // precise match: "id" or id
          const match = new RegExp(`\\b${selectedNodeId}\\b`).test(lines[i]);
          // Verify it looks like a definition (followed by string label or brace)
          if (match && (lines[i].includes('"') || lines[i].includes("{"))) {
            targetLine = i + 1;
            break;
          }
        }
      }

      if (targetLine > 0) {
        editorInstance.revealLineInCenter(targetLine);
        editorInstance.setPosition({ column: 1, lineNumber: targetLine });
        editorInstance.setSelection({
          startLineNumber: targetLine,
          startColumn: 1,
          endLineNumber: targetLine,
          endColumn: 1000,
        });
      }
    }
  }, [editorInstance, selectedNodeId, dslSource]);

  const targetLine = useUIStore((s) => s.targetLine);
  const clearTargetLine = useUIStore((s) => s.setTargetLine);

  // Scroll to target line from global store (e.g. from Governance Dashboard)
  useEffect(() => {
    if (editorInstance && targetLine && targetLine > 0) {
      // Reveal and highlight
      editorInstance.revealLineInCenter(targetLine);
      editorInstance.setPosition({ column: 1, lineNumber: targetLine });
      editorInstance.setSelection({
        startLineNumber: targetLine,
        startColumn: 1,
        endLineNumber: targetLine,
        endColumn: 1000,
      });

      // Clear after revealing so it doesn't keep scrolling back
      // if the user moves away and comes back to the DSL tab
      clearTargetLine(null);
    }
  }, [editorInstance, targetLine, clearTargetLine]);

  // ... render logic

  // Baseline for diff view
  const baselineModel = useArchitectureStore((s) => s.baselineModel);
  const [showDiff, setShowDiff] = useState(false);
  const [baselineDsl, setBaselineDsl] = useState<string | null>(null);

  // Generate baseline DSL when baseline changes or toggle is enabled
  useEffect(() => {
    if (baselineModel && showDiff && !baselineDsl) {
      // Try to convert baseline model to DSL
      // Since convertModelToDsl is async and we need to import it (or if it's already available)
      // Let's import it dynamically if strict separation, but here we can stick to standard import in next step
      // For now, assume convertModelToDsl is available or imported
      import("../../utils/modelToDsl").then(({ convertModelToDsl }) => {
        convertModelToDsl(baselineModel).then((dsl) => setBaselineDsl(dsl));
      });
    }
  }, [baselineModel, showDiff, baselineDsl]);

  if (!model) {
    return (
      <div className="dsl-panel empty">
        <p>No architecture loaded</p>
      </div>
    );
  }

  return (
    <div className="dsl-panel">
      <div className="dsl-panel-header">
        <div className="dsl-panel-title">
          <FileCode size={18} />
          <span>DSL Source</span>
          {/* Sync status indicator */}
          {isSaving && (
            <div
              className="sync-status"
              style={{
                marginLeft: "12px",
                display: "flex",
                alignItems: "center",
                gap: "6px",
                fontSize: "12px",
                color: "var(--text-secondary)",
              }}
            >
              <SrujaLoader size={14} />
              <span>Syncing...</span>
            </div>
          )}
          {!isSaving && !error && dslSource && (
            <div
              className="sync-status"
              style={{
                marginLeft: "12px",
                display: "flex",
                alignItems: "center",
                gap: "4px",
                fontSize: "12px",
                color: "var(--success-color, #10b981)",
              }}
            >
              <Check size={14} />
              <span>Synced</span>
            </div>
          )}
          {error && (
            <div
              className="sync-status"
              style={{
                marginLeft: "12px",
                display: "flex",
                alignItems: "center",
                gap: "4px",
                fontSize: "12px",
                color: "var(--error-color, #ef4444)",
              }}
            >
              <AlertCircle size={14} />
              <span>Error</span>
            </div>
          )}
          {baselineModel && (
            <div
              className="dsl-diff-toggle"
              style={{ marginLeft: 16, display: "flex", alignItems: "center", gap: 8 }}
            >
              <label
                style={{
                  fontSize: 12,
                  display: "flex",
                  alignItems: "center",
                  gap: 4,
                  cursor: "pointer",
                }}
              >
                <input
                  type="checkbox"
                  checked={showDiff}
                  onChange={(e) => setShowDiff(e.target.checked)}
                />
                Show Diff (vs Baseline)
              </label>
            </div>
          )}
        </div>
        {dslSource && (
          <div className="dsl-panel-actions">
            <Button variant="ghost" size="sm" onClick={handleCopy} title="Copy to clipboard">
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
        )}
      </div>

      <div className="dsl-panel-content">
        {loading && <div className="dsl-loading">Loading DSL source...</div>}

        {isSaving && <div className="dsl-loading">Saving DSL changes...</div>}

        {error && <div className="dsl-error">{error}</div>}

        {!loading && !isSaving && !error && (
          <SrujaMonacoEditor
            value={dslSource || ""}
            originalValue={showDiff && baselineDsl ? baselineDsl : undefined}
            onChange={handleDSLChange}
            onReady={handleEditorDidMount}
            theme={monacoTheme}
            height="100%"
            enableLsp={true}
            options={{ readOnly: false }}
          />
        )}
      </div>
    </div>
  );
}
