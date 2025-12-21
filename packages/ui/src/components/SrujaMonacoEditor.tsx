//
import { MonacoEditor, type MonacoEditorProps } from './MonacoEditor'
// LSP code is browser-only - import dynamically to avoid SSR issues

export type SrujaMonacoEditorProps = Omit<MonacoEditorProps, 'language'> & {
  enableLsp?: boolean
}

export function SrujaMonacoEditor({ value, onChange, theme = 'vs', height = '100%', options = {}, className, enableLsp = true, onReady }: SrujaMonacoEditorProps) {
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
        if (enableLsp && typeof window !== 'undefined') {
          try {
            // Use WASM-based LSP shim instead of worker-based LSP
            const { createWasmLspApi, initializeMonacoWasmLsp } = await import('@sruja/shared/lsp')
            const wasmApi = createWasmLspApi()
            initializeMonacoWasmLsp(monaco, editor, wasmApi)
          } catch (err) {
            console.warn('Failed to initialize WASM LSP (editor will work without LSP features):', err)
          }
        }
        if (onReady) onReady(monaco, editor)
      }}
    />
  )
}
