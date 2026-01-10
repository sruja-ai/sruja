// apps/designer/src/components/CTO/HealthScore.tsx
import { useMemo, useState } from "react";
import { Heart, TrendingUp, TrendingDown, AlertTriangle, Filter } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { detectAntiPatterns } from "../../utils/antiPatternDetector";
import type { Priority } from "../../types";
import "./HealthScore.css";

interface HealthMetric {
  name: string;
  value: string | number;
  status: "healthy" | "warning" | "critical";
  priority: Priority;
  description: string;
  actionable?: string; // One-line fix
}

function getPriority(status: HealthMetric["status"]): Priority {
  if (status === "critical") return "high";
  if (status === "warning") return "medium";
  return "low";
}

function getActionable(metric: HealthMetric): string | undefined {
  if (metric.status === "healthy") return undefined;
  if (metric.name.includes("Cyclic")) return "Break cycles using event-driven patterns";
  if (metric.name.includes("God Objects")) return "Split into smaller, focused components";
  if (metric.name.includes("Documentation")) return "Add descriptions to components";
  if (metric.name.includes("ADRs")) return "Document key architectural decisions";
  if (metric.name.includes("Policies")) return "Define governance policies";
  if (metric.name.includes("Connections")) return "Reduce coupling between components";
  return undefined;
}

