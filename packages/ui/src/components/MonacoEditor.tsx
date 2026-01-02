import { useEffect, useRef, useState } from "react";
import type * as monacoTypes from "monaco-editor";
import { logger } from "@sruja/shared";
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
  const lastValueRef = useRef<string | null>(null);
  const isInitializingRef = useRef<boolean>(true);
  const valueRef = useRef<string>(value);
  // Track when editor is ready to prevent flash during async Monaco load
  const [isEditorReady, setIsEditorReady] = useState(false);

  // Keep valueRef in sync with value prop
  useEffect(() => {
    valueRef.current = value;
  }, [value]);

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
            const errorMessage = err instanceof Error ? err.message : String(err);
            logger.warn("Failed to register language", {
              component: "MonacoEditor",
              action: "registerLanguage",
              language,
              error: errorMessage,
            });
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
              value: valueRef.current,
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
            const initialEditorValue = editor.getValue();
            lastValueRef.current = initialEditorValue;
            isInitializingRef.current = false;

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
                const newValue = editor.getValue();
                // Only fire onChange if content differs from prop value
                // This prevents sync loops when we programmatically set the value
                // (if editor content matches prop, it was a programmatic update, not user edit)
                if (newValue === valueRef.current) {
                  return;
                }
                lastValueRef.current = newValue;
                onChange(newValue);
              }
            });
            if (onReady && isMounted) onReady(monaco, editor);
            // Mark editor as ready to hide the placeholder
            if (isMounted) setIsEditorReady(true);
          } catch (_err) {
            const errorMessage = _err instanceof Error ? _err.message : String(_err);
            logger.error("Failed to create Monaco editor", {
              component: "MonacoEditor",
              action: "createEditor",
              error:
                _err instanceof Error
                  ? {
                      message: _err.message,
                      name: _err.name,
                      stack: _err.stack,
                    }
                  : errorMessage,
            });
          }
        })
        .catch((_err) => {
          const errorMessage = _err instanceof Error ? _err.message : String(_err);
          logger.error("Failed to load Monaco editor", {
            component: "MonacoEditor",
            action: "loadMonaco",
            error:
              _err instanceof Error
                ? {
                    message: _err.message,
                    name: _err.name,
                    stack: _err.stack,
                  }
                : errorMessage,
          });
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
        isInitializingRef.current = true;
      }
    };
  }, [language, theme, onChange, onReady, options]);

  useEffect(() => {
    const editor = editorRef.current;
    if (!editor) return;

    // Skip updates during initialization to prevent flickering
    if (isInitializingRef.current) {
      return;
    }

    // Skip if we've already synced this exact value
    if (lastValueRef.current === value) {
      return;
    }

    const currentValue = editor.getValue();
    // Only update if editor content differs from prop value
    if (currentValue !== value) {
      lastValueRef.current = value;
      editor.setValue(value);
      // Note: onDidChangeModelContent will fire but won't call onChange
      // because the new value === valueRef.current (the prop we just synced from)
    } else {
      // Editor already has correct value, just update ref
      lastValueRef.current = value;
    }
  }, [value]);

  useEffect(() => {
    const monaco = monacoRef.current;
    const editor = editorRef.current;
    if (!monaco || !editor) return;
    monaco.editor.setTheme(theme);
  }, [theme]);

  // Determine background and text colors based on theme
  const isDarkTheme = theme === "vs-dark" || theme === "hc-black";
  const placeholderStyle: React.CSSProperties = {
    position: "absolute",
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    overflow: "auto",
    padding: "0",
    margin: 0,
    fontFamily: "Menlo, Monaco, 'Courier New', monospace",
    fontSize: "13px",
    lineHeight: "18px",
    whiteSpace: "pre",
    backgroundColor: isDarkTheme ? "#1e1e1e" : "#ffffff",
    color: isDarkTheme ? "#d4d4d4" : "#000000",
    // Smooth transition for when editor takes over
    opacity: isEditorReady ? 0 : 1,
    pointerEvents: isEditorReady ? "none" : "auto",
    transition: "opacity 0.1s ease-out",
    zIndex: isEditorReady ? -1 : 1,
  };

  return (
    <div style={{ position: "relative", height }} className={className}>
      {/* Placeholder shown while Monaco loads - prevents flash */}
      {!isEditorReady && <pre style={placeholderStyle}>{value}</pre>}
      {/* Monaco editor container */}
      <div ref={containerRef} style={{ height: "100%", width: "100%" }} />
    </div>
  );
}
