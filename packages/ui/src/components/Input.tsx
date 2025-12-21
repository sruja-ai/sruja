// packages/ui/src/components/Input.tsx
import { forwardRef } from 'react'
import { TextInput } from '@mantine/core'
import type { TextInputProps } from '@mantine/core'

export type InputProps = Omit<TextInputProps, 'error'> & {
  helperText?: string
  error?: string
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input({ label, helperText, error, className, ...props }, ref) {
  return (
    <TextInput
      ref={ref}
      label={label}
      description={helperText}
      error={error}
      className={className}
      {...props}
    />
  )
})
