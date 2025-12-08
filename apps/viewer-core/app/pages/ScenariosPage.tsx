import { GitBranch, Play, Square } from 'lucide-react'
import { AnimatedSequenceDiagram } from './AnimatedSequenceDiagram'

export function ScenariosPage({ scenarios, onShowInDiagram, onPlay, onStop, onNextStep, onPrevStep, isPlaying, currentScenarioId, stepIndex, followCamera, onToggleFollowCamera, animated, onToggleAnimated }: { scenarios: any[], onShowInDiagram: (id: string) => void, onPlay: (id: string) => void, onStop: () => void, onNextStep: () => void, onPrevStep: () => void, isPlaying: boolean, currentScenarioId: string | null, stepIndex: number, followCamera: boolean, onToggleFollowCamera: () => void, animated: boolean, onToggleAnimated: () => void }) {
  if (scenarios.length === 0) {
    return (
      <div className="page-content" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <div style={{ textAlign: 'center', color: 'var(--text-muted)' }}>
          <GitBranch size={48} style={{ marginBottom: 16, opacity: 0.5 }} />
          <h3>No scenarios defined</h3>
        </div>
      </div>
    )
  }

  return (
    <div className="page-content">
      <h2>Scenarios</h2>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
        <label style={{ display: 'inline-flex', alignItems: 'center', gap: 8, fontSize: 12, color: 'var(--text-secondary)' }}>
          <input type="checkbox" checked={followCamera} onChange={onToggleFollowCamera} />
          Follow camera during playback
        </label>
        <label style={{ display: 'inline-flex', alignItems: 'center', gap: 8, fontSize: 12, color: 'var(--text-secondary)' }}>
          <input type="checkbox" checked={animated} onChange={onToggleAnimated} />
          Animated sequence view
        </label>
        {currentScenarioId && (
          <div style={{ marginLeft: 'auto', display: 'inline-flex', alignItems: 'center', gap: 8 }}>
            <button className="secondary-btn" style={{ padding: '6px 10px' }} onClick={onPrevStep}>Prev</button>
            <span style={{ fontSize: 12, color: 'var(--text-secondary)' }}>Step {stepIndex + 1}</span>
            <button className="secondary-btn" style={{ padding: '6px 10px' }} onClick={onNextStep}>Next</button>
          </div>
        )}
      </div>
      <div className="cards-grid">
        {scenarios.map(scenario => (
          <div key={scenario.id} className="scenario-card">
            <h3>{scenario.title || scenario.label || scenario.id}</h3>

            {scenario.description && (
              <p style={{ fontSize: 14, color: 'var(--text-secondary)', margin: '0 0 16px 0' }}>{scenario.description}</p>
            )}

            <div style={{ display: 'flex', gap: 8, marginTop: 'auto' }}>
              <button className="secondary-btn" style={{ flex: 1, padding: '6px' }} onClick={() => onShowInDiagram(scenario.id)}>
                Show in Diagram
              </button>
              {isPlaying && currentScenarioId === scenario.id ? (
                <button className="primary-btn" style={{ background: 'var(--gradient-error)', flex: 1, padding: '6px' }} onClick={onStop}>
                  <Square size={12} style={{ marginRight: 6 }} /> Stop
                </button>
              ) : (
                <button className="primary-btn" style={{ flex: 1, padding: '6px' }} onClick={() => onPlay(scenario.id)}>
                  <Play size={12} style={{ marginRight: 6 }} /> Play
                </button>
              )}
              <button className="secondary-btn" style={{ flex: 1, padding: '6px' }} onClick={onPrevStep}>
                Prev
              </button>
              <button className="secondary-btn" style={{ flex: 1, padding: '6px' }} onClick={onNextStep}>
                Next
              </button>
            </div>

            {scenario.steps && scenario.steps.length > 0 && (
              <div className="sequence-diagram">
                {animated ? (
                  <AnimatedSequenceDiagram steps={scenario.steps} activeIndex={isPlaying && currentScenarioId === scenario.id ? stepIndex : -1} />
                ) : (
                  <div>
                    {scenario.steps.map((step: any, idx: number) => {
                      const isActive = isPlaying && currentScenarioId === scenario.id && idx === stepIndex
                      return (
                        <div key={idx} className={`sequence-step ${isActive ? 'active' : ''}`}>
                          <span className="from">{step.from}</span>
                          <span className="arrow">→</span>
                          <span className="to">{step.to}</span>
                          {step.description && (
                            <span className="desc">{step.description}</span>
                          )}
                        </div>
                      )
                    })}
                  </div>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}

