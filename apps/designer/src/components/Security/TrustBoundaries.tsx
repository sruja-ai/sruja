// apps/designer/src/components/Security/TrustBoundaries.tsx
import { useMemo, useState } from "react";
import { Shield, Globe, Lock, Filter, XCircle } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import type { ElementDump, RelationDump } from "@sruja/shared";
import type { Priority } from "../../types";
import "./TrustBoundaries.css";

interface TrustViolation {
  from: string;
  to: string;
  fromZone: TrustZone;
  toZone: TrustZone;
  priority: Priority;
  actionable: string;
}

type TrustZone = "public" | "private" | "restricted";

interface ZoneComponent {
  id: string;
  name: string;
  zone: TrustZone;
}

export function TrustBoundaries() {
  const storeModel = useArchitectureStore((s) => s.model);
  const model = getArchitectureModel();
  const [selectedZone, setSelectedZone] = useState<TrustZone | "all">("all");
  const [showOnlyViolations, setShowOnlyViolations] = useState(true); // Show violations first

  // Memoize model data to prevent infinite loops
  const { nodes, relations } = useMemo(() => {
    const archModel = model.getModel();
    return {
      nodes: archModel?.elements ? new Map(Object.entries(archModel.elements)) : new Map(),
      relations: archModel?.relations || [],
    };
  }, [storeModel, model]);

  // Assign components to trust zones based on metadata or heuristics
  const zoneComponents = useMemo<ZoneComponent[]>(() => {
    const components: ZoneComponent[] = [];

    for (const [nodeId, node] of nodes.entries()) {
      const nodeData = node as ElementDump;

      // Determine zone from metadata, tags, or heuristics
      let zone: TrustZone = "private"; // Default

      // Check metadata for explicit zone assignment
      if (nodeData.metadata?.trustZone) {
        const trustZone = nodeData.metadata.trustZone;
        if (trustZone === "public" || trustZone === "private" || trustZone === "restricted") {
          zone = trustZone;
        }
      } else if (nodeData.tags?.includes("public") || nodeData.tags?.includes("external")) {
        zone = "public";
      } else if (nodeData.tags?.includes("restricted") || nodeData.tags?.includes("admin")) {
        zone = "restricted";
      } else {
        // Heuristic: Check if component is public-facing
        const isPublicFacing = relations.some(
          (r: RelationDump) =>
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

  // Find cross-boundary violations (public accessing restricted)
  const violations = useMemo<TrustViolation[]>(() => {
    const violationsList: TrustViolation[] = [];
    for (const rel of relations) {
      const r = rel as RelationDump;
      const fromId =
        typeof r.source === "string" ? r.source : (r.source as { model?: string })?.model;
      const toId =
        typeof r.target === "string" ? r.target : (r.target as { model?: string })?.model;
      if (!fromId || !toId) continue;

      const fromComp = zoneComponents.find((c) => c.id === fromId);
      const toComp = zoneComponents.find((c) => c.id === toId);
      if (!fromComp || !toComp) continue;

      // Violation: public accessing restricted (critical)
      if (fromComp.zone === "public" && toComp.zone === "restricted") {
        violationsList.push({
          from: fromId,
          to: toId,
          fromZone: fromComp.zone,
          toZone: toComp.zone,
          priority: "high",
          actionable: "Add API gateway or authentication layer between public and restricted zones",
        });
      }
      // Violation: public accessing private without proper security (medium)
      if (fromComp.zone === "public" && toComp.zone === "private") {
        violationsList.push({
          from: fromId,
          to: toId,
          fromZone: fromComp.zone,
          toZone: toComp.zone,
          priority: "medium",
          actionable: "Ensure proper authentication and authorization between zones",
        });
      }
    }
    return violationsList;
  }, [relations, zoneComponents]);

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
        <div className="trust-boundaries-title-row">
          <h3 className="trust-boundaries-title">
            <Shield size={18} />
            Trust Boundaries
          </h3>
          {violations.length > 0 && (
            <Badge color="error" className="priority-badge">
              {violations.length} Violations
            </Badge>
          )}
        </div>
        {violations.length > 0 && (
          <div className="trust-boundaries-filters">
            <Button
              variant={showOnlyViolations ? "primary" : "ghost"}
              size="sm"
              onClick={() => setShowOnlyViolations(!showOnlyViolations)}
            >
              <Filter size={12} />
              {showOnlyViolations ? "Showing Violations" : "Show All"}
            </Button>
          </div>
        )}
      </div>

      {violations.length > 0 && showOnlyViolations && (
        <div className="trust-violations-list">
          {violations.map((violation, idx) => (
            <div key={`${violation.from}-${violation.to}-${idx}`} className="trust-violation-item">
              <div className="trust-violation-header">
                <XCircle size={16} className="icon-critical" style={{ color: "#ef4444" }} />
                <div className="trust-violation-content">
                  <div className="trust-violation-title">
                    {violation.fromZone} → {violation.toZone} violation
                  </div>
                  <div className="trust-violation-details">
                    <strong>{violation.from}</strong> → <strong>{violation.to}</strong>
                  </div>
                  <div className="trust-violation-actionable">
                    <strong>Fix:</strong> {violation.actionable}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

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
