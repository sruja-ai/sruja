import type { SrujaModelDump, DotResult } from "@sruja/shared";
import { SrujaCanvasLite } from "./SrujaCanvasLite";

interface SrujaDiagramPreviewProps {
  model: SrujaModelDump;
  dotResult?: DotResult | null;
}

export function SrujaDiagramPreview({ model, dotResult }: SrujaDiagramPreviewProps) {
  const elements = model.elements ? Object.values(model.elements) : [];
  const elementMap = model.elements || {}; // Map of FQN -> Element
  const relations = model.relations || [];

  if (dotResult) {
    return (
      <div
        style={{
          height: "100%",
          width: "100%",
          background: "var(--color-background-secondary)",
        }}
      >
        <SrujaCanvasLite dotResult={dotResult} />
      </div>
    );
  }

  if (elements.length === 0) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: "var(--color-text-secondary)",
        }}
      >
        No elements to display
      </div>
    );
  }

  return (
    <div
      style={{
        padding: 16,
        height: "100%",
        overflow: "auto",
        background: "var(--color-background-secondary)",
      }}
    >
      <div style={{ marginBottom: 16 }}>
        <strong>Elements ({elements.length})</strong>
      </div>
      <div style={{ display: "grid", gap: 8 }}>
        {elements.map((elem) => (
          <div
            key={elem.id}
            style={{
              padding: 12,
              background: "var(--color-background)",
              border: "1px solid var(--color-border)",
              borderRadius: 6,
            }}
          >
            <div style={{ fontWeight: 600 }}>{elem.title || elem.id}</div>
            <div style={{ fontSize: 12, color: "var(--color-text-secondary)" }}>
              {elem.kind} {elem.technology ? `• ${elem.technology}` : ""}
            </div>
            {elem.description && (
              <div style={{ fontSize: 13, marginTop: 4 }}>
                {typeof elem.description === "string" ? elem.description : ""}
              </div>
            )}
          </div>
        ))}
      </div>
      {relations.length > 0 && (
        <>
          <div style={{ marginTop: 16, marginBottom: 8 }}>
            <strong>Relations ({relations.length})</strong>
          </div>
          <div style={{ display: "grid", gap: 4, fontSize: 13 }}>
            {relations.map((rel, idx) => {
              // Handle FqnRef objects (from SrujaRelationDump) or string FQNs
              const sourceFqn =
                typeof rel.source === "string"
                  ? rel.source
                  : (rel.source as { model?: string })?.model || String(rel.source);
              const targetFqn =
                typeof rel.target === "string"
                  ? rel.target
                  : (rel.target as { model?: string })?.model || String(rel.target);

              // Find elements by FQN (key in the Record) or by id
              const sourceElem = elementMap[sourceFqn] || elements.find((e) => e.id === sourceFqn);
              const targetElem = elementMap[targetFqn] || elements.find((e) => e.id === targetFqn);

              const sourceName = sourceElem?.title || sourceElem?.id || sourceFqn;
              const targetName = targetElem?.title || targetElem?.id || targetFqn;

              return (
                <div key={idx} style={{ color: "var(--color-text-secondary)" }}>
                  {sourceName} → {targetName}
                  {rel.title ? `: ${rel.title}` : ""}
                </div>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
