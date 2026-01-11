import { useState } from "react";
import { TextInput, Paper, ScrollArea } from "@mantine/core";

export type ComboOption = { id: string; label: string };

export type ComboboxProps = {
  options: ComboOption[];
  value?: ComboOption | null;
  onChange?: (value: ComboOption | null) => void;
  placeholder?: string;
};

export function Combobox({ options, onChange, placeholder }: ComboboxProps) {
  const [query, setQuery] = useState("");
  const filtered =
    query === ""
      ? options
      : options.filter((o) => o.label.toLowerCase().includes(query.toLowerCase()));

  return (
    <div className="relative">
      <TextInput
        placeholder={placeholder}
        value={query}
        onChange={(e) => setQuery(e.currentTarget.value)}
        className="w-full"
        classNames={{
          input:
            "px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]",
        }}
      />
      {query && (
        <Paper
          withBorder
          className="absolute z-50 mt-2 w-full max-h-[208px] overflow-auto"
          shadow="md"
        >
          <ScrollArea.Autosize mah={208}>
            {filtered.length === 0 ? (
              <div className="px-3.5 py-2 text-sm text-[var(--color-text-tertiary)]">
                No results
              </div>
            ) : (
              filtered.map((opt) => (
                <ActionItem
                  key={opt.id}
                  onClick={() => {
                    onChange?.(opt);
                    setQuery(opt.label);
                  }}
                >
                  {opt.label}
                </ActionItem>
              ))
            )}
          </ScrollArea.Autosize>
        </Paper>
      )}
    </div>
  );
}

function ActionItem({ onClick, children }: { onClick: () => void; children: React.ReactNode }) {
  return (
    <div
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick();
        }
      }}
      className="px-3.5 py-2 text-sm hover:bg-[var(--color-surface)] cursor-pointer transition-colors"
    >
      {children}
    </div>
  );
}
