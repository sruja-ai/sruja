// components/Panels/JSONPanel.tsx
// JSON Panel - Shows the JSON representation of the current architecture
import { useState, useEffect, useCallback } from "react";
import { FileJson, Copy, Check, Save, ChevronDown, ChevronUp } from "lucide-react";
import { useArchitectureStore } from "../../stores";
import { MonacoEditor, useTheme, Button } from "@sruja/ui";
// import { convertJsonToDsl } from "../../utils/jsonToDsl"; // Removed
import { convertModelToDsl } from "../../utils/modelToDsl";
import "./JSONPanel.css";

export function JSONPanel() {
  const data = useArchitectureStore((state) => state.likec4Model);
  // convertedJson removed
  // convertedJson removed
  const isConverting = useArchitectureStore((s) => s.isConverting);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const setDslSource = useArchitectureStore((s) => s.setDslSource);
  const { mode } = useTheme();
  const [copied, setCopied] = useState(false);
  const [monacoTheme, setMonacoTheme] = useState<"vs" | "vs-dark">("vs");
  // Initialize JSON source from current model data
  const [jsonSource, setJsonSource] = useState<string>(() => {
    const currentData = useArchitectureStore.getState().likec4Model;
    if (!currentData) return "";
    try {
      return JSON.stringify(currentData, null, 2);
    } catch {
      return "";
    }
  });
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [collapsed, setCollapsed] = useState<boolean>(() => {
    try {
      const saved = window.localStorage.getItem("playground:jsonPanelCollapsed");
      return saved === "true";
    } catch {
      return false;
    }
  });

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

  // Sync JSON source when model data changes
  useEffect(() => {
    if (!data) {
      setJsonSource("");
      return;
    }
    
    try {
      const jsonString = JSON.stringify(data, null, 2);
      // Only update if different to avoid unnecessary re-renders
      if (jsonString !== jsonSource) {
        setJsonSource(jsonString);
      }
    } catch (err) {
      console.error("Failed to stringify model to JSON:", err);
      setJsonSource("");
    }
  }, [data]); // Only depend on data, not jsonSource to avoid circular updates

  const handleCopy = async () => {
    if (!jsonSource) return;

    try {
      await navigator.clipboard.writeText(jsonSource);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  const handleJsonChange = useCallback((newJson: string) => {
    setJsonSource(newJson);
    setError(null);
  }, []);

  const handleSave = useCallback(async () => {
    if (!jsonSource || !jsonSource.trim()) {
      setError("JSON cannot be empty");
      return;
    }

    setIsSaving(true);
    setError(null);

    try {
      const parsed = JSON.parse(jsonSource);

      // Update architecture with parsed JSON
      await updateArchitecture(() => parsed);

      // Convert to DSL and update DSL source
      try {
        const newDsl = await convertModelToDsl(parsed);
        await setDslSource(newDsl, null);
      } catch (dslErr) {
        console.warn("Failed to convert JSON to DSL:", dslErr);
        // Don't fail the save if DSL conversion fails
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : "Invalid JSON";
      setError(`Failed to save JSON: ${message} `);
      console.error("Failed to save JSON:", err);
    } finally {
      setIsSaving(false);
    }
  }, [jsonSource, updateArchitecture, setDslSource]);

  if (!data) {
    return (
      <div className="json-panel empty">
        <p>No architecture loaded</p>
      </div>
    );
  }

  return (
    <div className="json-panel">
      <div className="json-panel-header">
        <div className="json-panel-title">
          <FileJson size={18} />
          <span>JSON Representation</span>
        </div>
        <div className="json-panel-actions">
          <Button
            variant="ghost"
            size="sm"
            className="json-collapse-btn"
            onClick={() => {
              setCollapsed((v) => {
                const next = !v;
                try {
                  window.localStorage.setItem(
                    "playground:jsonPanelCollapsed",
                    next ? "true" : "false"
                  );
                } catch { }
                return next;
              });
            }}
            title={collapsed ? "Expand" : "Collapse"}
          >
            {collapsed ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
            <span>{collapsed ? "Expand" : "Collapse"}</span>
          </Button>
          {jsonSource && !collapsed && (
            <>
              <Button
                variant="primary"
                size="sm"
                onClick={handleSave}
                disabled={isSaving}
                title="Save JSON changes"
              >
                <Save size={14} />
                {isSaving ? "Saving..." : "Save"}
              </Button>
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
            </>
          )}
        </div>
      </div>

      {!collapsed && (
        <div className="json-panel-content">
          {isConverting && <div className="json-loading">Converting DSL to JSON...</div>}

          {error && <div className="json-error">{error}</div>}

          {!isConverting && (
            <MonacoEditor
              value={jsonSource || ""}
              onChange={handleJsonChange}
              language="json"
              theme={monacoTheme}
              height="100%"
              options={{
                readOnly: false,
                lineNumbers: "on",
                glyphMargin: true,
                guides: {
                  indentation: true,
                },
                folding: true,
                foldingStrategy: "indentation",
                showFoldingControls: "always",
                formatOnPaste: true,
                formatOnType: true,
              }}
            />
          )}
        </div>
      )}
    </div>
  );
}
