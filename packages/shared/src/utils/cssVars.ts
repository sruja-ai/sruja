export function getCssVar(name: string): string {
  if (typeof document === 'undefined') return ''
  const val = getComputedStyle(document.documentElement).getPropertyValue(name)
  return val.trim()
}

export const Colors = {
  primary: () => getCssVar('--color-primary'),
  primary50: () => getCssVar('--color-primary-50'),
  primaryHover: () => getCssVar('--color-primary-hover'),
  border: () => getCssVar('--color-border'),
  textPrimary: () => getCssVar('--color-text-primary'),
  textSecondary: () => getCssVar('--color-text-secondary'),
  textTertiary: () => getCssVar('--color-text-tertiary'),
  background: () => getCssVar('--color-background'),
  surface: () => getCssVar('--color-surface'),
  success: () => getCssVar('--color-success-500'),
  error: () => getCssVar('--color-error-500'),
  info: () => getCssVar('--color-info-500'),
  neutral500: () => getCssVar('--color-neutral-500'),
}

