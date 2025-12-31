// apps/designer/src/components/Architect/AntiPatternDetector.tsx
import { useMemo } from "react";
import { AlertTriangle, GitBranch, Package, XCircle } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { detectAntiPatterns, type AntiPattern } from "../../utils/antiPatternDetector";
import "./AntiPatternDetector.css";

export function AntiPatternDetector() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();

  const antiPatterns = useMemo<AntiPattern[]>(() => {
    return detectAntiPatterns(architectureModel);
  }, [architectureModel]);

  const patternCounts = useMemo(() => {
    return {
      critical: antiPatterns.filter((p) => p.severity === "critical").length,
      warning: antiPatterns.filter((p) => p.severity === "warning").length,
      info: antiPatterns.filter((p) => p.severity === "info").length,
      total: antiPatterns.length,
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
        <h3 className="anti-pattern-detector-title">
          <AlertTriangle size={18} />
          Anti-Pattern Detection
        </h3>
        <div className="anti-pattern-detector-stats">
          {patternCounts.critical > 0 && (
            <span className="stat-item stat-critical">
              <XCircle size={14} />
              {patternCounts.critical}
            </span>
          )}
          {patternCounts.warning > 0 && (
            <span className="stat-item stat-warning">
              <AlertTriangle size={14} />
              {patternCounts.warning}
            </span>
          )}
        </div>
      </div>

      <div className="anti-pattern-detector-list">
        {antiPatterns.map((pattern, index) => (
          <div
            key={`${pattern.type}-${index}`}
            className={`anti-pattern-item anti-pattern-${pattern.severity}`}
          >
            <div className="anti-pattern-item-header">
              <div className="anti-pattern-item-icon">{getPatternIcon(pattern.type)}</div>
              <div className="anti-pattern-item-content">
                <div className="anti-pattern-item-type">{pattern.type.replace("-", " ")}</div>
                <div className="anti-pattern-item-description">{pattern.description}</div>
                {pattern.explanation && (
                  <div className="anti-pattern-item-explanation">
                    <strong>Why this matters:</strong> {pattern.explanation}
                  </div>
                )}
                {pattern.refactoring && (
                  <div className="anti-pattern-item-refactoring">
                    <strong>How to fix:</strong>
                    <pre
                      style={{ whiteSpace: "pre-wrap", fontSize: "0.875rem", marginTop: "0.5rem" }}
                    >
                      {pattern.refactoring}
                    </pre>
                  </div>
                )}
                {pattern.suggestion && !pattern.refactoring && (
                  <div className="anti-pattern-item-suggestion">
                    <strong>Suggestion:</strong> {pattern.suggestion}
                  </div>
                )}
              </div>
            </div>
            {pattern.affectedElements.length > 0 && (
              <div className="anti-pattern-item-elements">
                Affects: {pattern.affectedElements.join(", ")}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
