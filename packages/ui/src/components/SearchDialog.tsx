import { useEffect, useState } from "react";
import { Modal } from "@mantine/core";
import { SearchBar, type SearchItem } from "./SearchBar";

export type SearchDialogProps = {
  isOpen: boolean;
  onClose: () => void;
  fetchResults: (q: string) => Promise<SearchItem[]>;
  onSelect: (item: SearchItem | null) => void;
  verticalOffset?: string;
  badge?: React.ReactNode;
};

export function SearchDialog({
  isOpen,
  onClose,
  fetchResults,
  onSelect,
  verticalOffset,
  badge,
}: SearchDialogProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchItem[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let alive = true;
    const run = async () => {
      if (query.trim() === "") {
        setResults([]);
        return;
      }
      setLoading(true);
      try {
        const r = await fetchResults(query);
        if (alive) setResults(r);
      } finally {
        if (alive) setLoading(false);
      }
    };
    run();
    return () => {
      alive = false;
    };
  }, [query, fetchResults]);

  return (
    <Modal
      opened={isOpen}
      onClose={onClose}
      size="lg"
      classNames={{
        content: "bg-[var(--color-background)]",
        header: "border-b border-[var(--color-border)]",
        title: "text-[var(--color-text-primary)]",
        body: "max-h-[400px] overflow-auto",
      }}
      style={{
        top: verticalOffset || undefined,
      }}
    >
      <SearchBar
        query={query}
        onQueryChange={setQuery}
        results={results}
        loading={loading}
        onSelect={(item) => {
          onSelect(item);
          onClose();
        }}
        badge={badge}
      />
    </Modal>
  );
}
