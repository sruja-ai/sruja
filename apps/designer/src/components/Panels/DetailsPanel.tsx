import {
  X,
  Info,
  Tag,
  ArrowDownLeft,
  ArrowUpRight,
  ShieldCheck,
  FileText,
  FileCode,
  Play,
  Workflow,
} from "lucide-react";
import { Button } from "@sruja/ui";
import { useArchitectureStore, useSelectionStore, useUIStore } from "../../stores";
import "./DetailsPanel.css";

interface DetailsPanelProps {
  onClose?: () => void;
}

export function DetailsPanel({ onClose }: DetailsPanelProps) {
  const likec4Model = useArchitectureStore((s) => s.likec4Model);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const selectNode = useSelectionStore((s) => s.selectNode);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  if (!selectedNodeId || !likec4Model) {
    return null;
  }

  // Find the selected node directly from the flat map
  const node = likec4Model.elements?.[selectedNodeId] as any;

  if (!node) {
    return null;
  }

  // Capitalize usage for display
  const type = node.kind.charAt(0).toUpperCase() + node.kind.slice(1);

  // Calculate dependencies from relations
  const allRelations = likec4Model.relations || [];

  // Helper to extract FQN from FqnRef or string for backward compatibility
  const getFqn = (ref: any): string => typeof ref === 'object' && ref?.model ? ref.model : String(ref || '');

  // Find incoming relations (what uses this node)
  const incoming = allRelations
    .filter((r) => getFqn(r.target) === selectedNodeId)
    .map((r) => ({
      relation: r,
      source: likec4Model.elements?.[getFqn(r.source)],
    }))
    .filter((x) => x.source);

  // Find outgoing relations (what this node uses)
  const outgoing = allRelations
    .filter((r) => getFqn(r.source) === selectedNodeId)
    .map((r) => ({
      relation: r,
      target: likec4Model.elements?.[getFqn(r.target)],
    }))
    .filter((x) => x.target);

  // Sruja extensions data
  const sruja = likec4Model.sruja || {};
  const allRequirements = sruja.requirements || [];
  const allADRs = sruja.adrs || [];
  const allFlows = sruja.flows || [];
  const allScenarios = sruja.scenarios || [];

  // Currently we associate requirements/ADRs by tags matching the Node ID
  // Wait, did legacy use 'tags' or explicit linking? 
  // Legacy used: req.tags?.includes(nodeId) OR node.requirements array.
  // SrujaModelDump doesn't carry 'requirements' array on element directly based on shared types.
  // We'll rely on global lists + tags or manual associations.

  // Find requirements tagged with this node (case-insensitive)
  const relatedRequirements = allRequirements.filter((req: any) =>
    req.tags?.some((tag: string) => tag.toLowerCase() === node.id.toLowerCase())
  );

  // Find ADRs tagged with this node (case-insensitive)
  const relatedADRs = allADRs.filter((adr: any) =>
    adr.tags?.some((tag: string) => tag.toLowerCase() === node.id.toLowerCase())
  );

  // Find flows that reference this node in their steps
  const relatedFlows = allFlows.filter((flow) =>
    flow.steps?.some(
      (step) =>
        step.from === node.id ||
        step.to === node.id
    )
  );

  // Find scenarios that reference this node in their steps
  const relatedScenarios = allScenarios.filter((scenario) =>
    scenario.steps?.some(
      (step) =>
        step.from === node.id ||
        step.to === node.id
    )
  );

  const getTypeIcon = (_nodeType: string) => {
    // Determine icon based on type (currently all Info, but structured for future expansion)
    // Could accept any string from 'kind'
    return <Info size={12} />;
  };

  // Helper to count children - iterate all elements to find those with parent === id
  const children = Object.values(likec4Model.elements || {}).filter((e: any) => (e as any).parent === node.id);
  const containerCount = children.filter((c: any) => c.kind === 'container').length;
  const componentCount = children.filter((c: any) => c.kind === 'component').length;

  return (
    <div className="details-panel">
      <div className="details-header">
        <h3 className="details-title">{(node as any).title}</h3>
        <div className="details-actions">
          <Button
            variant="ghost"
            size="sm"
            className="action-icon-btn"
            onClick={() => setActiveTab("code")}
            title="View Source"
            aria-label="View Source"
          >
            <FileCode size={16} />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="close-btn"
            onClick={() => {
              selectNode(null);
              onClose?.();
            }}
            aria-label="Close details"
          >
            <X size={16} />
          </Button>
        </div>
      </div>

      <div className="details-content">
        {/* Basic Properties Group */}
        <div className="properties-group">
          <div className="detail-row">
            <span className="detail-label">Type</span>
            <span className="detail-value type-badge">{type}</span>
          </div>

          <div className="detail-row">
            <span className="detail-label">ID</span>
            <span className="detail-value code">{(node as any).id}</span>
          </div>

          {(node as any).technology && (
            <div className="detail-row">
              <span className="detail-label">Technology</span>
              <span className="detail-value">{(node as any).technology}</span>
            </div>
          )}

          {/* Child counts */}
          {containerCount > 0 && (
            <div className="detail-row">
              <span className="detail-label">Containers</span>
              <span className="detail-value">{containerCount}</span>
            </div>
          )}

          {componentCount > 0 && (
            <div className="detail-row">
              <span className="detail-label">Components</span>
              <span className="detail-value">{componentCount}</span>
            </div>
          )}
        </div>

        {(node as any).description && (
          <div className="detail-section">
            <div className="section-title">
              <Info size={14} />
              Description
            </div>
            <p className="description-text">
              {typeof (node as any).description === "string"
                ? (node as any).description
                : (node as any).description?.txt || (node as any).description?.md || ""}
            </p>
          </div>
        )}

        {(node as any).tags && (node as any).tags.length > 0 && (
          <div className="detail-section">
            <div className="section-title">
              <Tag size={14} />
              Tags
            </div>
            <div className="tags-list">
              {(node as any).tags.map((tag: any, i: any) => (
                <span key={i} className="tag">
                  {tag}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Dependency Analysis */}
        {(incoming.length > 0 || outgoing.length > 0) && (
          <div className="detail-section">
            <div className="section-title">
              <Info size={14} />
              Dependency Analysis
            </div>

            {incoming.length > 0 && (
              <div className="dependency-group">
                <h4 className="dep-header">
                  <ArrowDownLeft size={12} />
                  Incoming (Used By)
                </h4>
                <div className="dep-list">
                  {incoming.map((inc, idx) => (
                    <div
                      key={idx}
                      className="dep-item"
                      onClick={() => selectNode(getFqn(inc.relation.source))}
                    >
                      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                        {getTypeIcon((inc.source as any)?.kind || "")}
                        <span className="dep-name">
                          {(inc.source as any)?.title || getFqn(inc.relation.source)}
                        </span>
                      </div>
                      {(inc.relation.title || inc.relation.technology) && (
                        <span className="dep-desc">{inc.relation.title || inc.relation.technology}</span>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {outgoing.length > 0 && (
              <div className="dependency-group">
                <h4 className="dep-header">
                  <ArrowUpRight size={12} />
                  Outgoing (Uses)
                </h4>
                <div className="dep-list">
                  {outgoing.map((out, idx) => (
                    <div key={idx} className="dep-item" onClick={() => selectNode(getFqn(out.relation.target))}>
                      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                        {getTypeIcon((out.target as any)?.kind || "")}
                        <span className="dep-name">
                          {(out.target as any)?.title || getFqn(out.relation.target)}
                        </span>
                      </div>
                      {(out.relation.title || out.relation.technology) && (
                        <span className="dep-desc">{out.relation.title || out.relation.technology}</span>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {/* Related Requirements */}
        {relatedRequirements.length > 0 && (
          <div className="detail-section">
            <div className="section-title">
              <ShieldCheck size={14} />
              Requirements
            </div>
            <div className="related-list">
              {relatedRequirements.map((req) => (
                <div key={req.id} className="related-item">
                  <div className="related-item-header">
                    <span className="related-item-title">{req.title || req.id}</span>
                    {req.type && <span className="related-item-type">{req.type}</span>}
                  </div>
                  {req.description && <p className="related-item-desc">{req.description}</p>}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Related ADRs */}
        {relatedADRs.length > 0 && (
          <div className="detail-section">
            <div className="section-title">
              <FileText size={14} />
              Architecture Decision Records
            </div>
            <div className="related-list">
              {relatedADRs.map((adr) => {
                const isPending = !adr.decision || adr.decision.trim() === "";
                return (
                  <div
                    key={adr.id}
                    className={`related-item ${isPending ? "pending-action" : ""}`}
                  >
                    <div className="related-item-header">
                      <span className="related-item-title">{adr.title || adr.id}</span>
                      {isPending && (
                        <span className="pending-badge" title="Action required: Decision pending">
                          Pending
                        </span>
                      )}
                      {adr.status && !isPending && (
                        <span className="related-item-status">{adr.status}</span>
                      )}
                    </div>
                    {adr.context && <p className="related-item-desc">{adr.context}</p>}
                    {adr.decision ? (
                      <p className="related-item-desc">{adr.decision}</p>
                    ) : (
                      <p className="related-item-desc pending-note">
                        ⚠️ Decision pending - action required
                      </p>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Related Flows */}
        {relatedFlows.length > 0 && (
          <div className="detail-section">
            <div className="section-title">
              <Workflow size={14} />
              Flows
            </div>
            <div className="related-list">
              {relatedFlows.map((flow) => (
                <div key={flow.id} className="related-item">
                  <div className="related-item-header">
                    <span className="related-item-title">
                      {flow.title || flow.id}
                    </span>
                  </div>
                  {flow.description && <p className="related-item-desc">{flow.description}</p>}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Related Scenarios */}
        {relatedScenarios.length > 0 && (
          <div className="detail-section">
            <div className="section-title">
              <Play size={14} />
              Scenarios
            </div>
            <div className="related-list">
              {relatedScenarios.map((scenario) => (
                <div key={scenario.id} className="related-item">
                  <div className="related-item-header">
                    <span className="related-item-title">
                      {scenario.title || scenario.id}
                    </span>
                  </div>
                  {scenario.description && (
                    <p className="related-item-desc">{scenario.description}</p>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}


