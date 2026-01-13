// apps/designer/src/components/shared/forms/FormFields.tsx
// Reusable form field components to reduce duplication across Edit*Form components

import { Input, Textarea, Checkbox, Select } from "@sruja/ui";

interface NameFieldProps {
  label?: string;
  value: string;
  onChange: (value: string) => void;
  error?: string;
  placeholder?: string;
  required?: boolean;
  "data-testid"?: string;
}

export function NameField({
  label = "Name",
  value,
  onChange,
  error,
  placeholder = "Enter name",
  required = true,
  "data-testid": testId,
}: NameFieldProps) {
  return (
    <Input
      label={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      error={error}
      required={required}
      data-testid={testId}
    />
  );
}

interface DescriptionFieldProps {
  label?: string;
  value: string;
  onChange: (value: string) => void;
  error?: string;
  placeholder?: string;
  rows?: number;
  "data-testid"?: string;
}

export function DescriptionField({
  label = "Description",
  value,
  onChange,
  error,
  placeholder = "Enter description",
  rows = 3,
  "data-testid": testId,
}: DescriptionFieldProps) {
  return (
    <Textarea
      label={label}
      value={value}
      onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => onChange(e.target.value)}
      placeholder={placeholder}
      rows={rows}
      error={error}
      data-testid={testId}
    />
  );
}

interface TechnologyFieldProps {
  label?: string;
  value: string;
  onChange: (value: string) => void;
  error?: string;
  placeholder?: string;
  "data-testid"?: string;
}

export function TechnologyField({
  label = "Technology",
  value,
  onChange,
  error,
  placeholder = "e.g., Node.js, PostgreSQL",
  "data-testid": testId,
}: TechnologyFieldProps) {
  return (
    <Input
      label={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      error={error}
      data-testid={testId}
    />
  );
}

interface CustomIdFieldOptions {
  checkboxLabel?: string;
  inputLabel?: string;
  placeholder?: string;
  "data-testid"?: {
    checkbox?: string;
    input?: string;
  };
}

interface CustomIdFieldProps {
  useCustomId: boolean;
  onUseCustomIdChange: (checked: boolean) => void;
  idValue: string;
  onIdChange: (value: string) => void;
  error?: string;
  options?: CustomIdFieldOptions;
}

export function CustomIdField({
  useCustomId,
  onUseCustomIdChange,
  idValue,
  onIdChange,
  error,
  options = {},
}: CustomIdFieldProps) {
  const {
    checkboxLabel = "Set custom ID (optional)",
    inputLabel = "ID",
    placeholder = "If empty, ID is auto-generated from name",
    "data-testid": testId,
  } = options;
  return (
    <>
      <div className="form-group checkbox-row">
        <Checkbox
          id="custom-id-checkbox"
          label={checkboxLabel}
          checked={useCustomId}
          onChange={(e) => onUseCustomIdChange(e.currentTarget.checked)}
          data-testid={testId?.checkbox}
        />
      </div>
      {useCustomId && (
        <Input
          label={inputLabel}
          value={idValue}
          onChange={(e) => onIdChange(e.target.value)}
          placeholder={placeholder}
          error={error}
          data-testid={testId?.input}
        />
      )}
    </>
  );
}

// Re-export the options type for convenience
export type { CustomIdFieldOptions };

interface ParentSelectFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: Array<{ value: string; label: string }>;
  error?: string;
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  "data-testid"?: string;
}

export function ParentSelectField({
  label,
  value,
  onChange,
  options,
  error,
  placeholder = "Select...",
  required = false,
  disabled = false,
  "data-testid": testId,
}: ParentSelectFieldProps) {
  return (
    <Select
      label={label}
      value={value}
      onChange={(val) => onChange(val || "")}
      data={options}
      placeholder={placeholder}
      error={error}
      required={required}
      disabled={disabled}
      data-testid={testId}
    />
  );
}
