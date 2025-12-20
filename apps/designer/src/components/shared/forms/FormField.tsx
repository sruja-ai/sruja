// apps/designer/src/components/shared/forms/FormField.tsx
// Reusable form field component using Mantine components

import { Input, Textarea } from "@sruja/ui";
import type { InputChangeEvent, TextareaChangeEvent } from "../../../types/formHandlers";

export interface FormFieldProps {
  label: string;
  name: string;
  value: string;
  onChange: (value: string) => void;
  type?: "text" | "textarea" | "email" | "url";
  placeholder?: string;
  required?: boolean;
  error?: string;
  rows?: number;
  disabled?: boolean;
  description?: string;
}

/**
 * Reusable form field component that wraps Mantine Input/Textarea.
 * 
 * Provides consistent styling, error handling, and accessibility.
 * Supports text, email, url, and textarea input types.
 * 
 * @param props - Form field configuration
 * @param props.label - Field label text
 * @param props.name - Field name (for form identification)
 * @param props.value - Current field value
 * @param props.onChange - Change handler function
 * @param props.type - Input type: "text", "email", "url", or "textarea" (default: "text")
 * @param props.placeholder - Placeholder text
 * @param props.required - Whether field is required
 * @param props.error - Error message to display
 * @param props.rows - Number of rows for textarea (default: 4)
 * @param props.disabled - Whether field is disabled
 * @param props.description - Helper text displayed below field
 * @returns Form field component
 * 
 * @example
 * ```tsx
 * <FormField
 *   label="Email"
 *   name="email"
 *   value={values.email}
 *   onChange={(value) => setValue("email", value)}
 *   type="email"
 *   required
 *   error={errors.email}
 *   description="We'll never share your email"
 * />
 * ```
 */
export function FormField({
  label,
  name,
  value,
  onChange,
  type = "text",
  placeholder,
  required = false,
  error,
  rows,
  disabled = false,
  description,
}: FormFieldProps) {
  if (type === "textarea") {
    return (
      <Textarea
        label={label}
        name={name}
        value={value}
        onChange={(e: TextareaChangeEvent) => onChange(e.target.value)}
        placeholder={placeholder}
        required={required}
        error={error}
        rows={rows}
        disabled={disabled}
        helperText={description}
      />
    );
  }

  return (
    <Input
      label={label}
      name={name}
      type={type}
      value={value}
      onChange={(e: InputChangeEvent) => onChange(e.target.value)}
      placeholder={placeholder}
      required={required}
      error={error}
      disabled={disabled}
      helperText={description}
    />
  );
}
