// apps/designer/src/components/CTO/RiskAssessment.tsx
import { useMemo, useState } from "react";
import { AlertTriangle, TrendingUp, Shield, Filter } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { detectAntiPatterns } from "../../utils/antiPatternDetector";
import type { Priority } from "../../types";
import "./RiskAssessment.css";

type RiskLevel = "critical" | "high" | "medium" | "low";

interface Risk {
  id: string;
  category: "security" | "reliability" | "performance" | "maintainability" | "compliance";
  title: string;
  description: string;
  probability: RiskLevel;
  impact: RiskLevel;
  overall: RiskLevel;
  priority: Priority;
  mitigation?: string;
  actionable?: string; // One-line actionable mitigation
}

function getPriority(overall: RiskLevel): Priority {
  if (overall === "critical" || overall === "high") return "high";
  if (overall === "medium") return "medium";
  return "low";
}

export function RiskAssessment() {
  const storeModel = useArchitectureStore((s) => s.model);
  const model = getArchitectureModel();
  const [showOnlyHigh, setShowOnlyHigh] = useState(true);

  // Memoize model data to prevent infinite loops
  const { architectureModel, nodes, relations, policies } = useMemo(() => {
    const archModel = model.getModel();
    return {
      architectureModel: archModel,
      nodes: archModel?.elements ? new Map(Object.entries(archModel.elements)) : new Map(),
      relations: archModel?.relations || [],
      policies: model.getPolicies(),
    };
  }, [storeModel, model]);

  // Assess risks across multiple dimensions
  const risks = useMemo<Risk[]>(() => {
    const riskList: Risk[] = [];

    // 1. Security risks
    const publicFacingComponents = Array.from(nodes.values()).filter((n) => {
      const node = n as unknown as { tags?: string[]; technology?: string };
      const tags = node.tags || [];
      const tech = (node.technology || "").toLowerCase();
      return (
        tags.some((t: string) => t.toLowerCase().includes("public")) ||
        tech.includes("api") ||
        tech.includes("gateway")
      );
    });

    if (publicFacingComponents.length > 0) {
      const hasAuth = policies.some((p) => {
        const policy = p as unknown as { name?: string };
        const name = (policy.name || "").toLowerCase();
        return name.includes("auth") || name.includes("access");
      });

      const overall = hasAuth ? "medium" : "high";
      riskList.push({
        id: "risk-security-public",
        category: "security",
        title: "Public-facing components without authentication",
        description: `${publicFacingComponents.length} public-facing components may lack proper authentication`,
        probability: hasAuth ? "medium" : "high",
        impact: "high",
        overall,
        priority: getPriority(overall),
        actionable: "Add OAuth/JWT authentication to public-facing components",
        mitigation:
          "Implement authentication and authorization policies for all public-facing components",
      });
    }

    // 2. Reliability risks (anti-patterns)
    const antiPatterns = detectAntiPatterns(architectureModel);
    const cycles = antiPatterns.filter((p) => p.type === "cycle");

    if (cycles.length > 0) {
      riskList.push({
        id: "risk-reliability-cycles",
        category: "reliability",
        title: "Cyclic dependencies detected",
        description: `${cycles.length} cyclic dependency${cycles.length !== 1 ? "ies" : ""} can cause cascading failures`,
        probability: "high",
        impact: "high",
        overall: "high",
        priority: "high",
        actionable: "Break cycles using event-driven patterns or message queues",
        mitigation:
          "Refactor to break cycles using event-driven patterns or intermediate components",
      });
    }

    // 3. Performance risks (God objects)
    const godObjects = antiPatterns.filter(
      (p) => p.type === "god-object" || p.type === "god-component"
    );

    if (godObjects.length > 0) {
      riskList.push({
        id: "risk-performance-god-objects",
        category: "performance",
        title: "God objects/components detected",
        description: `${godObjects.length} component${godObjects.length !== 1 ? "s" : ""} with excessive connections may cause performance bottlenecks`,
        probability: "medium",
        impact: "medium",
        overall: "medium",
        priority: "medium",
        actionable: "Split into smaller, focused components",
        mitigation: "Split high-fan-in/fan-out components into smaller, focused components",
      });
    }

    // 4. Maintainability risks (complexity)
    const avgConnections = relations.length / Math.max(1, nodes.size);

    if (avgConnections > 3) {
      riskList.push({
        id: "risk-maintainability-complexity",
        category: "maintainability",
        title: "High architectural complexity",
        description: `Average ${avgConnections.toFixed(1)} connections per component indicates high complexity`,
        probability: "medium",
        impact: "medium",
        overall: "medium",
        priority: "medium",
        actionable: "Reduce coupling by introducing service boundaries or message queues",
        mitigation: "Consider simplifying architecture by reducing coupling between components",
      });
    }

    // 5. Compliance risks
    const hasPolicies = policies.length > 0;

    if (!hasPolicies) {
      riskList.push({
        id: "risk-compliance-policies",
        category: "compliance",
        title: "No governance policies defined",
        description: "Missing policies may lead to compliance violations",
        probability: "medium",
        impact: "high",
        overall: "medium",
        priority: "medium",
        actionable: "Define security, performance, and compliance policies",
        mitigation:
          "Define and enforce architectural policies for security, performance, and compliance",
      });
    }

    return riskList;
  }, [nodes, relations, policies, architectureModel]);

  const riskStats = useMemo(() => {
    return {
      critical: risks.filter((r) => r.overall === "critical").length,
      high: risks.filter((r) => r.overall === "high").length,
      medium: risks.filter((r) => r.overall === "medium").length,
      low: risks.filter((r) => r.overall === "low").length,
      total: risks.length,
      highPriority: risks.filter((r) => r.priority === "high").length,
      mediumPriority: risks.filter((r) => r.priority === "medium").length,
      lowPriority: risks.filter((r) => r.priority === "low").length,
    };
  }, [risks]);

  const filteredRisks = useMemo(() => {
    if (showOnlyHigh) {
      return risks.filter((r) => r.priority === "high");
    }
    return risks;
  }, [risks, showOnlyHigh]);

  const getCategoryIcon = (category: Risk["category"]) => {
    switch (category) {
      case "security":
        return <Shield size={14} />;
      case "reliability":
        return <AlertTriangle size={14} />;
      case "performance":
        return <TrendingUp size={14} />;
      default:
        return <AlertTriangle size={14} />;
    }
  };

  if (risks.length === 0) {
    return (
      <div className="risk-assessment">
        <div className="risk-assessment-header">
          <h3 className="risk-assessment-title">
            <AlertTriangle size={18} />
            Risk Assessment
          </h3>
        </div>
        <div className="risk-assessment-empty">
          <Shield size={32} className="empty-icon" />
          <p>No significant risks identified.</p>
          <p className="risk-assessment-empty-subtitle">
            Your architecture appears to be well-designed.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="risk-assessment">
      <div className="risk-assessment-header">
        <div className="risk-assessment-title-row">
          <h3 className="risk-assessment-title">
            <AlertTriangle size={18} />
            Architecture Risks
          </h3>
          <div className="risk-assessment-stats">
            {riskStats.highPriority > 0 && (
              <Badge color="error" className="priority-badge">
                {riskStats.highPriority} High
              </Badge>
            )}
            {riskStats.mediumPriority > 0 && (
              <Badge color="warning" className="priority-badge">
                {riskStats.mediumPriority} Medium
              </Badge>
            )}
          </div>
        </div>
        {risks.length > 0 && (
          <div className="risk-assessment-filters">
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

      <div className="risk-assessment-content">
        {filteredRisks.length === 0 ? (
          <div className="risk-assessment-empty">
            <Shield size={32} className="empty-icon" />
            <p>
              {risks.length === 0 ? "No significant risks identified." : "No high priority risks."}
            </p>
          </div>
        ) : (
          filteredRisks.map((risk) => (
            <div key={risk.id} className={`risk-item risk-${risk.overall}`}>
              <div className="risk-item-header">
                <div className="risk-item-meta">
                  <span className="risk-category-wrapper">
                    {getCategoryIcon(risk.category)}
                    <span className="risk-category-label">{risk.category}</span>
                  </span>
                  <Badge
                    color={
                      risk.priority === "high"
                        ? "error"
                        : risk.priority === "medium"
                          ? "warning"
                          : "neutral"
                    }
                    className="priority-badge-small"
                  >
                    {risk.priority}
                  </Badge>
                </div>
                <h4 className="risk-item-title">{risk.title}</h4>
              </div>

              <div className="risk-item-body">
                <p className="risk-item-description">{risk.description}</p>
                {risk.actionable && (
                  <div className="risk-item-actionable">
                    <strong>Mitigate:</strong> {risk.actionable}
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
