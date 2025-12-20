// packages/ui/src/components/Textarea.tsx
import { forwardRef } from 'react'
import { Textarea as MantineTextarea } from '@mantine/core'
import type { TextareaProps as MantineTextareaProps } from '@mantine/core'

export type TextareaProps = Omit<MantineTextareaProps, 'error'> & {
  helperText?: string
  error?: string
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  function Textarea({ label, helperText, error, className, ...props }, ref) {
    return (
      <MantineTextarea
        ref={ref}
        label={label}
        description={helperText}
        error={error}
        className={className}
        {...props}
      />
    )
  }
)
