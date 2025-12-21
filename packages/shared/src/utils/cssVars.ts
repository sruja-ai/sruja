// packages/shared/src/utils/cssVars.ts
// CSS variable utilities for theme-aware color access

import { isSSR } from './env';

/**
 * Get a CSS custom property value from the document root.
 * 
 * @public
 * @param name - The CSS variable name (e.g., '--color-primary')
 * @returns The computed CSS variable value, or empty string if document is unavailable
 * 
 * @remarks
 * Safe to call in SSR environments - returns empty string if document is undefined.
 * 
 * @example
 * const primaryColor = getCssVar('--color-primary');
 */
export function getCssVar(name: string): string {
  if (isSSR()) {
    return "";
  }
  const val = getComputedStyle(document.documentElement).getPropertyValue(
    name
  );
  return val.trim();
}

/**
 * Color utility functions for accessing theme CSS variables.
 * 
 * @public
 * @remarks
 * All functions return the computed CSS variable value.
 * Returns empty string in SSR environments.
 * 
 * @example
 * const primary = Colors.primary();
 * const error = Colors.error();
 */
export const Colors = {
  /** Primary brand color */
  primary: (): string => getCssVar("--color-primary"),
  /** Primary color at 50% opacity */
  primary50: (): string => getCssVar("--color-primary-50"),
  /** Primary color hover state */
  primaryHover: (): string => getCssVar("--color-primary-hover"),
  /** Border color */
  border: (): string => getCssVar("--color-border"),
  /** Primary text color */
  textPrimary: (): string => getCssVar("--color-text-primary"),
  /** Secondary text color */
  textSecondary: (): string => getCssVar("--color-text-secondary"),
  /** Tertiary text color */
  textTertiary: (): string => getCssVar("--color-text-tertiary"),
  /** Background color */
  background: (): string => getCssVar("--color-background"),
  /** Surface color (e.g., cards, panels) */
  surface: (): string => getCssVar("--color-surface"),
  /** Success color (500 shade) */
  success: (): string => getCssVar("--color-success-500"),
  /** Error color (500 shade) */
  error: (): string => getCssVar("--color-error-500"),
  /** Info color (500 shade) */
  info: (): string => getCssVar("--color-info-500"),
  /** Neutral color (500 shade) */
  neutral500: (): string => getCssVar("--color-neutral-500"),
} as const;

