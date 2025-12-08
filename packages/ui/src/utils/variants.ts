import { twMerge } from 'tailwind-merge'

export function vx(...classes: Array<string | undefined | false>) {
  return twMerge(classes.filter(Boolean).join(' '))
}

