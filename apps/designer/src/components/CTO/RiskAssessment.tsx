// apps/designer/src/components/CTO/RiskAssessment.tsx
import { useMemo } from "react";
import { AlertTriangle, TrendingUp, Shield } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { detectAntiPatterns } from "../../utils/antiPatternDetector";
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
  mitigation?: string;
}

export function RiskAssessment() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();
  const nodes = model.getNodes();
  const relations = model.getRelations();
  const policies = model.getPolicies();

  // Assess risks across multiple dimensions
  const risks = useMemo<Risk[]>(() => {
    const riskList: Risk[] = [];

    // 1. Security risks
    const publicFacingComponents = Array.from(nodes.values()).filter((n: any) => {
      const tags = n.tags || [];
      const tech = (n.technology || "").toLowerCase();
      return (
        tags.some((t: string) => t.toLowerCase().includes("public")) ||
        tech.includes("api") ||
        tech.includes("gateway")
      );
    });

    if (publicFacingComponents.length > 0) {
      const hasAuth = policies.some((p: any) => {
        const name = (p.name || "").toLowerCase();
        return name.includes("auth") || name.includes("access");
      });

      riskList.push({
        id: "risk-security-public",
        category: "security",
        title: "Public-facing components without authentication",
        description: `${publicFacingComponents.length} public-facing components may lack proper authentication`,
        probability: hasAuth ? "medium" : "high",
        impact: "high",
        overall: hasAuth ? "medium" : "high",
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
    };
  }, [risks]);

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
        <h3 className="risk-assessment-title">
          <AlertTriangle size={18} />
          Risk Assessment
        </h3>
        <div className="risk-assessment-stats">
          {riskStats.critical > 0 && (
            <span className="risk-stat risk-critical">{riskStats.critical} Critical</span>
          )}
          {riskStats.high > 0 && <span className="risk-stat risk-high">{riskStats.high} High</span>}
        </div>
      </div>

      <div className="risk-assessment-content">
        {risks.map((risk) => (
          <div key={risk.id} className={`risk-item risk-${risk.overall}`}>
            <div className="risk-item-header">
              <div className="risk-item-meta">
                <span className="risk-category-wrapper">
                  {getCategoryIcon(risk.category)}
                  <span className="risk-category-label">{risk.category}</span>
                </span>
                <span className={`risk-badge risk-badge-${risk.overall}`}>{risk.overall}</span>
              </div>
              <h4 className="risk-item-title">{risk.title}</h4>
            </div>

            <div className="risk-item-body">
              <p className="risk-item-description">{risk.description}</p>

              <div className="risk-item-metrics">
                <div className="risk-metric">
                  <span className="risk-metric-label">Probability</span>
                  <span className={`risk-metric-value text-${risk.probability}`}>
                    {risk.probability}
                  </span>
                </div>
                <div className="risk-metric-divider" />
                <div className="risk-metric">
                  <span className="risk-metric-label">Impact</span>
                  <span className={`risk-metric-value text-${risk.impact}`}>{risk.impact}</span>
                </div>
              </div>
            </div>

            {risk.mitigation && (
              <div className="risk-item-footer">
                <div className="risk-mitigation-box">
                  <span className="mitigation-label">Mitigation Strategy</span>
                  <p className="mitigation-text">{risk.mitigation}</p>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
