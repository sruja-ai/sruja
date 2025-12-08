import { useEffect, useRef } from 'react'

export function AnimatedSequenceDiagram({ steps, activeIndex }: { steps: { from: string; to: string; description?: string }[], activeIndex: number }) {
  const participantsOrdered: string[] = []
  const seen = new Set<string>()
  steps.forEach(s => {
    if (!seen.has(s.from)) { participantsOrdered.push(s.from); seen.add(s.from) }
    if (!seen.has(s.to)) { participantsOrdered.push(s.to); seen.add(s.to) }
  })

  const rowsRef = useRef<Array<HTMLDivElement | null>>([])
  useEffect(() => {
    if (activeIndex >= 0 && rowsRef.current[activeIndex]) {
      rowsRef.current[activeIndex]?.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  }, [activeIndex])

  return (
    <div className="seq-container" style={{ overflowX: 'auto' }}>
      <div className="seq-grid" style={{ gridTemplateColumns: `repeat(${participantsOrdered.length}, 1fr)` }}>
        {participantsOrdered.map((p) => (
          <div key={p} className="lifeline">
            <div className="lifeline-header">{p}</div>
            <div className="lifeline-line" />
          </div>
        ))}
      </div>
      <div className="seq-rows">
        {steps.map((step, idx) => {
          const fromIdx = participantsOrdered.indexOf(step.from)
          const toIdx = participantsOrdered.indexOf(step.to)
          const left = Math.min(fromIdx, toIdx)
          const right = Math.max(fromIdx, toIdx)
          const cols = right - left + 1
          const direction = fromIdx <= toIdx ? 'right' : 'left'
          const isActive = idx === activeIndex
          return (
            <div key={idx} ref={el => rowsRef.current[idx] = el} className={`seq-row ${isActive ? 'active' : ''}`}>
              <div className="seq-message" style={{ gridColumn: `${left + 1} / span ${cols}` }}>
                <div className={`message-arrow ${direction} ${isActive ? 'animate' : ''}`}/>
                <div className="message-label">
                  <span className="from">{step.from}</span>
                  <span className="arrow">→</span>
                  <span className="to">{step.to}</span>
                  {step.description && <span className="desc">{step.description}</span>}
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