export function HealthScore() {
  const storeModel = useArchitectureStore((s) => s.model);
  const model = getArchitectureModel();
  const [showOnlyIssues, setShowOnlyIssues] = useState(true); // Show only warnings/critical by default

  // Memoize model data to prevent infinite loops
  const { architectureModel, nodes, relations, policies, adrs } = useMemo(() => {
    const archModel = model.getModel();
    return {
      architectureModel: archModel,
      nodes: archModel?.elements ? new Map(Object.entries(archModel.elements)) : new Map(),
      relations: archModel?.relations || [],
      policies: model.getPolicies(),
      adrs: model.getADRs(),
    };
  }, [storeModel, model]);

  // Show actual health metrics (without arbitrary scoring)
  const healthMetrics = useMemo<HealthMetric[]>(() => {
    const metrics: HealthMetric[] = [];
    const antiPatterns = detectAntiPatterns(architectureModel);
    const cycleCount = antiPatterns.filter((p) => p.type === "cycle").length;
    const godObjectCount = antiPatterns.filter(
      (p) => p.type === "god-object" || p.type === "god-component"
    ).length;

    // 1. Structural Issues
    const cycleStatus: "healthy" | "warning" | "critical" =
      cycleCount === 0 ? "healthy" : cycleCount <= 2 ? "warning" : "critical";
    metrics.push({
      name: "Cyclic Dependencies",
      value: cycleCount,
      status: cycleStatus,
      priority: getPriority(cycleStatus),
      description:
        cycleCount === 0
          ? "No cyclic dependencies detected"
          : `${cycleCount} cyclic dependency${cycleCount !== 1 ? "ies" : ""} found`,
      actionable: getActionable({
        name: "Cyclic Dependencies",
        status: cycleStatus,
        priority: getPriority(cycleStatus),
        value: cycleCount,
        description: "",
      }),
    });

    const godObjectStatus: "healthy" | "warning" | "critical" =
      godObjectCount === 0 ? "healthy" : godObjectCount <= 2 ? "warning" : "critical";
    metrics.push({
      name: "God Objects",
      value: godObjectCount,
      status: godObjectStatus,
      priority: getPriority(godObjectStatus),
      description:
        godObjectCount === 0
          ? "No god objects/components detected"
          : `${godObjectCount} god object${godObjectCount !== 1 ? "s" : ""} found`,
      actionable: getActionable({
        name: "God Objects",
        status: godObjectStatus,
        priority: getPriority(godObjectStatus),
        value: godObjectCount,
        description: "",
      }),
    });

    // 2. Documentation
    const totalComponents = nodes.size;
    const documentedComponents = Array.from(nodes.values()).filter((n) => {
      const node = n as unknown as {
        description?: string;
        metadata?: { documentation?: string; docs?: string };
      };
      return node.description || node.metadata?.documentation || node.metadata?.docs;
    }).length;
    const documentationPercentage =
      totalComponents > 0 ? Math.round((documentedComponents / totalComponents) * 100) : 100;
    const docStatus: "healthy" | "warning" | "critical" =
      documentationPercentage >= 80
        ? "healthy"
        : documentationPercentage >= 50
          ? "warning"
          : "critical";
    metrics.push({
      name: "Documentation Coverage",
      value: `${documentationPercentage}%`,
      status: docStatus,
      priority: getPriority(docStatus),
      description: `${documentedComponents} of ${totalComponents} components documented`,
      actionable: getActionable({
        name: "Documentation Coverage",
        status: docStatus,
        priority: getPriority(docStatus),
        value: `${documentationPercentage}%`,
        description: "",
      }),
    });

    const adrStatus: "healthy" | "warning" | "critical" =
      adrs.length >= 3 ? "healthy" : adrs.length >= 1 ? "warning" : "critical";
    metrics.push({
      name: "ADRs",
      value: adrs.length,
      status: adrStatus,
      priority: getPriority(adrStatus),
      description: `${adrs.length} Architecture Decision Record${adrs.length !== 1 ? "s" : ""}`,
      actionable: getActionable({
        name: "ADRs",
        status: adrStatus,
        priority: getPriority(adrStatus),
        value: adrs.length,
        description: "",
      }),
    });

    // 3. Governance
    const policyStatus: "healthy" | "warning" | "critical" =
      policies.length >= 3 ? "healthy" : policies.length >= 1 ? "warning" : "critical";
    metrics.push({
      name: "Policies",
      value: policies.length,
      status: policyStatus,
      priority: getPriority(policyStatus),
      description: `${policies.length} governance polic${policies.length !== 1 ? "ies" : "y"}`,
      actionable: getActionable({
        name: "Policies",
        status: policyStatus,
        priority: getPriority(policyStatus),
        value: policies.length,
        description: "",
      }),
    });

    // 4. Complexity
    const avgConnections = relations.length / Math.max(1, nodes.size);
    const complexityStatus: "healthy" | "warning" | "critical" =
      avgConnections <= 3 ? "healthy" : avgConnections <= 5 ? "warning" : "critical";
    metrics.push({
      name: "Average Connections",
      value: avgConnections.toFixed(1),
      status: complexityStatus,
      priority: getPriority(complexityStatus),
      description: `${relations.length} relations across ${nodes.size} components`,
      actionable: getActionable({
        name: "Average Connections",
        status: complexityStatus,
        priority: getPriority(complexityStatus),
        value: avgConnections.toFixed(1),
        description: "",
      }),
    });

    return metrics;
  }, [nodes, relations, policies, adrs, architectureModel]);

  // Filter metrics by priority
  const filteredMetrics = useMemo(() => {
    if (showOnlyIssues) {
      return healthMetrics.filter((m) => m.status !== "healthy");
    }
    return healthMetrics;
  }, [healthMetrics, showOnlyIssues]);

  const getStatusColor = (status: HealthMetric["status"]) => {
    switch (status) {
      case "healthy":
        return "#22c55e";
      case "warning":
        return "#f59e0b";
      case "critical":
        return "#ef4444";
    }
  };

  const getStatusIcon = (status: HealthMetric["status"]) => {
    switch (status) {
      case "healthy":
        return <TrendingUp size={16} />;
      case "warning":
        return <AlertTriangle size={16} />;
      case "critical":
        return <TrendingDown size={16} />;
    }
  };

  const criticalCount = healthMetrics.filter((m) => m.status === "critical").length;
  const warningCount = healthMetrics.filter((m) => m.status === "warning").length;

  return (
    <div className="health-score">
      <div className="health-score-header">
        <div className="health-score-title-row">
          <h3 className="health-score-title">
            <Heart size={18} />
            Architecture Health
          </h3>
          <div className="health-score-stats">
            {criticalCount > 0 && (
              <Badge color="error" className="priority-badge">
                {criticalCount} Critical
              </Badge>
            )}
            {warningCount > 0 && (
              <Badge color="warning" className="priority-badge">
                {warningCount} Warning
              </Badge>
            )}
          </div>
        </div>
        {healthMetrics.length > 0 && (
          <div className="health-score-filters">
            <Button
              variant={showOnlyIssues ? "primary" : "ghost"}
              size="sm"
              onClick={() => setShowOnlyIssues(!showOnlyIssues)}
            >
              <Filter size={12} />
              {showOnlyIssues ? "Showing Issues Only" : "Show All"}
            </Button>
          </div>
        )}
      </div>

      <div className="health-score-content">
        {filteredMetrics.length === 0 ? (
          <div className="health-score-empty">
            <Heart size={32} className="empty-icon" style={{ color: "#22c55e" }} />
            <p>
              {healthMetrics.length === 0
                ? "Calculating health metrics..."
                : "All health metrics are healthy!"}
            </p>
          </div>
        ) : (
          filteredMetrics.map((metric) => (
            <div key={metric.name} className="health-dimension">
              <div className="health-dimension-header">
                <div className="health-dimension-info">
                  <div className="health-dimension-name-row">
                    <div className="health-dimension-name">{metric.name}</div>
                    <Badge
                      color={
                        metric.priority === "high"
                          ? "error"
                          : metric.priority === "medium"
                            ? "warning"
                            : "neutral"
                      }
                      className="priority-badge-small"
                    >
                      {metric.priority}
                    </Badge>
                  </div>
                  <div className="health-dimension-description">{metric.description}</div>
                  {metric.actionable && (
                    <div className="health-dimension-actionable">
                      <strong>Fix:</strong> {metric.actionable}
                    </div>
                  )}
                </div>
                <div className="health-dimension-score">
                  <div
                    className="health-score-value"
                    style={{ color: getStatusColor(metric.status) }}
                  >
                    {metric.value}
                  </div>
                  <div
                    className="health-score-icon"
                    style={{ color: getStatusColor(metric.status) }}
                  >
                    {getStatusIcon(metric.status)}
                  </div>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
