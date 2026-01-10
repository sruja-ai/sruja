// apps/designer/src/components/Security/DataFlowScanner.tsx
import { useMemo, useState } from "react";
import { Search, Lock, Unlock, AlertTriangle, Filter } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import type { Priority } from "../../types";
import "./DataFlowScanner.css";

type DataType = "PII" | "financial" | "health" | "other";

interface DataFlow {
  from: string;
  to: string;
  dataTypes: DataType[];
  encrypted: boolean;
  protocol?: string;
  priority: Priority; // High if sensitive data unencrypted, medium if encrypted, low if other
  actionable?: string; // One-line fix
}

// Helper interface for properties that might exist on elements
interface ElementWithDetails {
  metadata?: { encrypted?: boolean } & Record<string, unknown>;
  tags?: string[];
  technology?: string;
  description?: string;
}

// Helper to extract ID from relation source/target
const getRelationEndpointId = (
  endpoint: string | { model: string } | undefined
): string | undefined => {
  if (!endpoint) return undefined;
  if (typeof endpoint === "string") return endpoint;
  return endpoint.model;
};

export function DataFlowScanner() {
  const storeModel = useArchitectureStore((s) => s.model);
  const model = getArchitectureModel();
  const [showOnlyHigh, setShowOnlyHigh] = useState(true);

  // Memoize model data to prevent infinite loops
  const { nodes, relations } = useMemo(() => {
    const archModel = model.getModel();
    return {
      nodes: archModel?.elements ? new Map(Object.entries(archModel.elements)) : new Map(),
      relations: archModel?.relations || [],
    };
  }, [storeModel, model]);

  // Scan data flows for sensitive data
  const dataFlows = useMemo<DataFlow[]>(() => {
    const flows: DataFlow[] = [];

    for (const rel of relations) {
      const r = rel as unknown as {
        source: string | { model: string };
        target: string | { model: string };
        title?: string;
      };
      const source = getRelationEndpointId(r.source);
      const target = getRelationEndpointId(r.target);
      const title = r.title || "";

      if (!source || !target) continue;

      const sourceNode = nodes.get(source);
      const targetNode = nodes.get(target);

      if (!sourceNode || !targetNode) continue;

      const sourceData = sourceNode as ElementWithDetails;
      const targetData = targetNode as ElementWithDetails;

      // Detect data types from metadata, tags, or component names
      const dataTypes: DataType[] = [];

      // Check for PII indicators
      if (
        (sourceData.tags || []).some((t: string) => t.toLowerCase().includes("pii")) ||
        (targetData.tags || []).some((t: string) => t.toLowerCase().includes("pii")) ||
        source.toLowerCase().includes("user") ||
        target.toLowerCase().includes("user")
      ) {
        dataTypes.push("PII");
      }

      // Check for financial data
      if (
        (sourceData.tags || []).some((t: string) => t.toLowerCase().includes("payment")) ||
        (targetData.tags || []).some((t: string) => t.toLowerCase().includes("payment")) ||
        source.toLowerCase().includes("payment") ||
        target.toLowerCase().includes("payment")
      ) {
        dataTypes.push("financial");
      }

      // Check for health data
      if (
        (sourceData.tags || []).some((t: string) => t.toLowerCase().includes("health")) ||
        (targetData.tags || []).some((t: string) => t.toLowerCase().includes("health")) ||
        source.toLowerCase().includes("health") ||
        target.toLowerCase().includes("health")
      ) {
        dataTypes.push("health");
      }

      // If no specific data types detected, mark as "other" if it's a data flow
      if (
        dataTypes.length === 0 &&
        (title.toLowerCase().includes("data") || title.toLowerCase().includes("send"))
      ) {
        dataTypes.push("other");
      }

      // Check if flow is encrypted
      const isEncrypted =
        title.toLowerCase().includes("https") ||
        title.toLowerCase().includes("tls") ||
        title.toLowerCase().includes("ssl") ||
        sourceData.metadata?.encrypted ||
        targetData.metadata?.encrypted;

      if (dataTypes.length > 0) {
        // Determine priority: High if sensitive data is unencrypted, Medium if encrypted, Low for other
        const hasSensitiveData = dataTypes.some(
          (t) => t === "PII" || t === "financial" || t === "health"
        );
        const priority: Priority =
          hasSensitiveData && !isEncrypted ? "high" : hasSensitiveData ? "medium" : "low";
        const actionable =
          hasSensitiveData && !isEncrypted
            ? "Enable TLS/HTTPS encryption for this flow"
            : undefined;

        flows.push({
          from: source,
          to: target,
          dataTypes,
          encrypted: !!isEncrypted,
          protocol: title,
          priority,
          actionable,
        });
      }
    }

    return flows;
  }, [relations, nodes]);

  const flowStats = useMemo(() => {
    return {
      total: dataFlows.length,
      encrypted: dataFlows.filter((f) => f.encrypted).length,
      unencrypted: dataFlows.filter((f) => !f.encrypted).length,
      pii: dataFlows.filter((f) => f.dataTypes.includes("PII")).length,
      financial: dataFlows.filter((f) => f.dataTypes.includes("financial")).length,
      health: dataFlows.filter((f) => f.dataTypes.includes("health")).length,
      high: dataFlows.filter((f) => f.priority === "high").length,
      medium: dataFlows.filter((f) => f.priority === "medium").length,
      low: dataFlows.filter((f) => f.priority === "low").length,
    };
  }, [dataFlows]);

  const filteredFlows = useMemo(() => {
    if (showOnlyHigh) {
      return dataFlows.filter((f) => f.priority === "high");
    }
    return dataFlows;
  }, [dataFlows, showOnlyHigh]);

  const getDataTypeColor = (type: DataType) => {
    switch (type) {
      case "PII":
        return "#ef4444"; // Red
      case "financial":
        return "#f59e0b"; // Orange
      case "health":
        return "#8b5cf6"; // Purple
      default:
        return "#6b7280"; // Gray
    }
  };

  if (dataFlows.length === 0) {
    return (
      <div className="data-flow-scanner">
        <div className="data-flow-scanner-header">
          <h3 className="data-flow-scanner-title">
            <Search size={18} />
            Data Flow Scanner
          </h3>
        </div>
        <div className="data-flow-scanner-empty">
          <p>No sensitive data flows detected.</p>
          <p className="data-flow-scanner-empty-subtitle">
            Tag components with PII, financial, or health data to track flows.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="data-flow-scanner">
      <div className="data-flow-scanner-header">
        <div className="data-flow-scanner-title-row">
          <h3 className="data-flow-scanner-title">
            <Search size={18} />
            Data Flow Security
          </h3>
          <div className="data-flow-scanner-stats">
            {flowStats.high > 0 && (
              <Badge color="error" className="priority-badge">
                {flowStats.high} High
              </Badge>
            )}
            {flowStats.medium > 0 && (
              <Badge color="warning" className="priority-badge">
                {flowStats.medium} Medium
              </Badge>
            )}
          </div>
        </div>
        {dataFlows.length > 0 && (
          <div className="data-flow-scanner-filters">
            <Button
              variant={showOnlyHigh ? "primary" : "ghost"}
              size="sm"
              onClick={() => setShowOnlyHigh(!showOnlyHigh)}
            >
              <Filter size={12} />
              {showOnlyHigh ? "Showing High Priority" : "Show All"}
            </Button>
          </div>
        )}
      </div>

      <div className="data-flow-scanner-list">
        {filteredFlows.length === 0 ? (
          <div className="data-flow-scanner-empty">
            <p>
              {dataFlows.length === 0
                ? "No sensitive data flows detected."
                : "No high priority security issues."}
            </p>
          </div>
        ) : (
          filteredFlows.map((flow, index) => (
            <div key={`${flow.from}-${flow.to}-${index}`} className="data-flow-item">
              <div className="data-flow-item-header">
                <div className="data-flow-item-encryption">
                  {flow.encrypted ? (
                    <Lock size={14} className="icon-encrypted" />
                  ) : (
                    <Unlock size={14} className="icon-unencrypted" />
                  )}
                </div>
                <div className="data-flow-item-content">
                  <div className="data-flow-item-path-row">
                    <div className="data-flow-item-path">
                      <span className="data-flow-from">{flow.from}</span>
                      <span className="data-flow-arrow">→</span>
                      <span className="data-flow-to">{flow.to}</span>
                    </div>
                    <Badge
                      color={
                        flow.priority === "high"
                          ? "error"
                          : flow.priority === "medium"
                            ? "warning"
                            : "neutral"
                      }
                      className="priority-badge-small"
                    >
                      {flow.priority}
                    </Badge>
                  </div>
                  {flow.protocol && <div className="data-flow-item-protocol">{flow.protocol}</div>}
                </div>
              </div>
              <div className="data-flow-item-types">
                {flow.dataTypes.map((type) => (
                  <span
                    key={type}
                    className="data-type-badge"
                    style={{ backgroundColor: getDataTypeColor(type) }}
                  >
                    {type}
                  </span>
                ))}
              </div>
              {flow.actionable && (
                <div className="data-flow-item-actionable">
                  <strong>Fix:</strong> {flow.actionable}
                </div>
              )}
              {!flow.encrypted && !flow.actionable && (
                <div className="data-flow-item-warning">
                  <AlertTriangle size={12} />
                  <span>Unencrypted flow detected. Consider using HTTPS/TLS.</span>
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
