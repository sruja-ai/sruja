export type BadgeProps = {
  children: any
  color?: 'neutral' | 'brand' | 'success' | 'error' | 'warning' | 'info'
  className?: string
}

export function Badge({ children, color = 'neutral', className }: BadgeProps) {
  const map: Record<string, string> = {
    neutral: 'bg-[var(--color-neutral-100)] text-[var(--color-text-secondary)]',
    brand: 'bg-[var(--color-primary-50)] text-[var(--color-primary-600)]',
    success: 'bg-[#dcfce7] text-[#166534]',
    error: 'bg-[#fee2e2] text-[#991b1b]',
    warning: 'bg-[#fef3c7] text-[#92400e]',
    info: 'bg-[#dbeafe] text-[#1e40af]'
  }
  const base = 'inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium'
  return <span className={[base, map[color], className || ''].join(' ')}>{children}</span>
}

