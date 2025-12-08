// apps/website/src/features/playground/components/LiveSrujaBlock.tsx
import { useEffect, useRef, useState } from 'react'
import { SrujaMonacoEditor } from '@sruja/ui'
import { SrujaLoader } from '@sruja/ui'
import { initWasm, logger } from '@sruja/shared'
import { createViewer } from '@sruja/viewer'
import '@sruja/viewer-core/app/index.css'
import { trackEvent, trackInteraction } from '@/shared/utils/analytics'

export default function LiveSrujaBlock({ initialDsl }: { initialDsl: string }) {
  const [dsl, setDsl] = useState(initialDsl)
  const [busy, setBusy] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  const renderDiagram = async () => {
    setBusy(true)
    trackInteraction('click', 'render_button', { component: 'playground' })
    
    try {
      const api = await initWasm()
      const jsonStr = await api.parseDslToJson(dsl)
      const data = JSON.parse(jsonStr)
      const el = containerRef.current
      if (!el) {
        logger.error('Container element not found', { component: 'playground', action: 'render' })
        return
      }
      el.innerHTML = ''
      const viewer = createViewer({ container: el, data })
      await viewer.init()
      trackInteraction('success', 'render_diagram', { component: 'playground' })
    } catch (error) {
      logger.error('Failed to render diagram', {
        component: 'playground',
        action: 'render',
        errorType: error instanceof Error ? error.constructor.name : 'unknown',
        error: error instanceof Error ? error.message : String(error),
      })
    } finally {
      setBusy(false)
    }
  }

  useEffect(() => {
    trackEvent('live.render_view');
  }, [])

  return (
    <div style={{ border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 0, minHeight: 360 }}>
        <div style={{ borderRight: '1px solid var(--color-border)' }}>
          <SrujaMonacoEditor
            value={dsl}
            onChange={(v) => setDsl(v || '')}
            theme="vs-dark"
            options={{ minimap: { enabled: false }, wordWrap: 'on', fontSize: 14 }}
            height="360px"
          />
        </div>
        <div style={{ position: 'relative' }}>
          {busy && (
            <div style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'rgba(255,255,255,0.6)' }}>
              <div style={{ textAlign: 'center', color: 'var(--color-text-secondary)' }}>
                <SrujaLoader size={32} />
                <div style={{ marginTop: 8, fontSize: 12 }}>Rendering...</div>
              </div>
            </div>
          )}
          <div ref={containerRef} style={{ width: '100%', height: '360px' }} />
        </div>
      </div>
      <div style={{ padding: 8, display: 'flex', gap: 8, justifyContent: 'flex-end', borderTop: '1px solid var(--color-border)' }}>
        <button 
          onClick={() => {
            trackInteraction('click', 'render_button', { component: 'playground' });
            renderDiagram();
          }} 
          disabled={busy} 
          style={{ padding: '8px 12px', border: '1px solid var(--color-border)', borderRadius: 6, background: 'var(--color-background)', cursor: 'pointer' }}
        >
          Render Diagram
        </button>
      </div>
    </div>
  )
}
