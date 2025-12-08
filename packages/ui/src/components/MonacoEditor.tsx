import { useEffect, useRef } from 'react'
import type * as monacoTypes from 'monaco-editor'
import { registerSrujaLanguage } from './sruja-language'

export type MonacoEditorProps = {
  value: string
  onChange: (value: string) => void
  language?: string
  theme?: 'vs' | 'vs-dark' | 'hc-black'
  height?: number | string
  options?: Partial<monacoTypes.editor.IStandaloneEditorConstructionOptions>
  className?: string
  onReady?: (monaco: typeof import('monaco-editor'), editor: monacoTypes.editor.IStandaloneCodeEditor) => void
}

export function MonacoEditor({ value, onChange, language = 'plaintext', theme = 'vs', height = '100%', options = {}, className, onReady }: MonacoEditorProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const editorRef = useRef<monacoTypes.editor.IStandaloneCodeEditor | null>(null)
  const monacoRef = useRef<typeof import('monaco-editor') | null>(null)

  useEffect(() => {
    if (typeof window === 'undefined') return
    if (editorRef.current) return

    let isMounted = true
    let editor: monacoTypes.editor.IStandaloneCodeEditor | null = null
    let subscription: monacoTypes.IDisposable | null = null
    let timeoutId: ReturnType<typeof setTimeout> | null = null

    // Configure MonacoEnvironment for web workers (Vite-compatible)
    // We intentionally use main thread to avoid worker configuration complexity
    // This must be set BEFORE importing monaco-editor
    if (typeof (window as any).MonacoEnvironment === 'undefined') {
      (window as any).MonacoEnvironment = {
        getWorker: function (_workerId: string, _label: string) {
          // Use a dummy worker that does nothing to satisfy Monaco's requirement
          // while effectively running on the main thread for logic
          const blob = new Blob(['self.onmessage = () => {}'], { type: 'application/javascript' });
          return new Worker(URL.createObjectURL(blob));
        },
      }
    }

    // Wait for container to be available in DOM before importing Monaco
    const checkAndInit = () => {
      if (!isMounted) return
      
      const container = containerRef.current
      if (!container) {
        timeoutId = setTimeout(checkAndInit, 50)
        return
      }
      
      // Ensure container is in DOM - check multiple ways
      try {
        if (!container.parentNode || !container.isConnected || !document.body.contains(container)) {
          timeoutId = setTimeout(checkAndInit, 50)
          return
        }
      } catch (e) {
        // Container might be in shadow DOM or detached
        timeoutId = setTimeout(checkAndInit, 50)
        return
      }

      // Now that container is confirmed ready, import Monaco
      import('monaco-editor').then((monaco) => {
        if (!isMounted) return
        
        // Final check - container must still exist and be in DOM
        const currentContainer = containerRef.current
        try {
          if (!currentContainer || !currentContainer.parentNode || !currentContainer.isConnected || !document.body.contains(currentContainer)) {
            return
          }
        } catch (e) {
          return
        }
        
        monacoRef.current = monaco

        // Register Sruja language with syntax highlighting
        try {
          if (language === 'sruja') {
            registerSrujaLanguage(monaco)
          } else {
            monaco.languages.register({ id: language })
          }
        } catch (err) {
          console.warn('Failed to register language:', err)
          return
        }

        // One more check right before creating editor
        if (!containerRef.current) return
        try {
          if (!containerRef.current.parentNode || !containerRef.current.isConnected || !document.body.contains(containerRef.current)) {
            return
          }
        } catch (e) {
          return
        }

        try {
          editor = monaco.editor.create(containerRef.current, {
            value,
            language,
            automaticLayout: true,
            minimap: { enabled: false },
            theme,
            scrollBeyondLastLine: false,
            ...options,
          })
          editorRef.current = editor
          subscription = editor.onDidChangeModelContent(() => {
            if (editor && isMounted) {
              onChange(editor.getValue())
            }
          })
          if (onReady && isMounted) onReady(monaco, editor)
        } catch (err) {
          console.error('Failed to create Monaco editor:', err)
        }
      }).catch((err) => {
        console.error('Failed to load Monaco editor:', err)
      })
    }

    // Start checking after a small delay to ensure DOM is ready
    timeoutId = setTimeout(checkAndInit, 0)

    return () => {
      isMounted = false
      if (timeoutId) {
        clearTimeout(timeoutId)
        timeoutId = null
      }
      if (subscription) {
        subscription.dispose()
        subscription = null
      }
      if (editor) {
        try {
          editor.dispose()
        } catch (err) {
          // Ignore disposal errors
        }
        editor = null
      }
      if (editorRef.current) {
        try {
          editorRef.current.dispose()
        } catch (err) {
          // Ignore disposal errors
        }
        editorRef.current = null
      }
    }
  }, [language, theme])

  useEffect(() => {
    const editor = editorRef.current
    if (!editor) return
    if (editor.getValue() !== value) {
      editor.setValue(value)
    }
  }, [value])

  useEffect(() => {
    const monaco = monacoRef.current
    const editor = editorRef.current
    if (!monaco || !editor) return
    monaco.editor.setTheme(theme)
  }, [theme])

  return <div ref={containerRef} className={className} style={{ height }} />
}
