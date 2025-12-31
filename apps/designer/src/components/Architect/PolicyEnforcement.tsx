// apps/designer/src/components/Architect/PolicyEnforcement.tsx
import { useMemo } from "react";
import { Shield, AlertTriangle, CheckCircle, XCircle } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import "./PolicyEnforcement.css";

interface PolicyViolation {
  policyId: string;
  policyName: string;
  severity: "critical" | "warning" | "info";
  description: string;
  affectedElements: string[];
  suggestion?: string;
}

export function PolicyEnforcement() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();
  const policies = model.getPolicies();
  const nodes = model.getNodes();
  const relations = model.getRelations();

  // Scan for policy violations
  const violations = useMemo<PolicyViolation[]>(() => {
    if (!architectureModel || policies.length === 0) {
      return [];
    }

    const violationsList: PolicyViolation[] = [];

    // Example policy checks
    for (const policy of policies) {
      const policyData = policy as any;
      const policyId = policyData.id || policyData.name || "unknown";
      const policyName = policyData.name || policyId;
      const policyCategory = policyData.category?.toLowerCase() || "";

      // Check for HTTPS-only policy
      if (policyCategory.includes("security") || policyName.toLowerCase().includes("https")) {
        for (const [nodeId, node] of nodes.entries()) {
          const nodeData = node as any;
          const technology = (nodeData.technology || "").toLowerCase();
          const description = (nodeData.description || "").toLowerCase();
          const tags = ((nodeData.tags || []) as string[]).map((t: string) => t.toLowerCase());

          // Check if component uses HTTP instead of HTTPS
          const usesHttp =
            (technology.includes("http") && !technology.includes("https")) ||
            description.includes("http://") ||
            (tags.includes("http") && !tags.includes("https"));

          // Skip if explicitly marked as internal or local
          const isInternal =
            tags.includes("internal") || tags.includes("local") || description.includes("internal");

          if (usesHttp && !isInternal) {
            violationsList.push({
              policyId,
              policyName,
              severity: "critical",
              description: `${nodeId} uses HTTP instead of HTTPS`,
              affectedElements: [nodeId],
              suggestion:
                "Update to use HTTPS for secure communication. For internal-only services, consider marking with 'internal' tag.",
            });
          }
        }
      }

      // Check for "No God objects" or complexity policy
      if (
        policyName.toLowerCase().includes("god object") ||
        policyName.toLowerCase().includes("complexity") ||
        policyName.toLowerCase().includes("fan-in") ||
        policyName.toLowerCase().includes("fan-out")
      ) {
        // Count fan-in and fan-out separately for better analysis
        const fanIn = new Map<string, number>();
        const fanOut = new Map<string, number>();

        for (const rel of relations) {
          const source = (rel as any).source?.model || (rel as any).source;
          const target = (rel as any).target?.model || (rel as any).target;
          if (source) fanOut.set(source, (fanOut.get(source) || 0) + 1);
          if (target) fanIn.set(target, (fanIn.get(target) || 0) + 1);
        }

        // Check all nodes for high complexity
        const allNodeIds = new Set([...fanIn.keys(), ...fanOut.keys()]);
        for (const nodeId of allNodeIds) {
          const inCount = fanIn.get(nodeId) || 0;
          const outCount = fanOut.get(nodeId) || 0;
          const totalConnections = inCount + outCount;

          // Threshold: more than 5 connections total, or more than 3 in either direction
          if (totalConnections > 5 || inCount > 3 || outCount > 3) {
            const severity =
              totalConnections > 10 || inCount > 5 || outCount > 5 ? "critical" : "warning";
            violationsList.push({
              policyId,
              policyName,
              severity,
              description: `${nodeId} has high complexity: ${totalConnections} total connections (${inCount} incoming, ${outCount} outgoing)`,
              affectedElements: [nodeId],
              suggestion:
                "Consider splitting this component into smaller, more focused components with single responsibilities.",
            });
          }
        }
      }

      // Check for authentication/authorization policy
      if (
        policyCategory.includes("access") ||
        policyName.toLowerCase().includes("auth") ||
        policyName.toLowerCase().includes("authentication") ||
        policyName.toLowerCase().includes("authorization")
      ) {
        // Check for public-facing components without authentication
        for (const [nodeId, node] of nodes.entries()) {
          const nodeData = node as any;
          const tags = ((nodeData.tags || []) as string[]).map((t: string) => t.toLowerCase());
          const technology = (nodeData.technology || "").toLowerCase();
          const description = (nodeData.description || "").toLowerCase();

          const isPublicFacing =
            tags.some((t) => t.includes("public") || t.includes("external")) ||
            technology.includes("api") ||
            technology.includes("gateway") ||
            technology.includes("load balancer") ||
            description.includes("public") ||
            description.includes("external");

          const hasAuth =
            tags.some((t) => t.includes("auth") || t.includes("oauth") || t.includes("jwt")) ||
            technology.includes("auth") ||
            description.includes("authentication") ||
            description.includes("authorization");

          // Check if there's an auth relationship
          const hasAuthRelation = relations.some((r: any) => {
            const source = (r.source?.model || r.source) === nodeId;
            const target = (r.target?.model || r.target) === nodeId;
            const title = (r.title || "").toLowerCase();
            return (
              (source || target) &&
              (title.includes("auth") || title.includes("login") || title.includes("authenticate"))
            );
          });

          if (isPublicFacing && !hasAuth && !hasAuthRelation) {
            violationsList.push({
              policyId,
              policyName,
              severity: "critical",
              description: `${nodeId} is public-facing but lacks authentication`,
              affectedElements: [nodeId],
              suggestion:
                "Add authentication mechanism (OAuth, JWT, API key, etc.) or mark as internal-only if appropriate.",
            });
          }
        }
      }

      // Check for data encryption policy
      if (
        policyCategory.includes("encryption") ||
        policyName.toLowerCase().includes("encrypt") ||
        policyName.toLowerCase().includes("tls") ||
        policyName.toLowerCase().includes("ssl")
      ) {
        for (const [nodeId, node] of nodes.entries()) {
          const nodeData = node as any;
          const technology = (nodeData.technology || "").toLowerCase();
          const tags = ((nodeData.tags || []) as string[]).map((t: string) => t.toLowerCase());
          const description = (nodeData.description || "").toLowerCase();

          const handlesData =
            tags.some(
              (t) => t.includes("database") || t.includes("storage") || t.includes("cache")
            ) ||
            technology.includes("database") ||
            technology.includes("storage") ||
            technology.includes("cache") ||
            description.includes("data") ||
            description.includes("storage");

          const isEncrypted =
            technology.includes("https") ||
            technology.includes("tls") ||
            technology.includes("ssl") ||
            tags.some((t) => t.includes("encrypt") || t.includes("tls") || t.includes("ssl")) ||
            description.includes("encrypted") ||
            description.includes("tls");

          if (handlesData && !isEncrypted) {
            violationsList.push({
              policyId,
              policyName,
              severity: "critical",
              description: `${nodeId} handles data but encryption is not indicated`,
              affectedElements: [nodeId],
              suggestion:
                "Ensure data is encrypted at rest and in transit. Use TLS/SSL for connections and encryption for storage.",
            });
          }
        }
      }
    }

    return violationsList;
  }, [policies, nodes, relations, architectureModel]);

  const violationCounts = useMemo(() => {
    return {
      critical: violations.filter((v) => v.severity === "critical").length,
      warning: violations.filter((v) => v.severity === "warning").length,
      info: violations.filter((v) => v.severity === "info").length,
      total: violations.length,
    };
  }, [violations]);

  if (policies.length === 0) {
    return (
      <div className="policy-enforcement">
        <div className="policy-enforcement-header">
          <h3 className="policy-enforcement-title">
            <Shield size={18} />
            Policy Enforcement
          </h3>
        </div>
        <div className="policy-enforcement-empty">
          <p>No policies defined yet.</p>
          <p className="policy-enforcement-empty-subtitle">
            Define policies in your architecture model to enforce architectural standards. This view
            will automatically detect violations such as: missing HTTPS, high complexity components,
            missing authentication, and encryption requirements.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="policy-enforcement">
      <div className="policy-enforcement-header">
        <h3 className="policy-enforcement-title">
          <Shield size={18} />
          Policy Enforcement
        </h3>
        <div className="policy-enforcement-stats">
          {violationCounts.critical > 0 && (
            <span className="stat-item stat-critical">
              <XCircle size={14} />
              {violationCounts.critical}
            </span>
          )}
          {violationCounts.warning > 0 && (
            <span className="stat-item stat-warning">
              <AlertTriangle size={14} />
              {violationCounts.warning}
            </span>
          )}
          {violationCounts.total === 0 && (
            <span className="stat-item stat-clean">
              <CheckCircle size={14} />
              All compliant
            </span>
          )}
        </div>
      </div>

      <div className="policy-enforcement-list">
        {violations.length === 0 ? (
          <div className="policy-enforcement-clean">
            <CheckCircle size={32} className="clean-icon" />
            <p>All policies are compliant!</p>
          </div>
        ) : (
          violations.map((violation, index) => (
            <div
              key={`${violation.policyId}-${index}`}
              className={`violation-item violation-${violation.severity}`}
            >
              <div className="violation-item-header">
                <div className="violation-item-severity">
                  {violation.severity === "critical" && (
                    <XCircle size={16} className="icon-critical" />
                  )}
                  {violation.severity === "warning" && (
                    <AlertTriangle size={16} className="icon-warning" />
                  )}
                  {violation.severity === "info" && <CheckCircle size={16} className="icon-info" />}
                </div>
                <div className="violation-item-content">
                  <div className="violation-item-policy">{violation.policyName}</div>
                  <div className="violation-item-description">{violation.description}</div>
                  {violation.suggestion && (
                    <div className="violation-item-suggestion">{violation.suggestion}</div>
                  )}
                </div>
              </div>
              {violation.affectedElements.length > 0 && (
                <div className="violation-item-elements">
                  Affects: {violation.affectedElements.join(", ")}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
