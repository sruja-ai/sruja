import { Radio as MantineRadio, RadioGroup as MantineRadioGroup } from "@mantine/core";

export type RadioOption = {
  /** Option value */
  value: string;
  /** Option label */
  label: string;
  /** Option description */
  description?: string;
};

export type RadioGroupProps = {
  /** Group name */
  name?: string;
  /** Selected value */
  value: string | null;
  /** Callback on change */
  onChange: (value: string | null) => void;
  /** Radio options */
  options: RadioOption[];
  /** Label for group */
  label?: string;
};

export function RadioGroup({ name, value, onChange, options, label }: RadioGroupProps) {
  return (
    <div>
      {label && (
        <div className="mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">{label}</div>
      )}
      <MantineRadioGroup value={value || ""} onChange={onChange}>
        <div className="space-y-2">
          {options.map((opt) => (
            <MantineRadio
              key={opt.value}
              name={name}
              value={opt.value}
              label={opt.label}
              description={opt.description}
              className="px-4 py-3 rounded-md border cursor-pointer transition-all"
              style={{
                borderColor: value === opt.value ? "var(--color-primary)" : "var(--color-border)",
                backgroundColor: value === opt.value ? "var(--color-primary-50)" : "transparent",
              }}
            />
          ))}
        </div>
      </MantineRadioGroup>
    </div>
  );
}
