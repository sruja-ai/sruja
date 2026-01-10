// apps/designer/src/components/CTO/TechnicalDebt.tsx
import { useMemo, useState } from "react";
import { AlertTriangle, Code, Filter } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { detectAntiPatterns } from "../../utils/antiPatternDetector";
import type { Priority } from "../../types";
import "./TechnicalDebt.css";

type DebtCategory = "architecture" | "documentation" | "testing" | "security" | "performance";

interface TechnicalDebtItem {
  id: string;
  category: DebtCategory;
  description: string;
  severity: "high" | "medium" | "low";
  priority: Priority;
  affectedComponents: string[];
  actionable?: string; // One-line fix
}

export function TechnicalDebt() {
  const storeModel = useArchitectureStore((s) => s.model);
  const model = getArchitectureModel();
  const [showOnlyHigh, setShowOnlyHigh] = useState(true);

  // Memoize model data to prevent infinite loops
  const { architectureModel, nodes, relations, adrs } = useMemo(() => {
    const archModel = model.getModel();
    return {
      architectureModel: archModel,
      nodes: archModel?.elements ? new Map(Object.entries(archModel.elements)) : new Map(),
      relations: archModel?.relations || [],
      adrs: model.getADRs(),
    };
  }, [storeModel, model]);

  // Identify technical debt items
  const debtItems = useMemo<TechnicalDebtItem[]>(() => {
    const items: TechnicalDebtItem[] = [];

    // 1. Architecture debt (anti-patterns)
    const antiPatterns = detectAntiPatterns(architectureModel);
    for (const pattern of antiPatterns) {
      const severity =
        pattern.severity === "critical"
          ? "high"
          : pattern.severity === "warning"
            ? "medium"
            : "low";
      items.push({
        id: `debt-${pattern.type}-${pattern.affectedElements[0]}`,
        category: "architecture",
        description: pattern.description,
        severity,
        priority: severity,
        actionable: pattern.suggestion || "Refactor to remove anti-pattern",
        affectedComponents: pattern.affectedElements,
      });
    }

    // 2. Documentation debt
    const undocumentedComponents = Array.from(nodes.entries())
      .filter(([, n]) => {
        const node = n as unknown as {
          description?: string;
          metadata?: { documentation?: string };
        };
        return !node.description && !node.metadata?.documentation;
      })
      .map(([id]) => id);

    if (undocumentedComponents.length > 0) {
      const severity =
        undocumentedComponents.length > 10
          ? "high"
          : undocumentedComponents.length > 5
            ? "medium"
            : "low";
      items.push({
        id: "debt-documentation",
        category: "documentation",
        description: `${undocumentedComponents.length} component${undocumentedComponents.length !== 1 ? "s" : ""} lack documentation`,
        severity,
        priority: severity,
        actionable: "Add descriptions to components",
        affectedComponents: undocumentedComponents.slice(0, 5), // Show first 5
      });
    }

    // 3. ADR debt (deprecated/superseded ADRs)
    const deprecatedADRs = adrs.filter((adr) => {
      const a = adr as unknown as { status?: string };
      const status = (a.status || "").toLowerCase();
      return status === "deprecated" || status === "superseded";
    });

    if (deprecatedADRs.length > 0) {
      const severity = deprecatedADRs.length > 5 ? "high" : "medium";
      items.push({
        id: "debt-adr-deprecated",
        category: "architecture",
        description: `${deprecatedADRs.length} deprecated/superseded ADR${deprecatedADRs.length !== 1 ? "s" : ""} need cleanup`,
        severity,
        priority: severity,
        actionable: "Review and archive or update deprecated ADRs",
        affectedComponents: [],
      });
    }

    // 4. Complexity debt (high connection count)
    const highComplexityComponents: string[] = [];
    for (const [componentId] of nodes.entries()) {
      const connectionCount = relations.filter((r) => {
        const rel = r as unknown as {
          source: string | { model: string };
          target: string | { model: string };
        };
        const source = typeof rel.source === "string" ? rel.source : rel.source.model;
        const target = typeof rel.target === "string" ? rel.target : rel.target.model;
        return source === componentId || target === componentId;
      }).length;
      if (connectionCount > 5) {
        highComplexityComponents.push(componentId);
      }
    }

    if (highComplexityComponents.length > 0) {
      const severity = highComplexityComponents.length > 10 ? "high" : "medium";
      items.push({
        id: "debt-complexity",
        category: "architecture",
        description: `${highComplexityComponents.length} component${highComplexityComponents.length !== 1 ? "s" : ""} have high complexity (>5 connections)`,
        severity,
        priority: severity,
        actionable: "Split into smaller, focused components",
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
      highPriority: debtItems.filter((d) => d.priority === "high").length,
      mediumPriority: debtItems.filter((d) => d.priority === "medium").length,
      lowPriority: debtItems.filter((d) => d.priority === "low").length,
    };
  }, [debtItems]);

  const filteredDebt = useMemo(() => {
    if (showOnlyHigh) {
      return debtItems.filter((d) => d.priority === "high");
    }
    return debtItems;
  }, [debtItems, showOnlyHigh]);

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

  return (
    <div className="technical-debt">
      <div className="technical-debt-header">
        <div className="technical-debt-title-row">
          <h3 className="technical-debt-title">
            <AlertTriangle size={18} />
            Technical Debt
          </h3>
          <div className="technical-debt-stats">
            {debtStats.highPriority > 0 && (
              <Badge color="error" className="priority-badge">
                {debtStats.highPriority} High
              </Badge>
            )}
            {debtStats.mediumPriority > 0 && (
              <Badge color="warning" className="priority-badge">
                {debtStats.mediumPriority} Medium
              </Badge>
            )}
          </div>
        </div>
        {debtItems.length > 0 && (
          <div className="technical-debt-filters">
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

      <div className="technical-debt-content">
        {filteredDebt.length === 0 ? (
          <div className="technical-debt-empty">
            <AlertTriangle size={32} className="empty-icon" />
            <p>
              {debtItems.length === 0
                ? "No technical debt detected!"
                : "No high priority technical debt"}
            </p>
          </div>
        ) : (
          filteredDebt.map((item) => (
            <div key={item.id} className={`debt-item debt-${item.severity}`}>
              <div className="debt-item-header">
                <div className="debt-item-category">
                  {getCategoryIcon(item.category)}
                  <span className="debt-category-label">{item.category}</span>
                </div>
                <Badge
                  color={
                    item.priority === "high"
                      ? "error"
                      : item.priority === "medium"
                        ? "warning"
                        : "neutral"
                  }
                  className="priority-badge-small"
                >
                  {item.priority}
                </Badge>
              </div>
              <div className="debt-item-description">{item.description}</div>
              {item.actionable && (
                <div className="debt-item-actionable">
                  <strong>Fix:</strong> {item.actionable}
                </div>
              )}
              {item.affectedComponents.length > 0 && (
                <div className="debt-item-footer">
                  <div className="debt-item-components">
                    <strong>Affects:</strong> {item.affectedComponents.slice(0, 3).join(", ")}
                    {item.affectedComponents.length > 3 &&
                      ` (+${item.affectedComponents.length - 3} more)`}
                  </div>
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
