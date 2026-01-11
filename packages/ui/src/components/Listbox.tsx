import { Select as MantineSelect } from "@mantine/core";

export type ListOption = { id: string; label: string };

export type ListboxProps = {
  options: ListOption[];
  value: ListOption | null;
  onChange: (val: ListOption | null) => void;
  label?: string;
};

export function Listbox({ options, value, onChange, label }: ListboxProps) {
  return (
    <div>
      {label && (
        <div className="mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">{label}</div>
      )}
      <MantineSelect
        value={value?.id || ""}
        onChange={(id) => {
          const option = options.find((opt) => opt.id === id);
          onChange(option || null);
        }}
        data={options.map((opt) => ({ value: opt.id, label: opt.label }))}
        placeholder="Select..."
        classNames={{
          input:
            "px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)] text-left",
        }}
      />
    </div>
  );
}
