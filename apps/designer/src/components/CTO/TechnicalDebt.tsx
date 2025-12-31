// apps/designer/src/components/CTO/TechnicalDebt.tsx
import { useMemo } from "react";
import { AlertTriangle, Code } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { detectAntiPatterns } from "../../utils/antiPatternDetector";
import "./TechnicalDebt.css";

type DebtCategory = "architecture" | "documentation" | "testing" | "security" | "performance";

interface TechnicalDebtItem {
  id: string;
  category: DebtCategory;
  description: string;
  severity: "high" | "medium" | "low";
  affectedComponents: string[];
}

export function TechnicalDebt() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();
  const nodes = model.getNodes();
  const relations = model.getRelations();
  const adrs = model.getADRs();

  // Identify technical debt items
  const debtItems = useMemo<TechnicalDebtItem[]>(() => {
    const items: TechnicalDebtItem[] = [];

    // 1. Architecture debt (anti-patterns)
    const antiPatterns = detectAntiPatterns(architectureModel);
    for (const pattern of antiPatterns) {
      items.push({
        id: `debt-${pattern.type}-${pattern.affectedElements[0]}`,
        category: "architecture",
        description: pattern.description,
        severity:
          pattern.severity === "critical"
            ? "high"
            : pattern.severity === "warning"
              ? "medium"
              : "low",
        affectedComponents: pattern.affectedElements,
      });
    }

    // 2. Documentation debt
    const undocumentedComponents = Array.from(nodes.entries())
      .filter(([, node]: [string, any]) => !node.description && !node.metadata?.documentation)
      .map(([id]) => id);

    if (undocumentedComponents.length > 0) {
      items.push({
        id: "debt-documentation",
        category: "documentation",
        description: `${undocumentedComponents.length} component${undocumentedComponents.length !== 1 ? "s" : ""} lack documentation`,
        severity:
          undocumentedComponents.length > 10
            ? "high"
            : undocumentedComponents.length > 5
              ? "medium"
              : "low",
        affectedComponents: undocumentedComponents.slice(0, 5), // Show first 5
      });
    }

    // 3. ADR debt (deprecated/superseded ADRs)
    const deprecatedADRs = adrs.filter((adr: any) => {
      const status = (adr.status || "").toLowerCase();
      return status === "deprecated" || status === "superseded";
    });

    if (deprecatedADRs.length > 0) {
      items.push({
        id: "debt-adr-deprecated",
        category: "architecture",
        description: `${deprecatedADRs.length} deprecated/superseded ADR${deprecatedADRs.length !== 1 ? "s" : ""} need cleanup`,
        severity: deprecatedADRs.length > 5 ? "high" : "medium",
        affectedComponents: [],
      });
    }

    // 4. Complexity debt (high connection count)
    const highComplexityComponents: string[] = [];
    for (const [componentId] of nodes.entries()) {
      const connectionCount = relations.filter((r: any) => {
        const source = r.source?.model || r.source;
        const target = r.target?.model || r.target;
        return source === componentId || target === componentId;
      }).length;
      if (connectionCount > 5) {
        highComplexityComponents.push(componentId);
      }
    }

    if (highComplexityComponents.length > 0) {
      items.push({
        id: "debt-complexity",
        category: "architecture",
        description: `${highComplexityComponents.length} component${highComplexityComponents.length !== 1 ? "s" : ""} have high complexity (>5 connections)`,
        severity: highComplexityComponents.length > 10 ? "high" : "medium",
        affectedComponents: highComplexityComponents.slice(0, 5),
      });
    }

    return items;
  }, [nodes, relations, adrs, architectureModel]);

  const debtStats = useMemo(() => {
    const highSeverity = debtItems.filter((d) => d.severity === "high").length;
    const mediumSeverity = debtItems.filter((d) => d.severity === "medium").length;
    const lowSeverity = debtItems.filter((d) => d.severity === "low").length;

    return {
      total: debtItems.length,
      high: highSeverity,
      medium: mediumSeverity,
      low: lowSeverity,
    };
  }, [debtItems]);

  const getCategoryIcon = (category: DebtCategory) => {
    switch (category) {
      case "architecture":
        return <Code size={14} />;
      case "documentation":
      case "testing":
      case "security":
      case "performance":
        return <AlertTriangle size={14} />;
    }
  };

  const getSeverityColor = (severity: TechnicalDebtItem["severity"]) => {
    switch (severity) {
      case "high":
        return "#ef4444";
      case "medium":
        return "#f59e0b";
      case "low":
        return "#22c55e";
    }
  };

  if (debtItems.length === 0) {
    return (
      <div className="technical-debt">
        <div className="technical-debt-header">
          <h3 className="technical-debt-title">
            <AlertTriangle size={18} />
            Technical Debt
          </h3>
        </div>
        <div className="technical-debt-empty">
          <AlertTriangle size={32} className="empty-icon" />
          <p>No technical debt detected!</p>
          <p className="technical-debt-empty-subtitle">
            Your architecture is in good shape. This view tracks anti-patterns, undocumented
            components, deprecated ADRs, and high-complexity components as they appear.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="technical-debt">
      <div className="technical-debt-header">
        <h3 className="technical-debt-title">
          <AlertTriangle size={18} />
          Technical Debt
        </h3>
        <div className="technical-debt-stats">
          {debtStats.high > 0 && <span className="debt-stat debt-high">{debtStats.high} High</span>}
          {debtStats.medium > 0 && (
            <span className="debt-stat debt-medium">{debtStats.medium} Medium</span>
          )}
          {debtStats.low > 0 && <span className="debt-stat debt-low">{debtStats.low} Low</span>}
        </div>
      </div>

      <div className="technical-debt-content">
        {debtItems.map((item) => (
          <div key={item.id} className={`debt-item debt-${item.severity}`}>
            <div className="debt-item-header">
              <div className="debt-item-category">
                {getCategoryIcon(item.category)}
                <span className="debt-category-label">{item.category}</span>
              </div>
              <div
                className="debt-item-severity"
                style={{ color: getSeverityColor(item.severity) }}
              >
                {item.severity.toUpperCase()}
              </div>
            </div>
            <div className="debt-item-description">{item.description}</div>
            {item.affectedComponents.length > 0 && (
              <div className="debt-item-footer">
                <div className="debt-item-components">
                  Affects: {item.affectedComponents.join(", ")}
                  {item.affectedComponents.length > 5 && "..."}
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
