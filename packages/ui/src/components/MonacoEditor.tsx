import { useEffect, useRef } from "react";
import type * as monacoTypes from "monaco-editor";
import { registerSrujaLanguage } from "./sruja-language";

export type MonacoEditorProps = {
  value: string;
  onChange: (value: string) => void;
  language?: string;
  theme?: "vs" | "vs-dark" | "hc-black";
  height?: number | string;
  options?: Partial<monacoTypes.editor.IStandaloneEditorConstructionOptions>;
  className?: string;
  onReady?: (
    monaco: typeof import("monaco-editor"),
    editor: monacoTypes.editor.IStandaloneCodeEditor
  ) => void;
};

export function MonacoEditor({
  value,
  onChange,
  language = "plaintext",
  theme = "vs",
  height = "100%",
  options = {},
  className,
  onReady,
}: MonacoEditorProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const editorRef = useRef<monacoTypes.editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<typeof import("monaco-editor") | null>(null);

  useEffect(() => {
    if (typeof window === "undefined") return;
    if (editorRef.current) return;

    let isMounted = true;
    let editor: monacoTypes.editor.IStandaloneCodeEditor | null = null;
    let subscription: monacoTypes.IDisposable | null = null;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    // Configure MonacoEnvironment for web workers (Vite-compatible)
    // We intentionally use main thread to avoid worker configuration complexity
    // This must be set BEFORE importing monaco-editor
    if (
      typeof (window as unknown as { MonacoEnvironment: unknown }).MonacoEnvironment === "undefined"
    ) {
      (window as unknown as { MonacoEnvironment: Record<string, unknown> }).MonacoEnvironment = {
        getWorker: function (_workerId: string, _label: string) {
          // Use a dummy worker that does nothing to satisfy Monaco's requirement
          // while effectively running on the main thread for logic
          const blob = new Blob(["self.onmessage = () => {}"], { type: "application/javascript" });
          return new Worker(URL.createObjectURL(blob));
        },
      };
    }

    // Wait for container to be available in DOM before importing Monaco
    const checkAndInit = () => {
      if (!isMounted) return;

      const container = containerRef.current;
      if (!container) {
        timeoutId = setTimeout(checkAndInit, 50);
        return;
      }

      // Ensure container is in DOM - check multiple ways
      try {
        if (!container.parentNode || !container.isConnected || !document.body.contains(container)) {
          timeoutId = setTimeout(checkAndInit, 50);
          return;
        }
      } catch (e) {
        // Container might be in shadow DOM or detached
        timeoutId = setTimeout(checkAndInit, 50);
        return;
      }

      // Now that container is confirmed ready, import Monaco
      import("monaco-editor")
        .then((monaco) => {
          if (!isMounted) return;

          // Final check - container must still exist and be in DOM
          const currentContainer = containerRef.current;
          try {
            if (
              !currentContainer ||
              !currentContainer.parentNode ||
              !currentContainer.isConnected ||
              !document.body.contains(currentContainer)
            ) {
              return;
            }
          } catch (e) {
            return;
          }

          monacoRef.current = monaco;

          // Register Sruja language with syntax highlighting; rely on built-ins for others
          try {
            if (language === "sruja") {
              registerSrujaLanguage(monaco);
            }
          } catch (err) {
            console.warn("Failed to register language:", err);
            return;
          }

          // One more check right before creating editor
          if (!containerRef.current) return;
          try {
            if (
              !containerRef.current.parentNode ||
              !containerRef.current.isConnected ||
              !document.body.contains(containerRef.current)
            ) {
              return;
            }
          } catch (e) {
            return;
          }

          try {
            // Build editor options with folding explicitly enabled
            // Use 'indentation' strategy which works reliably for JSON and all languages
            const editorOptions = {
              value,
              language,
              automaticLayout: true,
              minimap: { enabled: false },
              theme,
              scrollBeyondLastLine: false,
              // Explicitly enable folding with indentation strategy (works for JSON)
              foldingHighlight: true,
              foldingImportsByDefault: false,
              unfoldOnClickAfterEndOfLine: true,
              // Accessibility improvements
              accessibilitySupport: "on" as const,
              // Apply user-provided options (folding settings above will override if needed)
              ...options,
              // Force folding to be enabled (unless explicitly disabled)
              folding: options.folding === false ? false : true,
              // Use indentation strategy for reliable folding
              foldingStrategy: options.foldingStrategy || "indentation",
              showFoldingControls: options.showFoldingControls || "always",
            };

            editor = monaco.editor.create(containerRef.current, editorOptions);

            // Ensure folding is properly initialized after editor creation
            if (editor) {
              // Force layout update to ensure folding controls are rendered
              setTimeout(() => {
                try {
                  editor?.layout();
                } catch (err) {
                  // Ignore layout errors
                }
              }, 100);
            }

            // Fix aria-hidden issue with find widget
            // Monaco's find widget sometimes sets aria-hidden on focused elements
            // We'll observe and fix this accessibility issue
            const fixAriaHidden = () => {
              const findWidget = containerRef.current?.querySelector(".editor-widget.find-widget");
              if (findWidget) {
                const focusedElement = findWidget.querySelector(":focus");
                if (focusedElement && findWidget.getAttribute("aria-hidden") === "true") {
                  // Remove aria-hidden when widget contains focused element
                  findWidget.removeAttribute("aria-hidden");
                }
              }
            };

            // Observe for focus changes in the editor container
            const observer = new MutationObserver(() => {
              fixAriaHidden();
            });

            if (containerRef.current) {
              observer.observe(containerRef.current, {
                attributes: true,
                attributeFilter: ["aria-hidden"],
                subtree: true,
              });

              // Also listen for focus events
              containerRef.current.addEventListener("focusin", fixAriaHidden, true);
              containerRef.current.addEventListener("focus", fixAriaHidden, true);
            }

            // Store observer for cleanup
            (editor as unknown as { _ariaObserver: MutationObserver })._ariaObserver = observer;
            (editor as unknown as { _ariaCleanup: () => void })._ariaCleanup = () => {
              observer.disconnect();
              if (containerRef.current) {
                containerRef.current.removeEventListener("focusin", fixAriaHidden, true);
                containerRef.current.removeEventListener("focus", fixAriaHidden, true);
              }
            };
            editorRef.current = editor;
            subscription = editor.onDidChangeModelContent(() => {
              if (editor && isMounted) {
                onChange(editor.getValue());
              }
            });
            if (onReady && isMounted) onReady(monaco, editor);
          } catch (_err) {
            console.error("Failed to create Monaco editor:", _err);
          }
        })
        .catch((_err) => {
          console.error("Failed to load Monaco editor:", _err);
        });
    };

    // Start checking after a small delay to ensure DOM is ready
    timeoutId = setTimeout(checkAndInit, 0);

    return () => {
      isMounted = false;
      if (timeoutId) {
        clearTimeout(timeoutId);
        timeoutId = null;
      }
      if (subscription) {
        subscription.dispose();
        subscription = null;
      }
      if (editor) {
        try {
          editor.dispose();
        } catch (_err) {
          // Ignore disposal errors
        }
        editor = null;
      }
      if (editorRef.current) {
        try {
          // Clean up aria observer if it exists
          const cleanup = (editorRef.current as unknown as { _ariaCleanup?: () => void })
            ?._ariaCleanup;
          if (cleanup) {
            cleanup();
          }
          editorRef.current.dispose();
        } catch (_err) {
          // Ignore disposal errors
        }
        editorRef.current = null;
      }
    };
  }, [language, theme, onChange, onReady, options, value]);

  useEffect(() => {
    const editor = editorRef.current;
    if (!editor) return;
    if (editor.getValue() !== value) {
      editor.setValue(value);
    }
  }, [value]);

  useEffect(() => {
    const monaco = monacoRef.current;
    const editor = editorRef.current;
    if (!monaco || !editor) return;
    monaco.editor.setTheme(theme);
  }, [theme]);

  return <div ref={containerRef} className={className} style={{ height }} />;
}
