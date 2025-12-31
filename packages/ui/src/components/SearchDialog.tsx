import { useEffect, useState } from "react";
import { Dialog, DialogPanel } from "@headlessui/react";
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
    <Dialog open={isOpen} onClose={onClose} className="relative z-[9999]">
      <div
        className="fixed inset-0 bg-black/40"
        style={{
          zIndex: 9998,
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
        }}
      />
      <div
        className="fixed inset-0 overflow-y-auto"
        style={{
          zIndex: 9999,
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          display: "flex",
          alignItems: verticalOffset ? "flex-start" : "center",
          justifyContent: "center",
          padding: "1rem",
          paddingTop: verticalOffset || undefined,
        }}
      >
        <DialogPanel
          className="w-full max-w-xl"
          style={{
            position: "relative",
            zIndex: 10000,
            margin: "0 auto",
            width: "100%",
            maxWidth: "42rem",
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
        </DialogPanel>
      </div>
    </Dialog>
  );
}
