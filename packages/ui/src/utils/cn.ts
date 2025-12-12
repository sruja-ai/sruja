// packages/ui/src/utils/cn.ts
// Utility for merging Tailwind classes
import { twMerge } from 'tailwind-merge';

export function cn(...classes: (string | undefined | null | false)[]): string {
  return twMerge(classes.filter(Boolean).join(' '));
}























