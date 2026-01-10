// apps/designer/src/components/SRE/SLOManager.tsx
import { useMemo } from "react";
import { Target, CheckCircle, AlertTriangle, XCircle } from "lucide-react";
import { Badge } from "@sruja/ui";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import "./SLOManager.css";

type SLOType = "availability" | "latency" | "error-rate";
type SLOStatus = "met" | "at-risk" | "breached";

interface SLO {
  id: string;
  service: string;
  type: SLOType;
  target: number; // e.g., 99.9 for 99.9% availability
  current: number; // Current measured value
  window: string; // e.g., "30d"
  status: SLOStatus;
}

export function SLOManager() {
  const model = getArchitectureModel();
  const architectureSLOs = model.getSLOs();

  // Convert architecture model SLOs to display format
  const slos = useMemo<SLO[]>(() => {
    return architectureSLOs.map(
      (sloData: {
        id?: string;
        slo?: Record<string, unknown>;
        service?: string;
        window?: string;
      }) => {
        const sloId = sloData.id || "unknown";
        const slo = sloData.slo || {};

        // Extract SLO properties (structure may vary)
        const type = (slo.type || slo.targetType || "availability") as SLOType;
        const target =
          slo.target ||
          slo.value ||
          (type === "availability" ? 99.9 : type === "latency" ? 200 : 0.1);
        const windowStr = sloData.window || slo.window || slo.timeWindow || "30d"; // Rename to avoid conflict with global window

        // For now, we don't have real-time metrics, so we'll show the target
        // In a real implementation, this would come from monitoring system
        const current = slo.current || slo.actual || target; // Default to target if no current value

        // Determine status based on current vs target
        let status: SLOStatus = "met";
        const currentVal = Number(current);
        const targetVal = Number(target);
        if (type === "availability") {
          status =
            currentVal >= targetVal
              ? "met"
              : currentVal >= targetVal * 0.95
                ? "at-risk"
                : "breached";
        } else if (type === "latency" || type === "error-rate") {
          status =
            currentVal <= targetVal
              ? "met"
              : currentVal <= targetVal * 1.1
                ? "at-risk"
                : "breached";
        } else if (type === "saturation") {
          // Assuming 'saturation' is a new possible type
          status =
            currentVal <= targetVal
              ? "met"
              : currentVal <= targetVal * 1.2
                ? "at-risk"
                : "breached";
        }

        return {
          id: sloId,
          service: sloData.service || sloId,
          type: type as SLOType,
          target: targetVal,
          current: currentVal,
          window: windowStr,
          status,
        } as SLO;
      }
    );
  }, [architectureSLOs]);

  const getStatusIcon = (status: SLOStatus) => {
    switch (status) {
      case "met":
        return <CheckCircle size={16} className="icon-met" />;
      case "at-risk":
        return <AlertTriangle size={16} className="icon-at-risk" />;
      case "breached":
        return <XCircle size={16} className="icon-breached" />;
    }
  };

  const getStatusColor = (status: SLOStatus) => {
    switch (status) {
      case "met":
        return "#22c55e";
      case "at-risk":
        return "#f59e0b";
      case "breached":
        return "#ef4444";
    }
  };

  const formatValue = (type: SLOType, value: number) => {
    if (type === "availability") {
      return `${value.toFixed(2)}%`;
    } else if (type === "latency") {
      return `${value.toFixed(0)}ms`;
    } else {
      return `${value.toFixed(2)}%`;
    }
  };

  const getSLOTypeLabel = (type: SLOType) => {
    switch (type) {
      case "availability":
        return "Availability";
      case "latency":
        return "Latency";
      case "error-rate":
        return "Error Rate";
    }
  };

  const atRiskCount = slos.filter((s) => s.status === "at-risk" || s.status === "breached").length;
  const metCount = slos.filter((s) => s.status === "met").length;

  return (
    <div className="slo-manager">
      <div className="slo-manager-header">
        <div className="slo-manager-title-row">
          <h3 className="slo-manager-title">
            <Target size={18} />
            Service Level Objectives
          </h3>
          <div className="slo-manager-stats">
            {atRiskCount > 0 && (
              <Badge color="error" className="priority-badge">
                {atRiskCount} At Risk
              </Badge>
            )}
            {metCount > 0 && (
              <Badge color="success" className="priority-badge">
                {metCount} Met
              </Badge>
            )}
          </div>
        </div>
      </div>

      <div className="slo-manager-content">
        {slos.length === 0 ? (
          <div className="slo-manager-empty">
            <Target size={32} className="empty-icon" />
            <p>No SLOs defined.</p>
            <p className="slo-manager-empty-subtitle">
              Define SLOs in your architecture model using the <code>slo</code> block syntax.
            </p>
          </div>
        ) : (
          <div className="slo-list">
            {/* Show at-risk/breached SLOs first */}
            {slos
              .sort((a, b) => {
                if (a.status === "breached" && b.status !== "breached") return -1;
                if (a.status === "at-risk" && b.status === "met") return -1;
                return 0;
              })
              .map((slo) => (
                <div key={slo.id} className={`slo-item slo-${slo.status}`}>
                  <div className="slo-item-header">
                    <div className="slo-item-info">
                      <div className="slo-item-service">{slo.service}</div>
                      <div className="slo-item-type">{getSLOTypeLabel(slo.type)}</div>
                    </div>
                    <div className="slo-item-status" style={{ color: getStatusColor(slo.status) }}>
                      {getStatusIcon(slo.status)}
                    </div>
                  </div>
                  <div className="slo-item-metrics">
                    <div className="slo-metric">
                      <span className="slo-metric-label">Target</span>
                      <span className="slo-metric-value">{formatValue(slo.type, slo.target)}</span>
                    </div>
                    <div className="slo-metric">
                      <span className="slo-metric-label">Current</span>
                      <span
                        className="slo-metric-value"
                        style={{ color: getStatusColor(slo.status) }}
                      >
                        {formatValue(slo.type, slo.current)}
                      </span>
                    </div>
                  </div>
                  {(slo.status === "at-risk" || slo.status === "breached") && (
                    <div className="slo-item-actionable">
                      <strong>Action:</strong> Review {slo.service} performance and investigate
                      issues
                    </div>
                  )}
                </div>
              ))}
          </div>
        )}
      </div>
    </div>
  );
}
