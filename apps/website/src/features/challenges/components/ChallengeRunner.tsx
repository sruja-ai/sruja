// apps/website/src/features/challenges/components/ChallengeRunner.tsx
import { useEffect, useState } from 'react'
import { SrujaMonacoEditor, Button, MantineProvider } from '@sruja/ui'
import { initWasm } from '@sruja/shared'
import { formatParseError } from '@/shared/utils/errors'
import { markCompleted } from '@/shared/lib/progress'
import { trackEvent } from '@/shared/utils/analytics'

type Check = { type: 'relationExists' | 'noErrors' | 'elementExists'; source?: string; target?: string; label?: string; name?: string; message?: string }
type Challenge = { title: string; slug: string; summary?: string; difficulty?: 'beginner' | 'intermediate' | 'advanced'; topic?: string; estimatedTime?: string; initialDsl: string; checks: Check[]; hints?: string[]; solution?: string }

export default function ChallengeRunner({ challenge }: { challenge: Challenge }) {
  const [dsl, setDsl] = useState(challenge.initialDsl)
  const [running, setRunning] = useState(false)
  const [result, setResult] = useState<{ passed: boolean; details: string[] } | null>(null)
  const [attempts, setAttempts] = useState(0)
  const [shownHintCount, setShownHintCount] = useState(0)

  useEffect(() => {
    trackEvent('challenge.view', { slug: challenge.slug });
  }, [challenge.slug])

  const run = async () => {
    setRunning(true)
    setResult(null)
    const details: string[] = []
    try {
      setAttempts(a => a + 1)
      const api = await initWasm()
      const jsonStr = await api.parseDslToJson(dsl)
      const data = JSON.parse(jsonStr)
      let passed = true
      for (const c of challenge.checks) {
        if (c.type === 'noErrors') {
          // assume parser throws on errors; presence of data implies OK
          details.push(c.message || 'No parse errors')
        } else if (c.type === 'relationExists') {
          const collect = (arch: any): Array<{from: string; to: string}> => {
            const out: Array<{from: string; to: string}> = []
            const add = (arr: any[]) => {
              for (const r of arr || []) {
                if (r && (r.from || r.source) && (r.to || r.target)) {
                  out.push({ from: r.from ?? r.source, to: r.to ?? r.target })
                }
              }
            }
            add(arch?.relations || [])
            for (const sys of arch?.systems || []) {
              add(sys?.relations || [])
              for (const ctn of sys?.containers || []) {
                add(ctn?.relations || [])
                for (const comp of ctn?.components || []) add(comp?.relations || [])
              }
              for (const comp of sys?.components || []) add(comp?.relations || [])
            }
            for (const ctn of arch?.containers || []) {
              add(ctn?.relations || [])
              for (const comp of ctn?.components || []) add(comp?.relations || [])
            }
            for (const comp of arch?.components || []) add(comp?.relations || [])
            return out
          }
          const rels = collect(data?.architecture || data)
          const matchName = (actual: string, expected?: string) => {
            if (!expected) return false
            const a = String(actual || '').trim()
            const e = String(expected || '').trim()
            if (!a || !e) return false
            if (a === e) return true
            const al = a.split('.').pop()
            const el = e.split('.').pop()
            if (al && el && al === el) return true
            if (a.endsWith('.' + e)) return true
            return false
          }
          const ok = rels.some((r: any) => matchName(r.from, c.source) && matchName(r.to, c.target))
          if (!ok) {
            passed = false
            details.push(c.message || `Add relation ${c.source} -> ${c.target}`)
          } else {
            details.push(`Found relation ${c.source} -> ${c.target}`)
          }
        } else if (c.type === 'elementExists') {
          const names = new Set<string>()
          const visit = (val: any) => {
            if (!val) return
            if (Array.isArray(val)) {
              for (const item of val) visit(item)
              return
            }
            if (typeof val === 'object') {
              const nm = (val.name || val.id || '').toString().trim()
              if (nm) names.add(nm)
              for (const k of Object.keys(val)) visit(val[k])
            }
          }
          visit(data?.architecture || data)
          const expected = (c.name || '').toString().trim()
          const ok = expected && Array.from(names).some((n) => {
            if (!n) return false
            if (n === expected) return true
            const nl = n.split('.').pop()
            const el = expected.split('.').pop()
            return !!nl && !!el && nl === el
          })
          if (!ok) {
            passed = false
            details.push(c.message || `Create element ${expected}`)
          } else {
            details.push(`Found element ${expected}`)
          }
        }
      }
      setResult({ passed, details })
      if (passed) {
        try { markCompleted(challenge.slug) } catch {}
      }
      trackEvent('challenge.run', { slug: challenge.slug, passed });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      const detailsFromCompiler = formatParseError(msg, dsl)
      setResult({ passed: false, details: detailsFromCompiler })
    } finally {
      setRunning(false)
    }
  }

  const showHint = () => {
    if (!challenge.hints || challenge.hints.length === 0) return
    const next = Math.min(challenge.hints.length, shownHintCount + 1)
    setShownHintCount(next)
    const hintsToShow = challenge.hints.slice(0, next)
    setResult(prev => ({ passed: false, details: [...(prev?.details || []), ...hintsToShow] }))
  }

  const revealSolution = () => {
    if (!challenge.solution) return
    setDsl(challenge.solution)
  }

  return (
    <MantineProvider>
      <div className="challenge">
      <div className="header">
        <h2>{challenge.title}</h2>
        {challenge.summary && <p className="summary">{challenge.summary}</p>}
      </div>
      <div className="editor">
        <SrujaMonacoEditor
          value={dsl}
          onChange={(v) => setDsl(v || '')}
          theme="vs-dark"
          options={{ minimap: { enabled: false }, fontSize: 14, wordWrap: 'on' }}
        />
      </div>
      <div className="actions">
        <Button variant="primary" onClick={run} disabled={running} isLoading={running}>
          {running ? 'Running...' : 'Run Tests'}
        </Button>
        {attempts >= 1 && !result?.passed && (
          <>
            <Button variant="outline" onClick={showHint} disabled={running || (challenge.hints?.length || 0) === shownHintCount}>
              Show Hint
            </Button>
            <Button variant="outline" onClick={revealSolution} disabled={running || !challenge.solution}>
              Reveal Solution
            </Button>
          </>
        )}
      </div>
      {result && (
        <div className={`result ${result.passed ? 'ok' : 'err'}`}>
          <strong>{result.passed ? 'Challenge passed' : 'Needs fixes'}</strong>
          <ul>
            {result.details.map((d, i) => <li key={i}>{d}</li>)}
          </ul>
        </div>
      )}
      <div className="hint">
        <small>Progress is saved locally in your browser.</small>
      </div>
      <style>{`
        .challenge { border: 1px solid var(--color-border); border-radius: 8px; padding: 16px; background: var(--color-background); }
        .summary { color: var(--color-text-tertiary); }
        .editor { height: 360px; margin-top: 12px; }
        .actions { margin-top: 12px; display: flex; gap: 8px; }
        .result { margin-top: 12px; padding: 12px; border-radius: 6px; }
        .result.ok { background: var(--success-dim); border: 1px solid var(--color-success-500); }
        .result.err { background: var(--error-dim); border: 1px solid var(--color-error-500); }
        .hint { margin-top: 8px; color: var(--color-text-tertiary); }
      `}</style>
      </div>
    </MantineProvider>
  )
}
