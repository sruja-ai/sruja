import { useMemo } from "react";
import { TrendingUp, TrendingDown, Minus, BarChart3, Camera, RefreshCw } from "lucide-react";
import { Button } from "@sruja/ui"; // Import Button from UI package
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { useArchitectureStore } from "../../stores"; // Import store hook
import "./TrendAnalysis.css";

interface TrendMetric {
  name: string;
  current: number;
  previous: number | null;
  trend: "up" | "down" | "stable";
  change: number;
  changePercent: number;
}

export function TrendAnalysis() {
  const model = getArchitectureModel();
  const nodes = model.getNodes();
  const relations = model.getRelations();
  const adrs = model.getADRs();
  const policies = model.getPolicies();

  const baselineModel = useArchitectureStore((s) => s.baselineModel);
  const setBaseline = useArchitectureStore((s) => s.setBaseline);

  // Calculate trend metrics (REAL comparison vs baseline)
  const trends = useMemo<TrendMetric[]>(() => {
    const diff = model.getBaselineDiff(baselineModel);
    const metrics: TrendMetric[] = [];

    // Helper to calculate trend from diff
    const createMetric = (name: string, current: number, delta: number): TrendMetric => {
      const previous = baselineModel ? current - delta : null;
      let trend: "up" | "down" | "stable" = "stable";
      if (delta > 0) trend = "up";
      if (delta < 0) trend = "down";

      const changePercent = previous && previous > 0 ? (delta / previous) * 100 : 0;

      return {
        name,
        current,
        previous,
        trend,
        change: Math.abs(delta), // Show magnitude, direction is in trend/icon
        changePercent: Math.abs(changePercent),
      };
    };

    // Component count trend
    metrics.push(createMetric("Components", nodes.size, diff.components));

    // Relations count trend
    metrics.push(createMetric("Relations", relations.length, diff.relations));

    // ADR count trend
    metrics.push(createMetric("ADRs", adrs.length, diff.adrs));

    // Policy count trend
    metrics.push(createMetric("Policies", policies.length, diff.policies));

    // Complexity trend
    const currentComplexity = nodes.size > 0 ? relations.length / nodes.size : 0;
    metrics.push(
      createMetric("Avg Complexity", Math.round(currentComplexity * 10) / 10, diff.complexity)
    );

    return metrics;
  }, [nodes, relations, adrs, policies, baselineModel, model]);

  const getTrendIcon = (trend: TrendMetric["trend"]) => {
    switch (trend) {
      case "up":
        return <TrendingUp size={14} className="trend-icon trend-up" />;
      case "down":
        return <TrendingDown size={14} className="trend-icon trend-down" />;
      default:
        return <Minus size={14} className="trend-icon trend-stable" />;
    }
  };

  const getTrendColor = (trend: TrendMetric["trend"]) => {
    switch (trend) {
      case "up":
        return "#22c55e";
      case "down":
        return "#ef4444";
      default:
        return "#6b7280";
    }
  };

  return (
    <div className="trend-analysis">
      <div className="trend-analysis-header">
        <div className="header-left">
          <h3 className="trend-analysis-title">
            <BarChart3 size={18} />
            Trend Analysis
          </h3>
        </div>
        <div className="header-actions">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setBaseline()}
            title="Set current state as baseline for comparison"
          >
            <Camera size={14} />
            Set Baseline
          </Button>
          {baselineModel && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setBaseline(null)}
              title="Clear baseline"
            >
              <RefreshCw size={14} />
            </Button>
          )}
        </div>
      </div>

      <div className="trend-analysis-content">
        {trends.map((trend) => (
          <div key={trend.name} className="trend-metric">
            <div className="trend-metric-header">
              <div className="trend-metric-name">{trend.name}</div>
              {getTrendIcon(trend.trend)}
            </div>
            <div className="trend-metric-values">
              <div className="trend-metric-current">{trend.current}</div>
              {trend.previous !== null && (
                <div className="trend-metric-change" style={{ color: getTrendColor(trend.trend) }}>
                  {trend.change > 0 ? "+" : ""}
                  {trend.change} ({trend.changePercent > 0 ? "+" : ""}
                  {trend.changePercent.toFixed(1)}%)
                </div>
              )}
            </div>
            {trend.previous !== null && (
              <div className="trend-metric-previous">Previous: {trend.previous}</div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
