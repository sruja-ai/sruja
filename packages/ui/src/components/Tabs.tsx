import { Fragment } from 'react'
import { Tab } from '@headlessui/react'

export type TabsProps = {
  tabs: Array<{ id: string; label: string; content: any }>
  defaultIndex?: number
  onChange?: (index: number) => void
}

export function Tabs({ tabs, defaultIndex = 0, onChange }: TabsProps) {
  return (
    <Tab.Group defaultIndex={defaultIndex} onChange={onChange}>
      <Tab.List className="flex gap-2 border-b border-[var(--color-border)]">
        {tabs.map((t) => (
          <Tab key={t.id} as={Fragment}>
            {({ selected }) => (
              <button
                className={[
                  'px-3.5 py-2 text-sm rounded-t-md',
                  'focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)]',
                  selected
                    ? 'bg-[var(--color-background)] text-[var(--color-text-primary)] border border-b-transparent border-[var(--color-border)]'
                    : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface)]'
                ].join(' ')}
              >
                {t.label}
              </button>
            )}
          </Tab>
        ))}
      </Tab.List>
      <Tab.Panels className="mt-3">
        {tabs.map((t) => (
          <Tab.Panel key={t.id} className="focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] focus-visible:ring-offset-2 rounded-md">
            {t.content}
          </Tab.Panel>
        ))}
      </Tab.Panels>
    </Tab.Group>
  )
}
