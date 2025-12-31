// apps/designer/src/components/Security/DataFlowScanner.tsx
import { useMemo } from "react";
import { Search, Lock, Unlock, AlertTriangle, CheckCircle } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import "./DataFlowScanner.css";

type DataType = "PII" | "financial" | "health" | "other";

interface DataFlow {
  from: string;
  to: string;
  dataTypes: DataType[];
  encrypted: boolean;
  protocol?: string;
}

export function DataFlowScanner() {
  const model = getArchitectureModel();
  const nodes = model.getNodes();
  const relations = model.getRelations();

  // Scan data flows for sensitive data
  const dataFlows = useMemo<DataFlow[]>(() => {
    const flows: DataFlow[] = [];

    for (const rel of relations) {
      const source = (rel as any).source?.model || (rel as any).source;
      const target = (rel as any).target?.model || (rel as any).target;
      const title = (rel as any).title || "";

      if (!source || !target) continue;

      const sourceNode = nodes.get(source);
      const targetNode = nodes.get(target);

      if (!sourceNode || !targetNode) continue;

      const sourceData = sourceNode as any;
      const targetData = targetNode as any;

      // Detect data types from metadata, tags, or component names
      const dataTypes: DataType[] = [];

      // Check for PII indicators
      if (
        sourceData.tags?.some((t: string) => t.toLowerCase().includes("pii")) ||
        targetData.tags?.some((t: string) => t.toLowerCase().includes("pii")) ||
        source.toLowerCase().includes("user") ||
        target.toLowerCase().includes("user")
      ) {
        dataTypes.push("PII");
      }

      // Check for financial data
      if (
        sourceData.tags?.some((t: string) => t.toLowerCase().includes("payment")) ||
        targetData.tags?.some((t: string) => t.toLowerCase().includes("payment")) ||
        source.toLowerCase().includes("payment") ||
        target.toLowerCase().includes("payment")
      ) {
        dataTypes.push("financial");
      }

      // Check for health data
      if (
        sourceData.tags?.some((t: string) => t.toLowerCase().includes("health")) ||
        targetData.tags?.some((t: string) => t.toLowerCase().includes("health")) ||
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
        flows.push({
          from: source,
          to: target,
          dataTypes,
          encrypted: isEncrypted,
          protocol: title,
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
    };
  }, [dataFlows]);

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
        <h3 className="data-flow-scanner-title">
          <Search size={18} />
          Data Flow Scanner
        </h3>
        <div className="data-flow-scanner-stats">
          <span className="flow-stat">{flowStats.total} flows</span>
          {flowStats.encrypted > 0 && (
            <span className="flow-stat flow-encrypted">
              <CheckCircle size={12} />
              {flowStats.encrypted} encrypted
            </span>
          )}
          {flowStats.unencrypted > 0 && (
            <span className="flow-stat flow-unencrypted">
              <AlertTriangle size={12} />
              {flowStats.unencrypted} unencrypted
            </span>
          )}
        </div>
      </div>

      <div className="data-flow-scanner-list">
        {dataFlows.map((flow, index) => (
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
                <div className="data-flow-item-path">
                  <span className="data-flow-from">{flow.from}</span>
                  <span className="data-flow-arrow">→</span>
                  <span className="data-flow-to">{flow.to}</span>
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
            {!flow.encrypted && (
              <div className="data-flow-item-warning">
                <AlertTriangle size={12} />
                <span>Unencrypted flow detected. Consider using HTTPS/TLS.</span>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
