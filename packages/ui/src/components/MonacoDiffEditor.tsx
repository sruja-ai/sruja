import { useEffect, useRef } from "react";
import type * as monacoTypes from "monaco-editor";
import { registerSrujaLanguage } from "./sruja-language";

export type MonacoDiffEditorProps = {
  original: string;
  modified: string;
  language?: string;
  theme?: "vs" | "vs-dark" | "hc-black";
  height?: number | string;
  options?: Partial<monacoTypes.editor.IDiffEditorConstructionOptions>;
  className?: string;
  onReady?: (
    monaco: typeof import("monaco-editor"),
    editor: monacoTypes.editor.IStandaloneDiffEditor
  ) => void;
};

export function MonacoDiffEditor({
  original,
  modified,
  language = "plaintext",
  theme = "vs",
  height = "100%",
  options = {},
  className,
  onReady,
}: MonacoDiffEditorProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const editorRef = useRef<monacoTypes.editor.IStandaloneDiffEditor | null>(null);
  const monacoRef = useRef<typeof import("monaco-editor") | null>(null);

  useEffect(() => {
    if (typeof window === "undefined") return;
    if (editorRef.current) return;

    let isMounted = true;
    let editor: monacoTypes.editor.IStandaloneDiffEditor | null = null;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    // Configure MonacoEnvironment for web workers (Vite-compatible)
    if (typeof (window as any).MonacoEnvironment === "undefined") {
      (window as any).MonacoEnvironment = {
        getWorker: function (_workerId: string, _label: string) {
          const blob = new Blob(["self.onmessage = () => {}"], { type: "application/javascript" });
          return new Worker(URL.createObjectURL(blob));
        },
      };
    }

    const checkAndInit = () => {
      if (!isMounted) return;

      const container = containerRef.current;
      if (!container) {
        timeoutId = setTimeout(checkAndInit, 50);
        return;
      }

      // Import Monaco
      import("monaco-editor")
        .then((monaco) => {
          if (!isMounted) return;
          if (!containerRef.current) return;

          monacoRef.current = monaco;

          // Register language
          try {
            if (language === "sruja") {
              registerSrujaLanguage(monaco);
            }
          } catch (err) {
            console.warn("Failed to register language:", err);
          }

          try {
            const originalModel = monaco.editor.createModel(original, language);
            const modifiedModel = monaco.editor.createModel(modified, language);

            const editorOptions = {
              theme,
              automaticLayout: true,
              readOnly: true, // Diff view is typically read-only or at least the original side is
              renderSideBySide: true,
              ...options,
            };

            editor = monaco.editor.createDiffEditor(containerRef.current, editorOptions);
            editor.setModel({
              original: originalModel,
              modified: modifiedModel,
            });

            editorRef.current = editor;
            if (onReady && isMounted) onReady(monaco, editor);
          } catch (err) {
            console.error("Failed to create Monaco diff editor:", err);
          }
        })
        .catch((err) => {
          console.error("Failed to load Monaco editor:", err);
        });
    };

    timeoutId = setTimeout(checkAndInit, 0);

    return () => {
      isMounted = false;
      if (timeoutId) clearTimeout(timeoutId);
      if (editor) {
        editor.dispose();
        editor = null;
      }
      editorRef.current = null;
    };
  }, [language, theme]);

  // Update models when props change
  useEffect(() => {
    const editor = editorRef.current;
    const monaco = monacoRef.current;
    if (!editor || !monaco) return;

    const model = editor.getModel();
    if (model) {
      if (model.original.getValue() !== original) {
        model.original.setValue(original);
      }
      if (model.modified.getValue() !== modified) {
        model.modified.setValue(modified);
      }
    }
  }, [original, modified]);

  useEffect(() => {
    const monaco = monacoRef.current;
    if (!monaco) return;
    monaco.editor.setTheme(theme);
  }, [theme]);

  return <div ref={containerRef} className={className} style={{ height }} />;
}
