import { forwardRef } from 'react'

export type InputProps = React.InputHTMLAttributes<HTMLInputElement> & {
  label?: string
  helperText?: string
  error?: string
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input({ label, helperText, error, className, ...props }, ref) {
  const base = 'w-full px-3.5 py-2.5 rounded-md border bg-[var(--color-background)] focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[var(--color-primary)] transition-all duration-200'
  const normal = 'border-[var(--color-border)] text-[var(--color-text-primary)] hover:bg-[var(--color-surface)]'
  const err = 'border-[var(--color-error-500)] focus:ring-[var(--color-error-500)]'
  return (
    <label className="block">
      {label && <div className="mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">{label}</div>}
      <input ref={ref} className={[base, error ? err : normal, className || ''].join(' ')} {...props} />
      {error ? (
        <div className="mt-1 text-xs text-[var(--color-error-500)]">{error}</div>
      ) : (
        helperText && <div className="mt-1 text-xs text-[var(--color-text-tertiary)]">{helperText}</div>
      )}
    </label>
  )
})
