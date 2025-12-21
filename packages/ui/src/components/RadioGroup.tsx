import { RadioGroup as HRadioGroup } from '@headlessui/react'

export type RadioOption = { id: string; label: string; description?: string }

export type RadioGroupProps = {
  options: RadioOption[]
  value: RadioOption | null
  onChange: (val: RadioOption | null) => void
  label?: string
}

export function RadioGroup({ options, value, onChange, label }: RadioGroupProps) {
  return (
    <div>
      {label && <div className="mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">{label}</div>}
      <HRadioGroup value={value} onChange={onChange}>
        <div className="space-y-2">
          {options.map((opt) => (
            <HRadioGroup.Option key={opt.id} value={opt} className={({ checked }) => [
              'px-3.5 py-2.5 rounded-md border cursor-pointer',
              checked ? 'border-[var(--color-primary)] bg-[var(--color-primary-50)]' : 'border-[var(--color-border)]'
            ].join(' ')}>
              {() => (
                <div>
                  <div className="text-sm text-[var(--color-text-primary)] font-medium">{opt.label}</div>
                  {opt.description && <div className="text-xs text-[var(--color-text-tertiary)]">{opt.description}</div>}
                </div>
              )}
            </HRadioGroup.Option>
          ))}
        </div>
      </HRadioGroup>
    </div>
  )
}

