import { useState, useEffect, useRef } from 'react'
import Editor from '@monaco-editor/react'
import './App.css'
import { examples } from './examples'

interface CompileResponse {
  d2?: string
  svg?: string
  errors?: string[]
  warnings?: string[]
  valid: boolean
}

function App() {
  const [code, setCode] = useState(examples[0].code)
  const [errors, setErrors] = useState<string[]>([])
  const [isCompiling, setIsCompiling] = useState(false)
  const [zoom, setZoom] = useState(1)
  const [pan, setPan] = useState({ x: 0, y: 0 })
  const [isDragging, setIsDragging] = useState(false)
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 })
  const [editorCollapsed, setEditorCollapsed] = useState(false)
  const [previewCollapsed, setPreviewCollapsed] = useState(false)
  const diagramRef = useRef<HTMLDivElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    // Reset pan and zoom when code changes
    setPan({ x: 0, y: 0 })
    setZoom(1)
    
    const timer = setTimeout(() => {
      compileCode()
    }, 500)

    return () => clearTimeout(timer)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [code])

  const compileCode = async () => {
    setIsCompiling(true)
    setErrors([])

    try {
      const response = await fetch('/api/compile', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ code }),
      })

      const data: CompileResponse = await response.json()

      if (data.valid && (data.d2 || data.svg)) {
        setErrors([])
        
        if (diagramRef.current) {
          try {
            if (data.svg) {
              // D2 rendered to SVG - just display it
              diagramRef.current.innerHTML = data.svg
            } else if (data.d2) {
              // D2 code but no SVG - show code with link to D2 playground
              const d2Code = encodeURIComponent(data.d2)
              const playgroundUrl = `https://play.d2lang.com/?script=${d2Code}`
              diagramRef.current.innerHTML = `
                <div style="padding: 20px; text-align: center; color: #888;">
                  <p>D2 diagram code generated. SVG rendering requires D2 libraries.</p>
                  <p style="margin-top: 10px;">
                    <a href="${playgroundUrl}" target="_blank" rel="noopener noreferrer" 
                       style="color: #4A90E2; text-decoration: underline;">
                      View in D2 Playground →
                    </a>
                  </p>
                  <pre style="margin-top: 20px; text-align: left; background: #1e1e1e; padding: 15px; border-radius: 4px; overflow-x: auto;">
                    <code style="color: #d4d4d4; font-size: 12px;">${data.d2}</code>
                  </pre>
                </div>
              `
            }
          } catch (err) {
            console.error('Render error:', err)
            const errorMessage = err instanceof Error 
              ? err.message 
              : (typeof err === 'object' && err !== null && 'message' in err)
                ? String(err.message)
                : String(err)
            setErrors([`Render error: ${errorMessage}`])
          }
        }
      } else {
        setErrors(data.errors || [])
        if (diagramRef.current) {
          diagramRef.current.innerHTML = ''
        }
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setErrors([`Compilation error: ${errorMessage}`])
      if (diagramRef.current) {
        diagramRef.current.innerHTML = ''
      }
    } finally {
      setIsCompiling(false)
    }
  }

  const handleExampleChange = (exampleCode: string) => {
    setCode(exampleCode)
  }

  const handleZoomIn = () => {
    setZoom((prev) => Math.min(prev + 0.1, 3))
  }

  const handleZoomOut = () => {
    setZoom((prev) => Math.max(prev - 0.1, 0.5))
  }

  const handleZoomReset = () => {
    setZoom(1)
    setPan({ x: 0, y: 0 })
  }

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 0) { // Left mouse button
      setIsDragging(true)
      setDragStart({
        x: e.clientX - pan.x,
        y: e.clientY - pan.y,
      })
    }
  }

  const handleMouseMove = (e: React.MouseEvent) => {
    if (isDragging) {
      setPan({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y,
      })
    }
  }

  const handleWheel = (e: React.WheelEvent) => {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault()
      const delta = e.deltaY > 0 ? -0.1 : 0.1
      setZoom((prev) => Math.min(Math.max(prev + delta, 0.5), 3))
    }
  }

  const handleMouseUp = () => {
    setIsDragging(false)
  }

  useEffect(() => {
    if (isDragging) {
      const handleGlobalMouseMove = (e: MouseEvent) => {
        setPan({
          x: e.clientX - dragStart.x,
          y: e.clientY - dragStart.y,
        })
      }

      const handleGlobalMouseUp = () => {
        setIsDragging(false)
      }

      window.addEventListener('mousemove', handleGlobalMouseMove)
      window.addEventListener('mouseup', handleGlobalMouseUp)

      return () => {
        window.removeEventListener('mousemove', handleGlobalMouseMove)
        window.removeEventListener('mouseup', handleGlobalMouseUp)
      }
    }
  }, [isDragging, dragStart])

  return (
    <div className="app">
      <header className="header">
        <h1>Sruja Playground</h1>
        <div className="header-actions">
          <select
            className="example-select"
            onChange={(e) => {
              const example = examples.find((ex) => ex.name === e.target.value)
              if (example) handleExampleChange(example.code)
            }}
          >
            {examples.map((ex) => (
              <option key={ex.name} value={ex.name}>
                {ex.name}
              </option>
            ))}
          </select>
          {isCompiling && <span className="status">Compiling...</span>}
        </div>
      </header>

      <div className="main-content">
        <div className={`editor-panel ${editorCollapsed ? 'collapsed' : ''}`}>
          <div className="panel-header">
            <div className="panel-title">
              <button
                className="collapse-btn"
                onClick={() => setEditorCollapsed(!editorCollapsed)}
                title={editorCollapsed ? 'Expand Editor' : 'Collapse Editor'}
                aria-label={editorCollapsed ? 'Expand Editor' : 'Collapse Editor'}
              >
                {editorCollapsed ? '▶' : '▼'}
              </button>
              Editor
            </div>
          </div>
          {!editorCollapsed && (
            <Editor
              height="100%"
              defaultLanguage="plaintext"
              value={code}
              onChange={(value) => setCode(value || '')}
              theme="vs-dark"
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                lineNumbers: 'on',
                scrollBeyondLastLine: false,
                wordWrap: 'on',
              }}
            />
          )}
        </div>

        <div className={`preview-panel ${previewCollapsed ? 'collapsed' : ''}`}>
          <div className="panel-header">
            <div className="panel-title">
              <button
                className="collapse-btn"
                onClick={() => setPreviewCollapsed(!previewCollapsed)}
                title={previewCollapsed ? 'Expand Preview' : 'Collapse Preview'}
                aria-label={previewCollapsed ? 'Expand Preview' : 'Collapse Preview'}
              >
                {previewCollapsed ? '▶' : '▼'}
              </button>
              Diagram Preview
            </div>
            {errors.length > 0 && (
              <span className="error-badge">{errors.length} error(s)</span>
            )}
            {errors.length === 0 && !previewCollapsed && (
              <div className="controls-group">
                <div className="zoom-controls">
                  <button 
                    className="zoom-btn" 
                    onClick={handleZoomOut}
                    title="Zoom Out"
                    aria-label="Zoom Out"
                  >
                    −
                  </button>
                  <span className="zoom-level">{Math.round(zoom * 100)}%</span>
                  <button 
                    className="zoom-btn" 
                    onClick={handleZoomIn}
                    title="Zoom In"
                    aria-label="Zoom In"
                  >
                    +
                  </button>
                  <button 
                    className="zoom-btn reset" 
                    onClick={handleZoomReset}
                    title="Reset Zoom & Pan"
                    aria-label="Reset Zoom & Pan"
                  >
                    ↺
                  </button>
                </div>
              </div>
            )}
          </div>
          {!previewCollapsed && (
            errors.length > 0 ? (
              <div className="errors">
                {errors.map((error, idx) => (
                  <div key={idx} className="error-item">
                    {error}
                  </div>
                ))}
              </div>
            ) : (
              <div 
                ref={containerRef}
                className="diagram-container"
                onMouseDown={handleMouseDown}
                onMouseMove={handleMouseMove}
                onMouseUp={handleMouseUp}
                onMouseLeave={handleMouseUp}
                onWheel={handleWheel}
                style={{ 
                  cursor: isDragging ? 'grabbing' : 'grab',
                }}
              >
                <div 
                  className="diagram-wrapper"
                  style={{
                    transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
                    transformOrigin: 'top left',
                  }}
                >
                  <div 
                    ref={diagramRef} 
                    className="diagram"
                  ></div>
                </div>
              </div>
            )
          )}
        </div>
      </div>
    </div>
  )
}

export default App

