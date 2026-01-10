// apps/designer/src/components/Architect/AntiPatternDetector.tsx
import { useMemo, useState } from "react";
import { AlertTriangle, GitBranch, Package, Filter } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { detectAntiPatterns, type AntiPattern } from "../../utils/antiPatternDetector";
import type { Priority } from "../../types";
import "./AntiPatternDetector.css";

interface PrioritizedAntiPattern extends AntiPattern {
  priority: Priority;
  actionable?: string;
}

function getPriority(pattern: AntiPattern): Priority {
  if (pattern.severity === "critical") return "high";
  if (pattern.severity === "warning") return "medium";
  return "low";
}

function getActionable(pattern: AntiPattern): string {
  if (pattern.refactoring) {
    // Extract first actionable step
    const lines = pattern.refactoring.split("\n");
    return lines[0].trim().replace(/^[-*]\s*/, "");
  }
  if (pattern.suggestion) {
    return pattern.suggestion;
  }
  return "Review and refactor to remove anti-pattern";
}

export function AntiPatternDetector() {
  const storeModel = useArchitectureStore((s) => s.model);
  const model = getArchitectureModel();
  const [showOnlyHigh, setShowOnlyHigh] = useState(true);

  // Memoize model data to prevent infinite loops
  const architectureModel = useMemo(() => model.getModel(), [storeModel, model]);

  const antiPatterns = useMemo<PrioritizedAntiPattern[]>(() => {
    const patterns = detectAntiPatterns(architectureModel);
    return patterns.map((p) => ({
      ...p,
      priority: getPriority(p),
      actionable: getActionable(p),
    }));
  }, [architectureModel]);

  const filteredPatterns = useMemo(() => {
    if (showOnlyHigh) {
      return antiPatterns.filter((p) => p.priority === "high");
    }
    return antiPatterns;
  }, [antiPatterns, showOnlyHigh]);

  const patternCounts = useMemo(() => {
    return {
      critical: antiPatterns.filter((p) => p.severity === "critical").length,
      warning: antiPatterns.filter((p) => p.severity === "warning").length,
      info: antiPatterns.filter((p) => p.severity === "info").length,
      total: antiPatterns.length,
      high: antiPatterns.filter((p) => p.priority === "high").length,
      medium: antiPatterns.filter((p) => p.priority === "medium").length,
      low: antiPatterns.filter((p) => p.priority === "low").length,
    };
  }, [antiPatterns]);

  const getPatternIcon = (type: AntiPattern["type"]) => {
    switch (type) {
      case "cycle":
      case "circular-dependency":
        return <GitBranch size={16} />;
      case "god-object":
      case "god-component":
        return <Package size={16} />;
      default:
        return <AlertTriangle size={16} />;
    }
  };

  if (antiPatterns.length === 0) {
    return (
      <div className="anti-pattern-detector">
        <div className="anti-pattern-detector-header">
          <h3 className="anti-pattern-detector-title">
            <AlertTriangle size={18} />
            Anti-Pattern Detection
          </h3>
        </div>
        <div className="anti-pattern-detector-clean">
          <AlertTriangle size={32} className="clean-icon" />
          <p>No anti-patterns detected!</p>
          <p className="anti-pattern-detector-clean-subtitle">
            Your architecture follows good design principles. This view monitors for cyclic
            dependencies, god objects, and other architectural anti-patterns as you add components.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="anti-pattern-detector">
      <div className="anti-pattern-detector-header">
        <div className="anti-pattern-detector-title-row">
          <h3 className="anti-pattern-detector-title">
            <AlertTriangle size={18} />
            Anti-Patterns
          </h3>
          <div className="anti-pattern-detector-stats">
            {patternCounts.high > 0 && (
              <Badge color="error" className="priority-badge">
                {patternCounts.high} High
              </Badge>
            )}
            {patternCounts.medium > 0 && (
              <Badge color="warning" className="priority-badge">
                {patternCounts.medium} Medium
              </Badge>
            )}
          </div>
        </div>
        {antiPatterns.length > 0 && (
          <div className="anti-pattern-detector-filters">
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

      <div className="anti-pattern-detector-list">
        {filteredPatterns.length === 0 ? (
          <div className="anti-pattern-detector-clean">
            <AlertTriangle size={32} className="clean-icon" />
            <p>
              {antiPatterns.length === 0
                ? "No anti-patterns detected!"
                : "No high priority anti-patterns"}
            </p>
          </div>
        ) : (
          filteredPatterns.map((pattern, index) => (
            <div
              key={`${pattern.type}-${index}`}
              className={`anti-pattern-item anti-pattern-${pattern.severity}`}
            >
              <div className="anti-pattern-item-header">
                <div className="anti-pattern-item-icon">{getPatternIcon(pattern.type)}</div>
                <div className="anti-pattern-item-content">
                  <div className="anti-pattern-item-type-row">
                    <div className="anti-pattern-item-type">{pattern.type.replace("-", " ")}</div>
                    <Badge
                      color={
                        pattern.priority === "high"
                          ? "error"
                          : pattern.priority === "medium"
                            ? "warning"
                            : "neutral"
                      }
                      className="priority-badge-small"
                    >
                      {pattern.priority}
                    </Badge>
                  </div>
                  <div className="anti-pattern-item-description">{pattern.description}</div>
                  {pattern.actionable && (
                    <div className="anti-pattern-item-actionable">
                      <strong>Fix:</strong> {pattern.actionable}
                    </div>
                  )}
                </div>
              </div>
              {pattern.affectedElements.length > 0 && (
                <div className="anti-pattern-item-elements">
                  <strong>Affects:</strong> {pattern.affectedElements.slice(0, 3).join(", ")}
                  {pattern.affectedElements.length > 3 &&
                    ` (+${pattern.affectedElements.length - 3} more)`}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
