//
import { MonacoEditor, type MonacoEditorProps } from "./MonacoEditor";
import { MonacoDiffEditor } from "./MonacoDiffEditor";
import { logger } from "@sruja/shared";
// LSP code is browser-only - import dynamically to avoid SSR issues

export type SrujaMonacoEditorProps = Omit<MonacoEditorProps, "language"> & {
  enableLsp?: boolean;
  originalValue?: string; // If present, renders DiffEditor
};

export function SrujaMonacoEditor({
  value,
  originalValue,
  onChange,
  theme = "vs",
  height = "100%",
  options = {},
  className,
  enableLsp = true,
  onReady,
}: SrujaMonacoEditorProps) {
  if (originalValue !== undefined) {
    return (
      <MonacoDiffEditor
        original={originalValue}
        modified={value}
        language="sruja"
        theme={theme}
        height={height}
        options={{
          ...options,
          readOnly: false,
          originalEditable: false,
        }}
        className={className}
        onReady={(monaco, editor) => {
          // Pass modified editor to onReady for backward compatibility
          if (onReady)
            onReady(
              monaco,
              editor.getModifiedEditor() as unknown as import("monaco-editor").editor.IStandaloneCodeEditor
            );
        }}
      />
    );
  }

  return (
    <MonacoEditor
      value={value}
      onChange={onChange}
      language="sruja"
      theme={theme}
      height={height}
      options={options}
      className={className}
      onReady={async (monaco, editor) => {
        if (enableLsp && typeof window !== "undefined") {
          try {
            // Use WASM-based LSP shim instead of worker-based LSP
            const { createWasmLspApi, initializeMonacoWasmLsp } = await import("@sruja/shared/lsp");
            const wasmApi = createWasmLspApi();
            initializeMonacoWasmLsp(monaco, editor, wasmApi);
          } catch (err) {
            const errorMessage = err instanceof Error ? err.message : String(err);
            logger.warn("WASM LSP initialization failed", {
              component: "SrujaMonacoEditor",
              feature: "lsp",
              error: errorMessage,
              fallbackMode: true,
            });
          }
        }
        if (onReady) onReady(monaco, editor);
      }}
    />
  );
}
