import { Disclosure as HDisclosure } from '@headlessui/react'

export type DisclosureProps = {
  title: string
  children: any
  defaultOpen?: boolean
}

export function Disclosure({ title, children, defaultOpen }: DisclosureProps) {
  return (
    <HDisclosure defaultOpen={defaultOpen}>
      {() => (
        <div className="border border-[var(--color-border)] rounded-md">
          <HDisclosure.Button className="w-full text-left px-3.5 py-2.5 bg-[var(--color-background)]">
            <span className="text-[var(--color-text-primary)] font-medium">{title}</span>
          </HDisclosure.Button>
          <HDisclosure.Panel className="px-3.5 py-2.5 bg-[var(--color-surface)]">
            {children}
          </HDisclosure.Panel>
        </div>
      )}
    </HDisclosure>
  )
}

