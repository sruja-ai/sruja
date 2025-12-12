import { useEffect, useMemo, useRef, useState } from 'react'
import { logger } from '@sruja/shared'

export interface MermaidDiagramProps {
  code: string
  onExpand?: (svg: string, code: string) => void
}

// Initialize mermaid once
let mermaidInitialized = false
async function initMermaid() {
  if (mermaidInitialized) return
  try {
    const mermaidModule = await import('mermaid')
    const mermaid = mermaidModule.default || mermaidModule
    if (typeof mermaid.initialize === 'function') {
      mermaid.initialize({ 
        startOnLoad: false,
        theme: 'default',
        securityLevel: 'loose',
        flowchart: { 
          useMaxWidth: true, 
          htmlLabels: true,
          subGraphTitleMargin: { top: 10, bottom: 20 }
        },
      })
      mermaidInitialized = true
    }
  } catch (err) {
    console.error('Failed to initialize Mermaid:', err)
  }
}

export function MermaidDiagram({ code, onExpand }: MermaidDiagramProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const [svgContent, setSvgContent] = useState<string>('')
  const renderId = useMemo(() => `mermaid-${Math.random().toString(36).slice(2)}`, [])

  useEffect(() => {
    let active = true
    async function render() {
      const diagramCode = code || 'graph TD; A-->B;'
      try {
        // Initialize mermaid first
        await initMermaid()
        
        // Import and use mermaid
        const mermaidModule = await import('mermaid')
        const mermaid = mermaidModule.default || mermaidModule
        
        // Parse and render the diagram
        await mermaid.parse(diagramCode)
        const result = await mermaid.render(renderId, diagramCode)
        if (!active) return
        const svg = result?.svg || ''
        setSvgContent(svg)
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : String(e)
        logger.error('Mermaid render error', {
          component: 'mermaid',
          action: 'render',
          errorType: e instanceof Error ? e.constructor.name : 'unknown',
          error: errorMsg,
        })
        setSvgContent(`<svg xmlns="http://www.w3.org/2000/svg" width="400" height="60"><text x="10" y="35" fill="red">Mermaid render failed: ${errorMsg}</text></svg>`) 
      }
    }
    render()
    return () => { active = false }
  }, [code, renderId])

  const handleExpand = (e: React.MouseEvent) => {
    if (!onExpand) return
    e.preventDefault()
    e.stopPropagation()
    onExpand(svgContent, code)
  }

  return (
    <div className="relative">
      <div ref={containerRef} dangerouslySetInnerHTML={{ __html: svgContent }} />
      {onExpand && (
        <button
          type="button"
          onClick={handleExpand}
          className="absolute top-2 right-2 inline-flex items-center rounded-md border border-slate-300 bg-white px-2 py-1 text-xs text-slate-700 shadow-sm hover:bg-slate-50"
        >
          Expand
        </button>
      )}
    </div>
  )
}
