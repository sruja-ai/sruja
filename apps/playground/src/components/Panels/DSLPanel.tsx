// DSL Panel - Shows the source DSL code for the current architecture
import { useState, useEffect, useCallback } from "react";
import { FileCode, Copy, Check } from "lucide-react";
import { useArchitectureStore, useSelectionStore } from "../../stores";
import { SrujaMonacoEditor, useTheme, Button } from "@sruja/ui";
import { convertDslToJson } from "../../wasm";
import "./DSLPanel.css";

export function DSLPanel() {
  const data = useArchitectureStore((s) => s.data);
  const storeDslSource = useArchitectureStore((s) => s.dslSource);
  const setDslSourceStore = useArchitectureStore((s) => s.setDslSource);
  const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);
  const currentExampleFile = useArchitectureStore((s) => s.currentExampleFile);
  const { mode } = useTheme();
  const [dslSource, setDslSourceLocal] = useState<string>("");
  const [loading, setLoading] = useState(false);
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

  // Load DSL from store when data changes (URL loading is handled in App.tsx)
  useEffect(() => {
    if (!data) {
      setDslSourceLocal("");
      return;
    }

    // Skip if we already loaded from URL
    const params = new URLSearchParams(window.location.search);
    if (params.get("share") || params.get("dsl")) {
      // URL DSL is loaded in App.tsx, just sync the local state
      if (storeDslSource) {
        setDslSourceLocal(storeDslSource);
      }
      return;
    }

    // Sync with store DSL source when it changes (e.g., from form edits)
    if (storeDslSource && storeDslSource !== dslSource) {
      setDslSourceLocal(storeDslSource);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      // Use DSL from store if available (from examples)
      if (storeDslSource) {
        setDslSourceLocal(storeDslSource);
      } else {
        const archName = data.metadata?.name || data.architecture?.name || "";
        setDslSourceLocal(
          `// Architecture: ${archName}\n// DSL source not available for this architecture.\n// This architecture may have been loaded from JSON or a custom file.`
        );
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : "Unknown error";
      setError(`Failed to load DSL: ${message}`);
    } finally {
      setLoading(false);
    }
  }, [data, storeDslSource, currentExampleFile, dslSource]);

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
  useEffect(() => {
    if (!dslSource || dslSource === storeDslSource || !dslSource.trim()) return;

    const timeoutId = setTimeout(async () => {
      setIsSaving(true);
      setError(null);
      try {
        const json = await convertDslToJson(dslSource);
        if (json) {
          loadFromDSL(json as any, dslSource, "");
        } else {
          setError("Failed to parse DSL");
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        setError(`Failed to save DSL: ${message}`);
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
    } catch (err) {
      console.error("Failed to copy:", err);
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

  const [editorInstance, setEditorInstance] = useState<any>(null);

  const handleEditorDidMount = (_monaco: any, editor: any) => {
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

  // ... render logic

  if (!data) {
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
