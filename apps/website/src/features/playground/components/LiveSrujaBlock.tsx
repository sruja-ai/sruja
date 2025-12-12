// apps/website/src/features/playground/components/LiveSrujaBlock.tsx
import { useEffect, useState } from 'react'
import { SrujaMonacoEditor } from '@sruja/ui'
import { SrujaLoader } from '@sruja/ui'
import { initWasm, logger } from '@sruja/shared'
import { trackEvent, trackInteraction } from '@/shared/utils/analytics'
import { type ArchitectureJSON } from '@sruja/diagram'
import { DiagramPreview } from '../../playground/components/DiagramPreview'

export default function LiveSrujaBlock({ initialDsl }: { initialDsl: string }) {
  const [dsl, setDsl] = useState(initialDsl)
  const [data, setData] = useState<ArchitectureJSON | null>(null)
  const [busy, setBusy] = useState(false)
  const [errorHeader, setErrorHeader] = useState<string | null>(null)

  const renderDiagram = async () => {
    setBusy(true)
    setErrorHeader(null)
    trackInteraction('click', 'render_button', { component: 'playground' })

    try {
      const normalize = (s: string) => {
        const basic = s
          .replace(/\u2192/g, '->')
          .replace(/[“”]/g, '"')
          .replace(/[’]/g, "'")
          .replace(/\u2013|\u2014/g, '-')
      return basic
        .split(/\r?\n/)
        .map(line => line.replace(/^\s*\d+\s*[→:.-]\s?/, ''))
        .join('\n')
      }
      const input = normalize(dsl)
      const api = await initWasm()
      const jsonStr = await api.parseDslToJson(input)
      const parsed = JSON.parse(jsonStr)
      setData(parsed)
      trackInteraction('success', 'render_diagram', { component: 'playground' })
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error)
      setErrorHeader(msg)
      logger.error('Failed to render diagram', {
        component: 'playground',
        action: 'render',
        errorType: error instanceof Error ? error.constructor.name : 'unknown',
        error: msg,
      })
    } finally {
      setBusy(false)
    }
  }

  // Initial render
  useEffect(() => {
    renderDiagram()
    trackEvent('live.render_view');
  }, [])

  const [theme, setTheme] = useState<'vs' | 'vs-dark' | 'hc-black'>(() => (typeof document !== 'undefined' && document.documentElement.getAttribute('data-theme') === 'dark' ? 'vs-dark' : 'vs'))
  useEffect(() => {
    const handler = () => {
      setTheme(document.documentElement.getAttribute('data-theme') === 'dark' ? 'vs-dark' : 'vs')
    }
    try {
      window.addEventListener('theme-change', handler)
    } catch (_e) { void 0 }
    return () => {
      try { window.removeEventListener('theme-change', handler) } catch (_e) { void 0 }
    }
  }, [])

  return (
    <div style={{ border: '1px solid var(--color-border)', borderRadius: 8, overflow: 'hidden' }}>
      <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) minmax(0, 1fr)', gap: 0, minHeight: 640 }}>
        <div style={{ borderRight: '1px solid var(--color-border)', position: 'relative' }}>
          <SrujaMonacoEditor
            value={dsl}
            onChange={(v) => setDsl(v || '')}
            theme={theme}
            options={{ minimap: { enabled: false }, wordWrap: 'on', fontSize: 14 }}
            height="640px"
          />
        </div>
        <div style={{ position: 'relative', height: 640 }}>
          {errorHeader && (
            <div style={{ position: 'absolute', top: 0, left: 0, right: 0, padding: 8, background: '#fee2e2', color: '#b91c1c', fontSize: 13, borderBottom: '1px solid #fecaca', zIndex: 10 }}>
              Failed to render: {errorHeader}
            </div>
          )}
          {data ? (
            <DiagramPreview data={data} />
          ) : (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--color-text-secondary)' }}>
              {busy ? <SrujaLoader size={32} /> : <div>Click Render to view diagram</div>}
            </div>
          )}
        </div>
      </div>
      <div style={{ padding: 8, display: 'flex', gap: 8, justifyContent: 'flex-end', borderTop: '1px solid var(--color-border)' }}>
        <button
          onClick={() => {
            trackInteraction('click', 'render_button', { component: 'playground' });
            renderDiagram();
          }}
          disabled={busy}
          style={{ padding: '8px 12px', border: '1px solid var(--color-border)', borderRadius: 6, background: 'var(--color-background)', cursor: 'pointer', opacity: busy ? 0.7 : 1 }}
        >
          {busy ? 'Rendering...' : 'Render Diagram'}
        </button>
      </div>
    </div>
  )
}
