import { Switch as HSwitch } from '@headlessui/react'

export type SwitchProps = {
  checked: boolean
  onChange: (val: boolean) => void
  label?: string
}

export function Switch({ checked, onChange, label }: SwitchProps) {
  return (
    <div className="flex items-center gap-2">
      {label && <span className="text-sm text-[var(--color-text-secondary)]">{label}</span>}
  <HSwitch
    checked={checked}
    onChange={onChange}
    className={[
      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)]',
      checked ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-neutral-200)]'
    ].join(' ')}
  >
        <span
          className={[
            'inline-block h-4 w-4 transform rounded-full bg-[var(--color-background)] transition',
            checked ? 'translate-x-6' : 'translate-x-1'
          ].join(' ')}
        />
      </HSwitch>
    </div>
  )
}
