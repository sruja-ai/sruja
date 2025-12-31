// apps/designer/src/components/Product/RequirementsCoverage.tsx
import { useMemo } from "react";
import { CheckCircle, AlertCircle, XCircle, FileText } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
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

  const requirements = model.getRequirements();
  const nodes = model.getNodes();

  // Analyze requirement coverage
  const coverage = useMemo<RequirementCoverage[]>(() => {
    if (!architectureModel || requirements.length === 0) {
      return [];
    }

    return requirements.map((req: any) => {
      const reqId = req.id || req.name || "unknown";
      const reqDescription = req.description || req.name || "No description";
      const reqType = req.type || "functional";

      // Check if requirement is linked to components via tags or metadata
      const coveredBy: string[] = [];

      // Check nodes for requirement references
      for (const [nodeId, node] of nodes.entries()) {
        const nodeData = node as any;
        // Check if node has this requirement in its tags, metadata, or requirements
        if (
          nodeData.tags?.includes(reqId) ||
          nodeData.metadata?.requirements?.includes(reqId) ||
          nodeData.requirements?.some((r: any) => r.id === reqId)
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
        <h3 className="requirements-coverage-title">
          <FileText size={18} />
          Requirements Coverage
        </h3>
        <div className="requirements-coverage-stats">
          <span className="stat-item stat-covered">
            <CheckCircle size={14} />
            {statusCounts.covered}
          </span>
          <span className="stat-item stat-partial">
            <AlertCircle size={14} />
            {statusCounts.partial}
          </span>
          <span className="stat-item stat-missing">
            <XCircle size={14} />
            {statusCounts.missing}
          </span>
        </div>
      </div>

      <div className="requirements-coverage-list">
        {coverage.map((req) => (
          <div key={req.id} className={`requirement-item requirement-${req.status}`}>
            <div className="requirement-item-header">
              <div className="requirement-item-status">
                {req.status === "covered" && <CheckCircle size={16} className="icon-covered" />}
                {req.status === "partial" && <AlertCircle size={16} className="icon-partial" />}
                {req.status === "missing" && <XCircle size={16} className="icon-missing" />}
              </div>
              <div className="requirement-item-content">
                <div className="requirement-item-id">{req.id}</div>
                <div className="requirement-item-description">{req.description}</div>
                {req.type && <div className="requirement-item-type">{req.type}</div>}
              </div>
            </div>
            {req.coveredBy.length > 0 && (
              <div className="requirement-item-covered-by">
                Covered by: {req.coveredBy.join(", ")}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
