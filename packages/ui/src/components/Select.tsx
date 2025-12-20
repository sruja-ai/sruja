// packages/ui/src/components/Select.tsx
import { forwardRef } from 'react'
import { Select as MantineSelect } from '@mantine/core'
import type { SelectProps as MantineSelectProps } from '@mantine/core'

export type SelectProps = Omit<MantineSelectProps, 'data'> & {
  data: Array<{ value: string; label: string }>
}

export const Select = forwardRef<HTMLInputElement, SelectProps>(function Select(props, ref) {
  return <MantineSelect ref={ref} {...props} />
})
