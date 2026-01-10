// apps/designer/src/components/Architect/PolicyEnforcement.tsx
import { useMemo, useState } from "react";
import { Shield, AlertTriangle, CheckCircle, XCircle, Filter } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import type { ElementDump, Policy, RelationDump } from "@sruja/shared";
import type { Priority } from "../../types";
import "./PolicyEnforcement.css";

interface PolicyViolation {
  policyId: string;
  policyName: string;
  severity: "critical" | "warning" | "info";
  priority: Priority;
  description: string;
  affectedElements: string[];
  suggestion?: string;
  actionable?: string; // One-line actionable fix
}

// Helper interface for properties that might exist on elements
interface ElementWithDetails extends ElementDump {
  technology?: string;
  description?: string;
}

// Helper to extract ID from relation source/target which can be string or object
const getRelationEndpointId = (
  endpoint: string | { model: string } | undefined
): string | undefined => {
  if (!endpoint) return undefined;
  if (typeof endpoint === "string") return endpoint;
  return endpoint.model;
};

export function PolicyEnforcement() {
  const storeModel = useArchitectureStore((s) => s.model);
  const model = getArchitectureModel();
  const [priorityFilter, setPriorityFilter] = useState<Priority | "all">("all");
  const [showOnlyHigh, setShowOnlyHigh] = useState(true); // Default: show only high priority

  // Memoize model data to prevent infinite loops
  const { architectureModel, policies, nodes, relations } = useMemo(() => {
    const archModel = model.getModel();
    return {
      architectureModel: archModel,
      policies: model.getPolicies(),
      nodes: archModel?.elements ? new Map(Object.entries(archModel.elements)) : new Map(),
      relations: archModel?.relations || [],
    };
  }, [storeModel, model]);

  // Scan for policy violations
  const violations = useMemo<PolicyViolation[]>(() => {
    if (!architectureModel || policies.length === 0) {
      return [];
    }

    const violationsList: PolicyViolation[] = [];

    // Example policy checks
    for (const policy of policies) {
      // Policy type should have these fields, but fallback safely if not matches exactly
      const policyId = policy.id || policy.title || "unknown";
      const policyName = policy.title || policyId;
      // category is not on standard Policy type yet, check if it exists or default
      const policyCategory = (
        (policy as Policy & { category?: string }).category || ""
      ).toLowerCase();

      // Check for HTTPS-only policy
      if (policyCategory.includes("security") || policyName.toLowerCase().includes("https")) {
        for (const [nodeId, node] of nodes.entries()) {
          const nodeData = node as ElementWithDetails;
          const technology = (nodeData.technology || "").toLowerCase();
          const description = (nodeData.description || "").toLowerCase();
          const tags = (nodeData.tags || []).map((t: string) => t.toLowerCase());

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
              priority: "high",
              description: `${nodeId} uses HTTP instead of HTTPS`,
              affectedElements: [nodeId],
              actionable: "Replace HTTP with HTTPS in service configuration",
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
          // Provide type assertion or check for relation properties
          // RelationDump typically has source/target
          const r = rel as RelationDump;
          const source = getRelationEndpointId(r.source);
          const target = getRelationEndpointId(r.target);

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
            const priority: Priority =
              totalConnections > 10 ? "high" : totalConnections > 7 ? "medium" : "low";
            violationsList.push({
              policyId,
              policyName,
              severity,
              priority,
              description: `${nodeId} has high complexity: ${totalConnections} connections`,
              affectedElements: [nodeId],
              actionable: "Split into smaller, focused components",
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
          const nodeData = node as ElementWithDetails;
          const tags = (nodeData.tags || []).map((t: string) => t.toLowerCase());
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
          const hasAuthRelation = relations.some((rel) => {
            const r = rel as RelationDump;
            const source = getRelationEndpointId(r.source);
            const target = getRelationEndpointId(r.target);

            const isConnected = source === nodeId || target === nodeId;
            const title = (r.title || "").toLowerCase();
            return (
              isConnected &&
              (title.includes("auth") || title.includes("login") || title.includes("authenticate"))
            );
          });

          if (isPublicFacing && !hasAuth && !hasAuthRelation) {
            violationsList.push({
              policyId,
              policyName,
              severity: "critical",
              priority: "high",
              description: `${nodeId} is public-facing but lacks authentication`,
              affectedElements: [nodeId],
              actionable: "Add OAuth/JWT authentication or mark as internal",
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
          const nodeData = node as ElementWithDetails;
          const technology = (nodeData.technology || "").toLowerCase();
          const tags = (nodeData.tags || []).map((t: string) => t.toLowerCase());
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
              priority: "high",
              description: `${nodeId} handles data without encryption`,
              affectedElements: [nodeId],
              actionable: "Enable TLS for connections and encryption at rest",
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
      high: violations.filter((v) => v.priority === "high").length,
      medium: violations.filter((v) => v.priority === "medium").length,
      low: violations.filter((v) => v.priority === "low").length,
    };
  }, [violations]);

  // Filter violations by priority
  const filteredViolations = useMemo(() => {
    if (showOnlyHigh) {
      return violations.filter((v) => v.priority === "high");
    }
    if (priorityFilter === "all") {
      return violations;
    }
    return violations.filter((v) => v.priority === priorityFilter);
  }, [violations, priorityFilter, showOnlyHigh]);

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
        <div className="policy-enforcement-title-row">
          <h3 className="policy-enforcement-title">
            <Shield size={18} />
            Policy Violations
          </h3>
          <div className="policy-enforcement-stats">
            {violationCounts.high > 0 && (
              <Badge color="error" className="priority-badge">
                {violationCounts.high} High
              </Badge>
            )}
            {violationCounts.medium > 0 && (
              <Badge color="warning" className="priority-badge">
                {violationCounts.medium} Medium
              </Badge>
            )}
            {violationCounts.total === 0 && (
              <Badge color="success">
                <CheckCircle size={12} />
                All compliant
              </Badge>
            )}
          </div>
        </div>
        {violations.length > 0 && (
          <div className="policy-enforcement-filters">
            <Button
              variant={showOnlyHigh ? "primary" : "ghost"}
              size="sm"
              onClick={() => setShowOnlyHigh(!showOnlyHigh)}
            >
              <Filter size={12} />
              {showOnlyHigh ? "Showing High Priority" : "Show All"}
            </Button>
            {!showOnlyHigh && (
              <div className="priority-filters">
                <Button
                  variant={priorityFilter === "all" ? "secondary" : "ghost"}
                  size="sm"
                  onClick={() => setPriorityFilter("all")}
                >
                  All
                </Button>
                <Button
                  variant={priorityFilter === "high" ? "secondary" : "ghost"}
                  size="sm"
                  onClick={() => setPriorityFilter("high")}
                >
                  High
                </Button>
                <Button
                  variant={priorityFilter === "medium" ? "secondary" : "ghost"}
                  size="sm"
                  onClick={() => setPriorityFilter("medium")}
                >
                  Medium
                </Button>
                <Button
                  variant={priorityFilter === "low" ? "secondary" : "ghost"}
                  size="sm"
                  onClick={() => setPriorityFilter("low")}
                >
                  Low
                </Button>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="policy-enforcement-list">
        {filteredViolations.length === 0 ? (
          <div className="policy-enforcement-clean">
            <CheckCircle size={32} className="clean-icon" />
            <p>
              {violations.length === 0
                ? "All policies are compliant!"
                : showOnlyHigh
                  ? "No high priority violations"
                  : "No violations match the filter"}
            </p>
          </div>
        ) : (
          filteredViolations.map((violation, index) => (
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
                  <div className="violation-item-policy-row">
                    <div className="violation-item-policy">{violation.policyName}</div>
                    <Badge
                      color={
                        violation.priority === "high"
                          ? "error"
                          : violation.priority === "medium"
                            ? "warning"
                            : "neutral"
                      }
                      className="priority-badge-small"
                    >
                      {violation.priority}
                    </Badge>
                  </div>
                  <div className="violation-item-description">{violation.description}</div>
                  {violation.actionable && (
                    <div className="violation-item-actionable">
                      <strong>Fix:</strong> {violation.actionable}
                    </div>
                  )}
                </div>
              </div>
              {violation.affectedElements.length > 0 && (
                <div className="violation-item-elements">
                  <strong>Affects:</strong> {violation.affectedElements.slice(0, 3).join(", ")}
                  {violation.affectedElements.length > 3 &&
                    ` (+${violation.affectedElements.length - 3} more)`}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
