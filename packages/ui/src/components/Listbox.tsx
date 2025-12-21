import { Listbox as HListbox } from '@headlessui/react'

export type ListOption = { id: string; label: string }

export type ListboxProps = {
  options: ListOption[]
  value: ListOption | null
  onChange: (val: ListOption | null) => void
  label?: string
}

export function Listbox({ options, value, onChange, label }: ListboxProps) {
  return (
    <div>
      {label && <div className="mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">{label}</div>}
      <HListbox value={value} onChange={onChange}>
        <div className="relative">
          <HListbox.Button className="w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)] text-left">
            {value?.label || 'Select...'}
          </HListbox.Button>
          <HListbox.Options className="absolute z-50 mt-2 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-background)] shadow-sm">
            {options.map((opt) => (
              <HListbox.Option key={opt.id} value={opt} className={({ active }) => [
                'px-3.5 py-2 text-sm cursor-pointer',
                active ? 'bg-[var(--color-surface)]' : ''
              ].join(' ')}>
                {opt.label}
              </HListbox.Option>
            ))}
          </HListbox.Options>
        </div>
      </HListbox>
    </div>
  )
}

