import { useEffect, useMemo, useState } from 'react'

type Option = { id: string; label: string }
type Question = { id: string; prompt: string; options: Option[]; answer: string; explanation?: string }
type QuizData = { title: string; slug: string; summary?: string; questions: Question[] }

export default function Quiz({ quiz }: { quiz: QuizData }) {
  const [current, setCurrent] = useState(0)
  const [answers, setAnswers] = useState<Record<string, string>>({})
  const [showResult, setShowResult] = useState(false)

  const q = quiz.questions[current]
  const total = quiz.questions.length
  const correctCount = useMemo(() => quiz.questions.reduce((acc, qq) => acc + (answers[qq.id] === qq.answer ? 1 : 0), 0), [answers, quiz.questions])

  useEffect(() => {
    try { window.dispatchEvent(new CustomEvent('sruja:event', { detail: { type: 'quiz.view', slug: quiz.slug } })) } catch {}
  }, [quiz.slug])

  const select = (opt: string) => {
    setAnswers(prev => ({ ...prev, [q.id]: opt }))
  }

  const next = () => {
    if (current < total - 1) {
      setCurrent(current + 1)
    } else {
      setShowResult(true)
      try { window.dispatchEvent(new CustomEvent('sruja:event', { detail: { type: 'quiz.complete', slug: quiz.slug, score: correctCount, total } })) } catch {}
      try {
        const xp = parseInt(localStorage.getItem('sruja-xp') || '0', 10) + Math.max(1, correctCount)
        localStorage.setItem('sruja-xp', String(xp))
      } catch {}
    }
  }

  const prev = () => setCurrent(Math.max(0, current - 1))

  if (showResult) {
    const pct = Math.round((correctCount / total) * 100)
    return (
      <div className="quiz">
        <h2>Results</h2>
        <p>Score: {correctCount}/{total} ({pct}%)</p>
        <button onClick={() => { setShowResult(false); setCurrent(0); setAnswers({}) }}>Retry</button>
      </div>
    )
  }

  return (
    <div className="quiz">
      <div className="quiz-header">
        <h2>{quiz.title}</h2>
        {quiz.summary && <p className="summary">{quiz.summary}</p>}
        <div className="progress">Question {current + 1} of {total}</div>
      </div>
      <div className="quiz-question">
        <p className="prompt">{q.prompt}</p>
        <div className="options">
          {q.options.map(opt => {
            const selected = answers[q.id] === opt.id
            const isCorrect = selected && opt.id === q.answer
            const isWrong = selected && opt.id !== q.answer
            return (
              <button
                key={opt.id}
                className={`option ${selected ? 'selected' : ''} ${isCorrect ? 'correct' : ''} ${isWrong ? 'wrong' : ''}`}
                onClick={() => select(opt.id)}
              >
                {opt.label}
              </button>
            )
          })}
        </div>
        {answers[q.id] && (
          <div className="feedback">
            {answers[q.id] === q.answer ? (
              <span className="ok">Correct</span>
            ) : (
              <span className="err">Not quite</span>
            )}
            {q.explanation && <p className="explain">{q.explanation}</p>}
          </div>
        )}
      </div>
      <div className="quiz-actions">
        <button onClick={prev} disabled={current === 0}>Previous</button>
        <button onClick={next}>{current === total - 1 ? 'Finish' : 'Next'}</button>
      </div>
      <style>{`
        .quiz { border: 1px solid var(--color-border); border-radius: 8px; padding: 16px; background: var(--color-background); }
        .quiz-header .summary { color: var(--color-text-secondary); margin: 4px 0 8px; }
        .progress { font-size: 12px; color: var(--color-text-secondary); }
        .prompt { font-weight: 600; }
        .options { display: grid; gap: 8px; margin-top: 8px; }
        .option { padding: 8px 12px; border: 1px solid var(--color-border); border-radius: 6px; background: var(--color-surface); cursor: pointer; text-align: left; }
        .option.selected { border-color: var(--color-primary); background: var(--color-primary-50); }
        .option.correct { border-color: var(--color-success-500); }
        .option.wrong { border-color: var(--color-error-500); }
        .quiz-actions { display: flex; gap: 8px; margin-top: 12px; }
        .feedback { margin-top: 8px; }
        .ok { color: var(--color-success-500); font-weight: 600; }
        .err { color: var(--color-error-500); font-weight: 600; }
        .explain { color: var(--color-text-secondary); margin: 4px 0 0; }
      `}</style>
    </div>
  )
}
