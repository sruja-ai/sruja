// apps/designer/src/components/Security/TrustBoundaries.tsx
import { useMemo, useState } from "react";
import { Shield, Globe, Lock } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import "./TrustBoundaries.css";

type TrustZone = "public" | "private" | "restricted";

interface ZoneComponent {
  id: string;
  name: string;
  zone: TrustZone;
}

export function TrustBoundaries() {
  const model = getArchitectureModel();
  const nodes = model.getNodes();
  const relations = model.getRelations();

  const [selectedZone, setSelectedZone] = useState<TrustZone | "all">("all");

  // Assign components to trust zones based on metadata or heuristics
  const zoneComponents = useMemo<ZoneComponent[]>(() => {
    const components: ZoneComponent[] = [];

    for (const [nodeId, node] of nodes.entries()) {
      const nodeData = node as any;

      // Determine zone from metadata, tags, or heuristics
      let zone: TrustZone = "private"; // Default

      // Check metadata for explicit zone assignment
      if (nodeData.metadata?.trustZone) {
        zone = nodeData.metadata.trustZone;
      } else if (nodeData.tags?.includes("public") || nodeData.tags?.includes("external")) {
        zone = "public";
      } else if (nodeData.tags?.includes("restricted") || nodeData.tags?.includes("admin")) {
        zone = "restricted";
      } else {
        // Heuristic: Check if component is public-facing
        const isPublicFacing = relations.some(
          (r: any) =>
            (r.target?.model === nodeId || r.source?.model === nodeId) &&
            (r.title?.toLowerCase().includes("api") || r.title?.toLowerCase().includes("http"))
        );

        if (isPublicFacing) {
          zone = "public";
        }
      }

      components.push({
        id: nodeId,
        name: nodeId,
        zone,
      });
    }

    return components;
  }, [nodes, relations]);

  const filteredComponents = useMemo(() => {
    if (selectedZone === "all") return zoneComponents;
    return zoneComponents.filter((c) => c.zone === selectedZone);
  }, [zoneComponents, selectedZone]);

  const zoneStats = useMemo(() => {
    return {
      public: zoneComponents.filter((c) => c.zone === "public").length,
      private: zoneComponents.filter((c) => c.zone === "private").length,
      restricted: zoneComponents.filter((c) => c.zone === "restricted").length,
    };
  }, [zoneComponents]);

  const getZoneColor = (zone: TrustZone) => {
    switch (zone) {
      case "public":
        return "#ef4444"; // Red
      case "private":
        return "#f59e0b"; // Yellow/Orange
      case "restricted":
        return "#22c55e"; // Green
    }
  };

  const getZoneIcon = (zone: TrustZone) => {
    switch (zone) {
      case "public":
        return <Globe size={16} />;
      case "private":
        return <Shield size={16} />;
      case "restricted":
        return <Lock size={16} />;
    }
  };

  if (zoneComponents.length === 0) {
    return (
      <div className="trust-boundaries">
        <div className="trust-boundaries-header">
          <h3 className="trust-boundaries-title">
            <Shield size={18} />
            Trust Boundaries
          </h3>
        </div>
        <div className="trust-boundaries-empty">
          <p>No components found.</p>
          <p className="trust-boundaries-empty-subtitle">
            Add components to visualize trust boundaries.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="trust-boundaries">
      <div className="trust-boundaries-header">
        <h3 className="trust-boundaries-title">
          <Shield size={18} />
          Trust Boundaries
        </h3>
        <div className="trust-boundaries-stats">
          <span className="zone-stat zone-public" style={{ color: getZoneColor("public") }}>
            {zoneStats.public} Public
          </span>
          <span className="zone-stat zone-private" style={{ color: getZoneColor("private") }}>
            {zoneStats.private} Private
          </span>
          <span className="zone-stat zone-restricted" style={{ color: getZoneColor("restricted") }}>
            {zoneStats.restricted} Restricted
          </span>
        </div>
      </div>

      <div className="trust-boundaries-filters">
        <button
          className={`zone-filter ${selectedZone === "all" ? "active" : ""}`}
          onClick={() => setSelectedZone("all")}
        >
          All Zones
        </button>
        <button
          className={`zone-filter ${selectedZone === "public" ? "active" : ""}`}
          onClick={() => setSelectedZone("public")}
          style={{ borderLeftColor: getZoneColor("public") }}
        >
          Public
        </button>
        <button
          className={`zone-filter ${selectedZone === "private" ? "active" : ""}`}
          onClick={() => setSelectedZone("private")}
          style={{ borderLeftColor: getZoneColor("private") }}
        >
          Private
        </button>
        <button
          className={`zone-filter ${selectedZone === "restricted" ? "active" : ""}`}
          onClick={() => setSelectedZone("restricted")}
          style={{ borderLeftColor: getZoneColor("restricted") }}
        >
          Restricted
        </button>
      </div>

      <div className="trust-boundaries-list">
        {filteredComponents.map((component) => (
          <div
            key={component.id}
            className="trust-boundary-item"
            style={{ borderLeftColor: getZoneColor(component.zone) }}
          >
            <div className="trust-boundary-item-header">
              <div
                className="trust-boundary-item-icon"
                style={{ color: getZoneColor(component.zone) }}
              >
                {getZoneIcon(component.zone)}
              </div>
              <div className="trust-boundary-item-content">
                <div className="trust-boundary-item-name">{component.name}</div>
                <div className="trust-boundary-item-zone">{component.zone}</div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
