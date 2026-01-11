import { useEffect, useRef, useState } from "react";
import { TextInput, Paper, ScrollArea } from "@mantine/core";
import { cn } from "../utils/cn";

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
  const [focusedIndex, setFocusedIndex] = useState<number>(-1);
  const resultsRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    // optional: autofocus behavior when the component mounts
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    // Reset focused index when results change
    setFocusedIndex(-1);
  }, [results]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (query.trim() === "" || results.length === 0) {
      if (e.key === "Escape") {
        inputRef.current?.blur();
      }
      return;
    }

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setFocusedIndex((prev) => (prev < results.length - 1 ? prev + 1 : prev));
        break;
      case "ArrowUp":
        e.preventDefault();
        setFocusedIndex((prev) => (prev > 0 ? prev - 1 : -1));
        break;
      case "Enter":
        e.preventDefault();
        if (focusedIndex >= 0 && focusedIndex < results.length) {
          onSelect(results[focusedIndex]);
        } else if (results.length > 0) {
          // Select first result if no item is focused
          onSelect(results[0]);
        }
        break;
      case "Escape":
        e.preventDefault();
        inputRef.current?.blur();
        setFocusedIndex(-1);
        break;
    }
  };

  const handleItemClick = (item: SearchItem) => {
    onSelect(item);
    inputRef.current?.blur();
  };

  const hasQuery = query.trim() !== "";

  return (
    <div className={cn("relative", className)}>
      <div className="flex items-center gap-2 w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]">
        <span className="text-[var(--color-text-tertiary)]">⌘K</span>
        <TextInput
          ref={inputRef}
          value={query}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => onQueryChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          autoComplete="off"
          classNames={{
            root: "flex-1",
            input:
              "bg-transparent border-none outline-none text-[var(--color-text-primary)] px-0 shadow-none focus:shadow-none",
            wrapper: "border-none",
          }}
          style={{ flex: 1 }}
        />
        {loading && (
          <span className="inline-block w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin text-[var(--color-text-tertiary)]" />
        )}
        {badge && (
          <span className="ml-2 text-[11px] text-[var(--color-text-tertiary)] whitespace-nowrap">
            {badge}
          </span>
        )}
      </div>

      {hasQuery && (
        <Paper
          ref={resultsRef}
          withBorder
          className="absolute z-50 mt-2 w-full max-h-64 overflow-hidden border border-[var(--color-border)] bg-[var(--color-background)] shadow-sm"
          style={{ top: "100%" }}
        >
          {results.length === 0 ? (
            <div className="px-3.5 py-2 text-sm text-[var(--color-text-tertiary)]">No results</div>
          ) : (
            <ScrollArea.Autosize mah={256}>
              {results.map((item, index) => {
                const isFocused = index === focusedIndex;
                return (
                  <div
                    key={item.id}
                    role="button"
                    tabIndex={0}
                    onClick={() => handleItemClick(item)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        handleItemClick(item);
                      }
                    }}
                    className={cn(
                      "px-3.5 py-2 text-sm cursor-pointer flex items-center justify-between transition-colors",
                      isFocused ? "bg-[var(--color-surface)]" : "hover:bg-[var(--color-surface)]"
                    )}
                    onMouseEnter={() => setFocusedIndex(index)}
                    onMouseLeave={() => setFocusedIndex(-1)}
                  >
                    <span className="text-[var(--color-text-primary)]">{item.label}</span>
                    {item.subLabel && (
                      <span className="text-[var(--color-text-tertiary)] text-xs">
                        {item.subLabel}
                      </span>
                    )}
                  </div>
                );
              })}
            </ScrollArea.Autosize>
          )}
        </Paper>
      )}
    </div>
  );
}
