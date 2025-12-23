import { useState, useEffect, useMemo, useRef } from "react";
import { FileCode } from "lucide-react";
import { Button, SearchBar, type SearchItem } from "@sruja/ui";
import { getAllExamples, fetchExampleDsl } from "../../examples";
import { convertDslToLikeC4 } from "../../wasm";
import { useArchitectureStore } from "../../stores";
import type { SrujaModelDump, Example } from "@sruja/shared";

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
      .then((all) => {
        // Show both DSL and JSON examples (filter out only if explicitly skipped)
        setExamples(all)
      })
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
      let content: string;
      try {
        content = await fetchExampleDsl(example.file);
      } catch (fetchErr) {
        throw new Error(`Failed to fetch example file "${example.file}": ${fetchErr instanceof Error ? fetchErr.message : "Unknown error"}`);
      }

      if (example.isDsl) {
        // Parse DSL to Model
        const data = await convertDslToLikeC4(content, example.file);
        if (!data) throw new Error("Failed to parse DSL to Model");

        loadFromDSL(data as SrujaModelDump, content, example.file);
      } else {
        // Legacy JSON path - try to migrate or error
        // For now assuming all examples are DSL or will fail gracefully
        throw new Error("JSON examples are deprecated. Please use DSL examples.");
      }

      // Update URL with example parameter
      const url = new URL(window.location.href);
      url.searchParams.set("example", example.file);
      url.searchParams.delete("share");
      url.searchParams.delete("dsl");
      url.searchParams.delete("code");
      window.history.pushState({}, "", url.toString());
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
        style={{ paddingLeft: "8px", paddingRight: "8px", minWidth: "36px" }}
      >
        <FileCode size={18} />
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
                <Button
                  key={example.file}
                  variant="ghost"
                  size="sm"
                  className="example-item"
                  onClick={() => handleExampleSelect(example)}
                  style={{
                    width: "100%",
                    padding: "10px 8px",
                    textAlign: "left",
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
                </Button>
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
