import { useState, useEffect, useMemo, useRef } from "react";
import { FileCode } from "lucide-react";
import { Button, SearchBar, type SearchItem } from "@sruja/ui";
import { getAllExamples, fetchExampleDsl } from "../../examples";
import { convertDslToJson } from "../../wasm";
import { convertJsonToDsl } from "../../utils/jsonToDsl";
import { useArchitectureStore } from "../../stores";
import type { ArchitectureJSON } from "../../types";
import type { Example } from "@sruja/shared";

export function ExamplesDropdown() {
  const [isOpen, setIsOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [examples, setExamples] = useState<Array<Example & { isDsl: boolean }>>([]);
  const [query, setQuery] = useState("");
  const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);
  const panelRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    getAllExamples()
      .then((all) => setExamples(all.filter((e) => e.isDsl)))
      .catch((err) => {
        console.error("Failed to load examples:", err);
        setError("Failed to load examples");
      });
  }, []);

  const itemsIndex = useMemo(() => {
    const byId = new Map<string, Example & { isDsl: boolean }>();
    for (const ex of examples) byId.set(ex.file, ex);
    return byId;
  }, [examples]);

  const filteredExamples = useMemo(() => {
    const q = query.toLowerCase().trim();
    const list = q
      ? examples.filter(
          (ex) =>
            ex.name.toLowerCase().includes(q) ||
            ex.description.toLowerCase().includes(q) ||
            ex.category.toLowerCase().includes(q)
        )
      : examples;
    return list.sort((a, b) => a.order - b.order);
  }, [examples, query]);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (!panelRef.current) return;
      const target = e.target as Node;
      if (isOpen && target && !panelRef.current.contains(target)) {
        setIsOpen(false);
      }
    };
    document.addEventListener("click", onDocClick, true);
    return () => document.removeEventListener("click", onDocClick, true);
  }, [isOpen]);

  const handleExampleSelect = async (example: Example & { isDsl: boolean }) => {
    setIsOpen(false);
    setLoading(true);
    setError(null);

    try {
      if (example.isDsl) {
        const dsl = await fetchExampleDsl(example.file);
        const converted = await convertDslToJson(dsl);
        if (!converted) throw new Error("Failed to parse DSL");
        loadFromDSL(converted as ArchitectureJSON, dsl, example.file);
      } else {
        const jsonText = await fetchExampleDsl(example.file);
        const parsed = JSON.parse(jsonText) as ArchitectureJSON;
        const dsl = convertJsonToDsl(parsed);
        loadFromDSL(parsed, dsl, example.file);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : "Unknown error";
      setError(`Failed to load example: ${message}`);
      console.error("Example load error:", err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ position: "relative", display: "inline-block" }}>
      <Button
        variant="ghost"
        size="md"
        onClick={() => setIsOpen((v) => !v)}
        disabled={loading}
        aria-label={loading ? "Loading examples..." : "Examples"}
        title={loading ? "Loading examples..." : "Examples"}
      >
        <FileCode size={18} />
        <span>Examples</span>
      </Button>

      {isOpen && (
        <div
          ref={panelRef}
          className="examples-menu"
          style={{
            position: "absolute",
            top: "calc(100% + 4px)",
            right: 0,
            width: 420,
            maxHeight: 520,
            overflowY: "auto",
            background: "var(--color-background)",
            border: "1px solid var(--color-border)",
            borderRadius: 8,
            boxShadow: "0 4px 20px rgba(0,0,0,0.15)",
            zIndex: 10000,
            padding: 12,
          }}
        >
          <SearchBar
            query={query}
            onQueryChange={setQuery}
            results={filteredExamples.slice(0, 0).map((ex) => ({ id: ex.file, label: ex.name }))}
            onSelect={(item: SearchItem | null) => {
              if (!item) return;
              const ex = itemsIndex.get(item.id);
              if (ex) void handleExampleSelect(ex);
            }}
            placeholder="Filter examples"
          />

          {filteredExamples.length === 0 && (
            <div
              className="examples-loading"
              style={{ padding: 12, color: "var(--color-text-tertiary)", fontSize: 13 }}
            >
              No results
            </div>
          )}

          {Object.entries(
            filteredExamples.reduce(
              (acc, ex) => {
                (acc[ex.category] ||= []).push(ex);
                return acc;
              },
              {} as Record<string, Array<Example & { isDsl: boolean }>>
            )
          ).map(([category, list]) => (
            <div key={category}>
              <div
                className="example-category"
                style={{
                  padding: "8px 6px",
                  fontSize: 11,
                  fontWeight: 600,
                  textTransform: "uppercase",
                  letterSpacing: 0.5,
                  color: "var(--color-text-secondary)",
                }}
              >
                {category}
              </div>
              {list.map((example) => (
                <button
                  key={example.file}
                  className="example-item"
                  onClick={() => handleExampleSelect(example)}
                  style={{
                    width: "100%",
                    padding: "10px 8px",
                    background: "transparent",
                    border: "none",
                    textAlign: "left",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "flex-start",
                    justifyContent: "space-between",
                    gap: 8,
                    color: "var(--color-text-primary)",
                  }}
                >
                  <div
                    className="example-item-content"
                    style={{
                      display: "flex",
                      flexDirection: "column",
                      gap: 4,
                      flex: 1,
                      minWidth: 0,
                    }}
                  >
                    <span className="example-name" style={{ fontSize: 14, fontWeight: 500 }}>
                      {example.name}
                    </span>
                    <span
                      className="example-desc"
                      style={{ fontSize: 12, color: "var(--color-text-secondary)" }}
                    >
                      {example.description}
                    </span>
                  </div>
                  {example.isDsl && (
                    <span
                      className="example-badge"
                      style={{ fontSize: 10, fontWeight: 600, color: "var(--color-text-tertiary)" }}
                    >
                      DSL
                    </span>
                  )}
                </button>
              ))}
            </div>
          ))}

          {error && (
            <div
              className="examples-error"
              style={{ marginTop: 8, fontSize: 12, color: "var(--color-error-500)" }}
            >
              {error}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
