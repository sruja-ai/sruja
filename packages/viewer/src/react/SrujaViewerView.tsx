import { useEffect, useRef } from 'react'
import type { ArchitectureJSON, ViewerInstance } from '../types'
import { createViewer } from '../viewer'

export type SrujaViewerViewProps = {
  data: ArchitectureJSON
  onSelect?: (id: string | null) => void
  className?: string
  style?: React.CSSProperties
}

export function SrujaViewerView({ data, onSelect, className, style }: SrujaViewerViewProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const viewerRef = useRef<ViewerInstance | null>(null)

  useEffect(() => {
    if (!containerRef.current) return
    const viewer = createViewer({ container: containerRef.current, data, onSelect })
    viewerRef.current = viewer
    viewer.init()
    return () => {
      viewer.destroy()
      viewerRef.current = null
    }
  }, [])

  useEffect(() => {
    if (!viewerRef.current) return
    viewerRef.current.load(data)
  }, [data])

  return <div ref={containerRef} className={className} style={{ width: '100%', height: '100%', ...(style || {}) }} />
}

