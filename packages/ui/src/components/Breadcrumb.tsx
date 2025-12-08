// packages/ui/src/components/Breadcrumb.tsx
import { ReactNode } from 'react'
import { ChevronRight, Home } from 'lucide-react'
import { vx } from '../utils/variants'

export interface BreadcrumbItem {
  /** Unique identifier for the breadcrumb item */
  id: string
  /** Display label */
  label: string
}

export interface BreadcrumbProps {
  /** Array of breadcrumb items */
  items: BreadcrumbItem[]
  /** Callback fired when a breadcrumb item is clicked */
  onItemClick: (id: string) => void
  /** Callback fired when home/root is clicked */
  onHomeClick?: () => void
  /** Custom home icon */
  homeIcon?: ReactNode
  /** Custom separator between items */
  separator?: ReactNode
  /** Whether to show home button */
  showHome?: boolean
  /** Additional CSS classes */
  className?: string
}

export function Breadcrumb({
  items,
  onItemClick,
  onHomeClick,
  homeIcon,
  separator,
  showHome = true,
  className = '',
}: BreadcrumbProps) {
  const baseClasses = 'flex items-center gap-1 text-sm'
  const itemClasses = 'px-2 py-1 rounded-md transition-colors cursor-pointer text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-surface)]'
  const activeItemClasses = 'text-[var(--color-text-primary)] font-medium pointer-events-none'
  const separatorClasses = 'text-[var(--color-text-tertiary)]'

  return (
    <nav className={vx(baseClasses, className)} aria-label="Breadcrumb">
      {showHome && (
        <>
          <button
            onClick={onHomeClick || (() => onItemClick('root'))}
            className={vx(itemClasses, 'flex items-center')}
            aria-label="Home"
          >
            {homeIcon || <Home size={16} />}
          </button>
          {items.length > 0 && (
            <span className={separatorClasses}>
              {separator || <ChevronRight size={14} />}
            </span>
          )}
        </>
      )}
      {items.map((item, index) => {
        const isLast = index === items.length - 1
        return (
          <div key={item.id} className="flex items-center gap-1">
            <button
              onClick={() => onItemClick(item.id)}
              className={vx(itemClasses, isLast && activeItemClasses)}
              aria-current={isLast ? 'page' : undefined}
            >
              {item.label}
            </button>
            {!isLast && (
              <span className={separatorClasses}>
                {separator || <ChevronRight size={14} />}
              </span>
            )}
          </div>
        )
      })}
    </nav>
  )
}

