// apps/designer/src/components/SRE/ReliabilityMetrics.tsx
import { useMemo } from "react";
import { Activity, TrendingUp, Server, AlertCircle, Minus } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import "./ReliabilityMetrics.css";

interface ReliabilityMetric {
  name: string;
  value: number;
  unit: string;
  trend: "up" | "down" | "stable";
  status: "healthy" | "warning" | "critical";
}

export function ReliabilityMetrics() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();
  const nodes = model.getNodes();
  const relations = model.getRelations();

  // Calculate reliability metrics (simulated - in real app, would come from observability data)
  const metrics = useMemo<ReliabilityMetric[]>(() => {
    const metricList: ReliabilityMetric[] = [];

    // 1. Mean Time Between Failures (MTBF) - simulated
    const mtbf = 720; // hours
    metricList.push({
      name: "MTBF",
      value: mtbf,
      unit: "hours",
      trend: "up",
      status: mtbf > 500 ? "healthy" : mtbf > 200 ? "warning" : "critical",
    });

    // 2. Mean Time To Recovery (MTTR) - simulated
    const mttr = 2.5; // hours
    metricList.push({
      name: "MTTR",
      value: mttr,
      unit: "hours",
      trend: "down", // Lower is better
      status: mttr < 4 ? "healthy" : mttr < 8 ? "warning" : "critical",
    });

    // 3. Availability - calculated from components
    const totalComponents = nodes.size;
    const healthyComponents = Math.floor(totalComponents * 0.95); // Simulated 95% healthy
    const availability = totalComponents > 0 ? (healthyComponents / totalComponents) * 100 : 100;
    metricList.push({
      name: "Availability",
      value: availability,
      unit: "%",
      trend: availability > 99 ? "stable" : "down",
      status: availability >= 99.9 ? "healthy" : availability >= 99 ? "warning" : "critical",
    });

    // 4. Error Rate - simulated
    const errorRate = 0.15; // 0.15%
    metricList.push({
      name: "Error Rate",
      value: errorRate,
      unit: "%",
      trend: errorRate < 0.1 ? "down" : errorRate < 0.5 ? "stable" : "up",
      status: errorRate < 0.1 ? "healthy" : errorRate < 0.5 ? "warning" : "critical",
    });

    // 5. P95 Latency - simulated
    const p95Latency = 180; // ms
    metricList.push({
      name: "P95 Latency",
      value: p95Latency,
      unit: "ms",
      trend: p95Latency < 200 ? "down" : p95Latency < 500 ? "stable" : "up",
      status: p95Latency < 200 ? "healthy" : p95Latency < 500 ? "warning" : "critical",
    });

    // 6. System Complexity (based on architecture)
    const avgConnections = relations.length / Math.max(1, nodes.size);
    metricList.push({
      name: "Complexity",
      value: Math.round(avgConnections * 10) / 10,
      unit: "avg connections",
      trend: avgConnections < 3 ? "down" : avgConnections < 5 ? "stable" : "up",
      status: avgConnections < 3 ? "healthy" : avgConnections < 5 ? "warning" : "critical",
    });

    return metricList;
  }, [nodes, relations, architectureModel]);

  const getStatusColor = (status: ReliabilityMetric["status"]) => {
    switch (status) {
      case "healthy":
        return "#22c55e";
      case "warning":
        return "#f59e0b";
      case "critical":
        return "#ef4444";
    }
  };

  const getStatusIcon = (status: ReliabilityMetric["status"]) => {
    switch (status) {
      case "healthy":
        return <Activity size={14} className="icon-healthy" />;
      case "warning":
        return <AlertCircle size={14} className="icon-warning" />;
      case "critical":
        return <AlertCircle size={14} className="icon-critical" />;
    }
  };

  const getTrendIcon = (trend: ReliabilityMetric["trend"], metricName: string) => {
    // For MTTR and Error Rate, down is good; for others, up is good
    const isGoodTrend =
      metricName === "MTTR" ||
      metricName === "Error Rate" ||
      metricName === "P95 Latency" ||
      metricName === "Complexity"
        ? trend === "down"
        : trend === "up";

    if (isGoodTrend) {
      return <TrendingUp size={12} className="trend-icon trend-good" />;
    } else if (trend === "stable") {
      return <Minus size={12} className="trend-icon trend-stable" />;
    } else {
      return (
        <TrendingUp
          size={12}
          className="trend-icon trend-bad"
          style={{ transform: "rotate(180deg)" }}
        />
      );
    }
  };

  return (
    <div className="reliability-metrics">
      <div className="reliability-metrics-header">
        <h3 className="reliability-metrics-title">
          <Server size={18} />
          Reliability Metrics
        </h3>
      </div>

      <div className="reliability-metrics-content">
        {metrics.map((metric) => (
          <div key={metric.name} className={`reliability-metric metric-${metric.status}`}>
            <div className="reliability-metric-header">
              <div className="reliability-metric-info">
                <div className="reliability-metric-name">{metric.name}</div>
                <div
                  className="reliability-metric-value"
                  style={{ color: getStatusColor(metric.status) }}
                >
                  {metric.value.toFixed(metric.unit === "%" ? 2 : metric.unit === "hours" ? 1 : 0)}{" "}
                  {metric.unit}
                </div>
              </div>
              <div className="reliability-metric-status">
                {getStatusIcon(metric.status)}
                {getTrendIcon(metric.trend, metric.name)}
              </div>
            </div>
            <div className="reliability-metric-bar">
              <div
                className="reliability-bar-fill"
                style={{
                  width: `${Math.min(100, (metric.value / (metric.name === "Availability" ? 100 : metric.name === "Error Rate" ? 1 : metric.name === "MTBF" ? 1000 : metric.name === "MTTR" ? 10 : 10)) * 100)}%`,
                  backgroundColor: getStatusColor(metric.status),
                }}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
