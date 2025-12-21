// packages/ui/src/components/Checkbox.tsx
import { forwardRef } from 'react'
import { Checkbox as MantineCheckbox } from '@mantine/core'
import type { CheckboxProps as MantineCheckboxProps } from '@mantine/core'

export type CheckboxProps = MantineCheckboxProps

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(function Checkbox(props, ref) {
  return <MantineCheckbox ref={ref} {...props} />
})
