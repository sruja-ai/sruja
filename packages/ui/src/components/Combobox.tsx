import { useState } from 'react'
import { Combobox as HCombobox } from '@headlessui/react'

export type ComboOption = { id: string; label: string }

export type ComboboxProps = {
  options: ComboOption[]
  value?: ComboOption | null
  onChange?: (value: ComboOption | null) => void
  placeholder?: string
}

export function Combobox({ options, value, onChange, placeholder }: ComboboxProps) {
  const [query, setQuery] = useState('')
  const filtered = query === '' ? options : options.filter((o) => o.label.toLowerCase().includes(query.toLowerCase()))
  return (
    <HCombobox value={value} onChange={onChange} nullable>
      <div className="relative">
        <HCombobox.Input
          className="w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]"
          placeholder={placeholder}
          displayValue={(opt: ComboOption) => opt?.label || ''}
          onChange={(e) => setQuery(e.target.value)}
        />
        <HCombobox.Options className="absolute z-50 mt-2 w-full max-h-52 overflow-auto rounded-md border border-[var(--color-border)] bg-[var(--color-background)] shadow-sm">
          {filtered.length === 0 && (
            <div className="px-3.5 py-2 text-sm text-[var(--color-text-tertiary)]">No results</div>
          )}
          {filtered.map((opt) => (
            <HCombobox.Option key={opt.id} value={opt} className={({ active }) => [
              'px-3.5 py-2 text-sm cursor-pointer',
              active ? 'bg-[var(--color-surface)]' : ''
            ].join(' ')}>
              {opt.label}
            </HCombobox.Option>
          ))}
        </HCombobox.Options>
      </div>
    </HCombobox>
  )
}

