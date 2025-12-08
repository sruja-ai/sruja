import { Transition } from '@headlessui/react'

export type CardProps = {
  title?: string
  subtitle?: string
  children: any
  footer?: any
  interactive?: boolean
  onClick?: () => void
  href?: string
  className?: string
}

export function Card({ title, subtitle, children, footer, interactive, onClick, href, className }: CardProps) {
  const base = 'bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm'
  const pad = 'p-5'
  const hover = (interactive || onClick || href) ? 'transition-shadow hover:shadow-md focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:ring-offset-2' : ''
  const content = (
    <div className={[base, hover, 'group', className || ''].join(' ')} role={(onClick || href) ? 'button' : undefined} tabIndex={(onClick || href) ? 0 : -1} aria-label={title || undefined} onClick={onClick}>
      {(title || subtitle) && (
        <div className="pb-4 mb-4 border-b border-[var(--color-border)]">
          {title && <div className="text-xl font-semibold text-[var(--color-text-primary)]">{title}</div>}
          {subtitle && <div className="mt-1 text-sm text-[var(--color-text-secondary)]">{subtitle}</div>}
        </div>
      )}
      <div className={pad}>{children}</div>
      {footer && <div className="mt-4 pt-4 border-t border-[var(--color-border)]">{footer}</div>}
    </div>
  )
  return (
    <Transition
      appear
      show
      enter="ease-out duration-200"
      enterFrom="opacity-0 translate-y-1"
      enterTo="opacity-100 translate-y-0"
    >
      {href ? <a href={href} className="no-underline">{content}</a> : content}
    </Transition>
  )
}
