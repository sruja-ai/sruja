// apps/website/src/features/playground/components/SrujaDiagramPreview.tsx
// Simple diagram preview component for website playground
import type { SrujaModelDump } from "@sruja/shared";

interface SrujaDiagramPreviewProps {
  model: SrujaModelDump;
}

export function SrujaDiagramPreview({ model }: SrujaDiagramPreviewProps) {
  const elements = model.elements ? Object.values(model.elements) : [];
  const relations = model.relations || [];

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
            {relations.map((rel, idx) => (
              <div key={idx} style={{ color: "var(--color-text-secondary)" }}>
                {String(rel.source)} → {String(rel.target)}
                {rel.title ? `: ${rel.title}` : ""}
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
