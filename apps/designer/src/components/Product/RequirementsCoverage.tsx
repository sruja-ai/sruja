// apps/designer/src/components/Product/RequirementsCoverage.tsx
import { useMemo, useState } from "react";
import { CheckCircle, AlertCircle, XCircle, FileText, Filter } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import type { RequirementDump, ElementDump } from "@sruja/shared";
import "./RequirementsCoverage.css";

type RequirementStatus = "covered" | "partial" | "missing";

interface RequirementCoverage {
  id: string;
  description: string;
  type?: string;
  status: RequirementStatus;
  coveredBy: string[]; // Component IDs that cover this requirement
}

export function RequirementsCoverage() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();
  const [showOnlyIssues, setShowOnlyIssues] = useState(true); // Show only missing/partial by default

  const requirements = model.getRequirements();
  const nodes = model.getNodes();

  // Analyze requirement coverage
  const coverage = useMemo<RequirementCoverage[]>(() => {
    if (!architectureModel || requirements.length === 0) {
      return [];
    }

    return requirements.map((req: RequirementDump) => {
      const reqId = req.id || "unknown";
      const reqDescription = req.description || "No description";
      const reqType = req.type || "functional";

      // Check if requirement is linked to components via tags or metadata
      const coveredBy: string[] = [];

      // Check nodes for requirement references
      for (const [nodeId, node] of nodes.entries()) {
        const nodeData = node as ElementDump;
        // Check if node has this requirement in its tags, metadata, or requirements
        if (
          nodeData.tags?.includes(reqId) ||
          (nodeData.metadata as { requirements?: string[] })?.requirements?.includes(reqId) ||
          (nodeData as unknown as { requirements?: RequirementDump[] }).requirements?.some(
            (r) => r.id === reqId
          )
        ) {
          coveredBy.push(nodeId);
        }
      }

      // Determine status
      let status: RequirementStatus = "missing";
      if (coveredBy.length > 0) {
        status = coveredBy.length >= 1 ? "covered" : "partial";
      }

      return {
        id: reqId,
        description: reqDescription,
        type: reqType,
        status,
        coveredBy,
      };
    });
  }, [requirements, nodes, architectureModel]);

  const statusCounts = useMemo(() => {
    return {
      covered: coverage.filter((c) => c.status === "covered").length,
      partial: coverage.filter((c) => c.status === "partial").length,
      missing: coverage.filter((c) => c.status === "missing").length,
      total: coverage.length,
    };
  }, [coverage]);

  const filteredCoverage = useMemo(() => {
    if (showOnlyIssues) {
      return coverage.filter((c) => c.status !== "covered");
    }
    return coverage;
  }, [coverage, showOnlyIssues]);

  if (coverage.length === 0) {
    return (
      <div className="requirements-coverage">
        <div className="requirements-coverage-header">
          <h3 className="requirements-coverage-title">
            <FileText size={18} />
            Requirements Coverage
          </h3>
        </div>
        <div className="requirements-coverage-empty">
          <p>No requirements defined yet.</p>
          <p className="requirements-coverage-empty-subtitle">
            Add requirements to your architecture model to track coverage across components. Link
            requirements to components via tags or metadata to see coverage analysis.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="requirements-coverage">
      <div className="requirements-coverage-header">
        <div className="requirements-coverage-title-row">
          <h3 className="requirements-coverage-title">
            <FileText size={18} />
            Requirements Coverage
          </h3>
          <div className="requirements-coverage-stats">
            {statusCounts.missing > 0 && (
              <Badge color="error" className="priority-badge">
                {statusCounts.missing} Missing
              </Badge>
            )}
            {statusCounts.partial > 0 && (
              <Badge color="warning" className="priority-badge">
                {statusCounts.partial} Partial
              </Badge>
            )}
            {statusCounts.covered > 0 && (
              <Badge color="success" className="priority-badge">
                {statusCounts.covered} Covered
              </Badge>
            )}
          </div>
        </div>
        {coverage.length > 0 && (
          <div className="requirements-coverage-filters">
            <Button
              variant={showOnlyIssues ? "primary" : "ghost"}
              size="sm"
              onClick={() => setShowOnlyIssues(!showOnlyIssues)}
            >
              <Filter size={12} />
              {showOnlyIssues ? "Showing Issues Only" : "Show All"}
            </Button>
          </div>
        )}
      </div>

      <div className="requirements-coverage-list">
        {filteredCoverage.length === 0 ? (
          <div className="requirements-coverage-empty">
            <CheckCircle size={32} className="empty-icon" style={{ color: "#22c55e" }} />
            <p>
              {coverage.length === 0
                ? "No requirements defined yet."
                : "All requirements are covered!"}
            </p>
          </div>
        ) : (
          filteredCoverage
            .sort((a, b) => {
              if (a.status === "missing" && b.status !== "missing") return -1;
              if (a.status === "partial" && b.status === "covered") return -1;
              return 0;
            })
            .map((req) => (
              <div key={req.id} className={`requirement-item requirement-${req.status}`}>
                <div className="requirement-item-header">
                  <div className="requirement-item-status">
                    {req.status === "covered" && <CheckCircle size={16} className="icon-covered" />}
                    {req.status === "partial" && <AlertCircle size={16} className="icon-partial" />}
                    {req.status === "missing" && <XCircle size={16} className="icon-missing" />}
                  </div>
                  <div className="requirement-item-content">
                    <div className="requirement-item-id-row">
                      <div className="requirement-item-id">{req.id}</div>
                      {req.status === "missing" && (
                        <Badge color="error" className="priority-badge-small">
                          High
                        </Badge>
                      )}
                      {req.status === "partial" && (
                        <Badge color="warning" className="priority-badge-small">
                          Medium
                        </Badge>
                      )}
                    </div>
                    <div className="requirement-item-description">{req.description}</div>
                    {req.status !== "covered" && (
                      <div className="requirement-item-actionable">
                        <strong>Fix:</strong> Link requirement to components via tags or metadata
                      </div>
                    )}
                  </div>
                </div>
                {req.coveredBy.length > 0 && (
                  <div className="requirement-item-covered-by">
                    <strong>Covered by:</strong> {req.coveredBy.slice(0, 3).join(", ")}
                    {req.coveredBy.length > 3 && ` (+${req.coveredBy.length - 3} more)`}
                  </div>
                )}
              </div>
            ))
        )}
      </div>
    </div>
  );
}
