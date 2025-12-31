// apps/designer/src/components/CTO/HealthScore.tsx
import { useMemo } from "react";
import { Heart, TrendingUp, TrendingDown, AlertTriangle } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { detectAntiPatterns } from "../../utils/antiPatternDetector";
import "./HealthScore.css";

interface HealthMetric {
  name: string;
  value: string | number;
  status: "healthy" | "warning" | "critical";
  description: string;
}

export function HealthScore() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();
  const nodes = model.getNodes();
  const relations = model.getRelations();
  const policies = model.getPolicies();
  const adrs = model.getADRs();

  // Show actual health metrics (without arbitrary scoring)
  const healthMetrics = useMemo<HealthMetric[]>(() => {
    const metrics: HealthMetric[] = [];
    const antiPatterns = detectAntiPatterns(architectureModel);
    const cycleCount = antiPatterns.filter((p) => p.type === "cycle").length;
    const godObjectCount = antiPatterns.filter(
      (p) => p.type === "god-object" || p.type === "god-component"
    ).length;

    // 1. Structural Issues
    metrics.push({
      name: "Cyclic Dependencies",
      value: cycleCount,
      status: cycleCount === 0 ? "healthy" : cycleCount <= 2 ? "warning" : "critical",
      description:
        cycleCount === 0
          ? "No cyclic dependencies detected"
          : `${cycleCount} cyclic dependency${cycleCount !== 1 ? "ies" : ""} found`,
    });

    metrics.push({
      name: "God Objects",
      value: godObjectCount,
      status: godObjectCount === 0 ? "healthy" : godObjectCount <= 2 ? "warning" : "critical",
      description:
        godObjectCount === 0
          ? "No god objects/components detected"
          : `${godObjectCount} god object${godObjectCount !== 1 ? "s" : ""} found`,
    });

    // 2. Documentation
    const totalComponents = nodes.size;
    const documentedComponents = Array.from(nodes.values()).filter((n: any) => {
      return n.description || n.metadata?.documentation || n.metadata?.docs;
    }).length;
    const documentationPercentage =
      totalComponents > 0 ? Math.round((documentedComponents / totalComponents) * 100) : 100;

    metrics.push({
      name: "Documentation Coverage",
      value: `${documentationPercentage}%`,
      status:
        documentationPercentage >= 80
          ? "healthy"
          : documentationPercentage >= 50
            ? "warning"
            : "critical",
      description: `${documentedComponents} of ${totalComponents} components documented`,
    });

    metrics.push({
      name: "ADRs",
      value: adrs.length,
      status: adrs.length >= 3 ? "healthy" : adrs.length >= 1 ? "warning" : "critical",
      description: `${adrs.length} Architecture Decision Record${adrs.length !== 1 ? "s" : ""}`,
    });

    // 3. Governance
    metrics.push({
      name: "Policies",
      value: policies.length,
      status: policies.length >= 3 ? "healthy" : policies.length >= 1 ? "warning" : "critical",
      description: `${policies.length} governance polic${policies.length !== 1 ? "ies" : "y"}`,
    });

    // 4. Complexity
    const avgConnections = relations.length / Math.max(1, nodes.size);
    metrics.push({
      name: "Average Connections",
      value: avgConnections.toFixed(1),
      status: avgConnections <= 3 ? "healthy" : avgConnections <= 5 ? "warning" : "critical",
      description: `${relations.length} relations across ${nodes.size} components`,
    });

    return metrics;
  }, [nodes, relations, policies, adrs, architectureModel]);

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

  return (
    <div className="health-score">
      <div className="health-score-header">
        <h3 className="health-score-title">
          <Heart size={18} />
          Architecture Health Score
        </h3>
      </div>

      <div className="health-score-content">
        {healthMetrics.map((metric) => (
          <div key={metric.name} className="health-dimension">
            <div className="health-dimension-header">
              <div className="health-dimension-info">
                <div className="health-dimension-name">{metric.name}</div>
                <div className="health-dimension-description">{metric.description}</div>
              </div>
              <div className="health-dimension-score">
                <div
                  className="health-score-value"
                  style={{ color: getStatusColor(metric.status) }}
                >
                  {metric.value}
                </div>
                <div className="health-score-icon" style={{ color: getStatusColor(metric.status) }}>
                  {getStatusIcon(metric.status)}
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
