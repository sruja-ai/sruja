import React from 'react'

type NodeType = 'person' | 'system' | 'container' | 'component' | 'datastore' | 'queue' | 'requirement' | 'adr' | 'deployment'

interface StepGuideProps {
  step: number
  onClose: () => void
  onNext: () => void
  onPrev: () => void
  onSetLevel: (level: number) => void
  onAddNode: (type: NodeType) => void
}

export default function StepGuide({ step, onClose, onNext, onPrev, onSetLevel, onAddNode }: StepGuideProps) {
  return (
    <div className="fixed top-4 right-4 z-[1000] w-[360px] rounded-md border border-[var(--color-border)] bg-[var(--color-background)] shadow-xl">
      <div className="px-4 py-3 border-b border-[var(--color-border)] flex items-center justify-between">
        <div className="text-sm font-semibold text-[var(--color-text-primary)]">Design Mode</div>
        <button onClick={onClose} className="px-2 py-1 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">Close</button>
      </div>
      <div className="p-4 space-y-3">
        <div className="text-xs text-[var(--color-text-secondary)]">Step {step} of 4</div>
        {step === 1 && (
          <div className="space-y-2">
            <div className="text-sm text-[var(--color-text-primary)]">Define Context</div>
            <div className="flex gap-2">
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onAddNode('system')}>Add System</button>
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onAddNode('person')}>Add Person</button>
            </div>
            <div className="flex gap-2">
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onSetLevel(1)}>View Context</button>
            </div>
          </div>
        )}
        {step === 2 && (
          <div className="space-y-2">
            <div className="text-sm text-[var(--color-text-primary)]">Add Containers</div>
            <div className="flex gap-2">
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onAddNode('container')}>Add Container</button>
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onAddNode('datastore')}>Add Datastore</button>
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onAddNode('queue')}>Add Queue</button>
            </div>
            <div className="flex gap-2">
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onSetLevel(2)}>View Containers</button>
            </div>
          </div>
        )}
        {step === 3 && (
          <div className="space-y-2">
            <div className="text-sm text-[var(--color-text-primary)]">Add Components</div>
            <div className="flex gap-2">
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onAddNode('component')}>Add Component</button>
            </div>
            <div className="flex gap-2">
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onSetLevel(3)}>View Components</button>
            </div>
          </div>
        )}
        {step === 4 && (
          <div className="space-y-2">
            <div className="text-sm text-[var(--color-text-primary)]">Stitch & Share</div>
            <div className="flex gap-2">
              <button className="px-3 py-1.5 rounded-md border" onClick={() => onSetLevel(0)}>All Layers</button>
            </div>
          </div>
        )}
      </div>
      <div className="px-4 py-3 border-t border-[var(--color-border)] flex items-center justify-between">
        <button className="px-3 py-1.5 rounded-md border" onClick={onPrev} disabled={step === 1}>Back</button>
        <button className="px-3 py-1.5 rounded-md border" onClick={onNext} disabled={step === 4}>Next</button>
      </div>
    </div>
  )
}
