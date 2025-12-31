import { useEffect, useRef } from "react";
import { Combobox as HCombobox } from "@headlessui/react";

export type SearchItem = {
  id: string;
  label: string;
  subLabel?: string;
};

export type SearchBarProps = {
  query: string;
  onQueryChange: (q: string) => void;
  results: SearchItem[];
  loading?: boolean;
  onSelect: (item: SearchItem | null) => void;
  placeholder?: string;
  className?: string;
  badge?: React.ReactNode;
};

export function SearchBar({
  query,
  onQueryChange,
  results,
  loading,
  onSelect,
  placeholder = "Search…",
  className,
  badge,
}: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  useEffect(() => {
    // optional: autofocus behavior when the component mounts
    inputRef.current?.focus();
  }, []);
  return (
    <HCombobox value={null} onChange={onSelect} nullable>
      <div className={["relative", className || ""].join(" ")}>
        <div className="flex items-center gap-2 w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]">
          <span className="text-[var(--color-text-tertiary)]">⌘K</span>
          <HCombobox.Input
            ref={inputRef as unknown as React.RefObject<HTMLInputElement>}
            className="flex-1 bg-transparent outline-none text-[var(--color-text-primary)]"
            placeholder={placeholder}
            value={query}
            onChange={(e) => onQueryChange(e.target.value)}
            autoComplete="off"
          />
          {loading && (
            <span className="inline-block w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
          )}
          {badge && (
            <span className="ml-2 text-[11px] text-[var(--color-text-tertiary)] whitespace-nowrap">
              {badge}
            </span>
          )}
        </div>

        {query.trim() !== "" && (
          <HCombobox.Options className="absolute z-50 mt-2 w-full max-h-64 overflow-auto rounded-md border border-[var(--color-border)] bg-[var(--color-background)] shadow-sm">
            {results.length === 0 && (
              <div className="px-3.5 py-2 text-sm text-[var(--color-text-tertiary)]">
                No results
              </div>
            )}
            {results.map((item) => (
              <HCombobox.Option
                key={item.id}
                value={item}
                className={({ active }) =>
                  [
                    "px-3.5 py-2 text-sm cursor-pointer flex items-center justify-between",
                    active ? "bg-[var(--color-surface)]" : "",
                  ].join(" ")
                }
              >
                <span className="text-[var(--color-text-primary)]">{item.label}</span>
                {item.subLabel && (
                  <span className="text-[var(--color-text-tertiary)] text-xs">{item.subLabel}</span>
                )}
              </HCombobox.Option>
            ))}
          </HCombobox.Options>
        )}
      </div>
    </HCombobox>
  );
}
