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
import { useArchitectureStore, useSelectionStore, useUIStore } from "../../stores";
import type {
  SystemJSON,
  ContainerJSON,
  ComponentJSON,
  PersonJSON,
  RequirementJSON,
  ADRJSON,
  FlowJSON,
  ScenarioJSON,
} from "../../types";
import "./DetailsPanel.css";

interface DetailsPanelProps {
  onClose?: () => void;
}

export function DetailsPanel({ onClose }: DetailsPanelProps) {
  const data = useArchitectureStore((s) => s.data);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const selectNode = useSelectionStore((s) => s.selectNode);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  if (!selectedNodeId || !data) {
    return null;
  }

  // Find the selected node in the data
  const nodeInfo = findNode(data, selectedNodeId);

  if (!nodeInfo) {
    return null;
  }

  const { node, type } = nodeInfo;

  // Calculate dependencies from relations
  const arch = data.architecture;
  const relations = arch.relations || [];

  // Find incoming relations (what uses this node)
  const incoming = relations
    .filter((r) => r.to === selectedNodeId || r.to === node.id)
    .map((r) => ({
      relation: r,
      source: findNodeById(data, r.from),
    }))
    .filter((x) => x.source);

  // Find outgoing relations (what this node uses)
  const outgoing = relations
    .filter((r) => r.from === selectedNodeId || r.from === node.id)
    .map((r) => ({
      relation: r,
      target: findNodeById(data, r.to),
    }))
    .filter((x) => x.target);

  // Find linked requirements and ADRs (if node has them)
  const linkedReqs =
    "requirements" in node && Array.isArray(node.requirements) ? node.requirements : [];
  const linkedADRs = "adrs" in node && Array.isArray(node.adrs) ? node.adrs : [];

  // Find related items by tags or node references
  const allRequirements = arch.requirements || [];
  const allADRs = arch.adrs || [];
  const allFlows = arch.flows || [];
  const allScenarios = arch.scenarios || [];

  // Find requirements tagged with this node
  const relatedRequirements = allRequirements.filter((req: RequirementJSON) =>
    req.tags?.some((tag) => tag === node.id || tag === selectedNodeId)
  );

  // Find ADRs tagged with this node
  const relatedADRs = allADRs.filter((adr: ADRJSON) =>
    adr.tags?.some((tag) => tag === node.id || tag === selectedNodeId)
  );

  // Find flows that reference this node in their steps
  const relatedFlows = allFlows.filter((flow: FlowJSON) =>
    flow.steps?.some(
      (step) =>
        step.from === node.id ||
        step.to === node.id ||
        step.from === selectedNodeId ||
        step.to === selectedNodeId
    )
  );

  // Find scenarios that reference this node in their steps
  const relatedScenarios = allScenarios.filter((scenario: ScenarioJSON) =>
    scenario.steps?.some(
      (step) =>
        step.from === node.id ||
        step.to === node.id ||
        step.from === selectedNodeId ||
        step.to === selectedNodeId
    )
  );

  const getTypeIcon = (nodeType: string) => {
    switch (nodeType?.toLowerCase()) {
      case "person":
        return <Info size={12} />;
      case "system":
        return <Info size={12} />;
      case "container":
        return <Info size={12} />;
      case "component":
        return <Info size={12} />;
      default:
        return <Info size={12} />;
    }
  };

  return (
    <div className="details-panel">
      <div className="details-header">
        <h3 className="details-title">{getLabel(node)}</h3>
        <div className="details-actions">
          <button
            className="action-icon-btn"
            onClick={() => setActiveTab("code")}
            title="View Source"
            aria-label="View Source"
          >
            <FileCode size={16} />
          </button>
          <button
            className="close-btn"
            onClick={() => {
              selectNode(null);
              onClose?.();
            }}
            aria-label="Close details"
          >
            <X size={16} />
          </button>
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
            <span className="detail-value code">{node.id}</span>
          </div>

          {"technology" in node && node.technology && (
            <div className="detail-row">
              <span className="detail-label">Technology</span>
              <span className="detail-value">{node.technology}</span>
            </div>
          )}

          {/* Child counts */}
          {"containers" in node && node.containers && node.containers.length > 0 && (
            <div className="detail-row">
              <span className="detail-label">Containers</span>
              <span className="detail-value">{node.containers.length}</span>
            </div>
          )}

          {"components" in node && node.components && node.components.length > 0 && (
            <div className="detail-row">
              <span className="detail-label">Components</span>
              <span className="detail-value">{node.components.length}</span>
            </div>
          )}
        </div>

        {getDescription(node) && (
          <div className="detail-section">
            <div className="section-title">
              <Info size={14} />
              Description
            </div>
            <p className="description-text">{getDescription(node)}</p>
          </div>
        )}

        {"tags" in node && node.tags && node.tags.length > 0 && (
          <div className="detail-section">
            <div className="section-title">
              <Tag size={14} />
              Tags
            </div>
            <div className="tags-list">
              {node.tags.map((tag, i) => (
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
                      onClick={() => selectNode(inc.relation.from)}
                    >
                      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                        {getTypeIcon(inc.source?.type || "")}
                        <span className="dep-name">
                          {inc.source ? getLabel(inc.source.node) : inc.relation.from}
                        </span>
                      </div>
                      {(inc.relation.label || inc.relation.verb) && (
                        <span className="dep-desc">{inc.relation.label || inc.relation.verb}</span>
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
                    <div key={idx} className="dep-item" onClick={() => selectNode(out.relation.to)}>
                      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                        {getTypeIcon(out.target?.type || "")}
                        <span className="dep-name">
                          {out.target ? getLabel(out.target.node) : out.relation.to}
                        </span>
                      </div>
                      {(out.relation.label || out.relation.verb) && (
                        <span className="dep-desc">{out.relation.label || out.relation.verb}</span>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {/* Related Requirements */}
        {(relatedRequirements.length > 0 || linkedReqs.length > 0) && (
          <div className="detail-section">
            <div className="section-title">
              <ShieldCheck size={14} />
              Requirements
            </div>
            <div className="related-list">
              {linkedReqs.map((reqId: string) => {
                const req = allRequirements.find((r: RequirementJSON) => r.id === reqId);
                if (!req)
                  return (
                    <div key={reqId} className="related-item">
                      {reqId}
                    </div>
                  );
                return (
                  <div key={req.id} className="related-item">
                    <div className="related-item-header">
                      <span className="related-item-title">{req.title || req.id}</span>
                      {req.type && <span className="related-item-type">{req.type}</span>}
                    </div>
                    {req.description && <p className="related-item-desc">{req.description}</p>}
                  </div>
                );
              })}
              {relatedRequirements
                .filter((req) => !linkedReqs.includes(req.id))
                .map((req: RequirementJSON) => (
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
        {(relatedADRs.length > 0 || linkedADRs.length > 0) && (
          <div className="detail-section">
            <div className="section-title">
              <FileText size={14} />
              Architecture Decision Records
            </div>
            <div className="related-list">
              {linkedADRs.map((adrId: string) => {
                const adr = allADRs.find((a: ADRJSON) => a.id === adrId);
                if (!adr)
                  return (
                    <div key={adrId} className="related-item">
                      {adrId}
                    </div>
                  );
                const isPending = !adr.decision || adr.decision.trim() === "";
                return (
                  <div key={adr.id} className={`related-item ${isPending ? "pending-action" : ""}`}>
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
              {relatedADRs
                .filter((adr) => !linkedADRs.includes(adr.id))
                .map((adr: ADRJSON) => {
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
              {relatedFlows.map((flow: FlowJSON) => (
                <div key={flow.id} className="related-item">
                  <div className="related-item-header">
                    <span className="related-item-title">
                      {flow.title || flow.label || flow.id}
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
              {relatedScenarios.map((scenario: ScenarioJSON) => (
                <div key={scenario.id} className="related-item">
                  <div className="related-item-header">
                    <span className="related-item-title">
                      {scenario.title || scenario.label || scenario.id}
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

// Helper functions
function findNode(
  data: Parameters<Parameters<typeof useArchitectureStore>[0]>[0]["data"],
  id: string
): { node: SystemJSON | ContainerJSON | ComponentJSON | PersonJSON; type: string } | null {
  if (!data) return null;
  const arch = data.architecture;

  // Check persons
  const person = arch.persons?.find((p) => p.id === id);
  if (person) return { node: person, type: "Person" };

  // Check systems and their children
  for (const system of arch.systems ?? []) {
    if (system.id === id) return { node: system, type: "System" };

    for (const container of system.containers ?? []) {
      if (container.id === id) return { node: container, type: "Container" };

      for (const component of container.components ?? []) {
        if (component.id === id) return { node: component, type: "Component" };
      }
    }
  }

  return null;
}

function getLabel(node: { id: string; label?: string }): string {
  return node.label ?? node.id;
}

function getDescription(node: { description?: string }): string | undefined {
  return node.description;
}

// Helper to find node by ID (for dependency analysis)
function findNodeById(
  data: Parameters<Parameters<typeof useArchitectureStore>[0]>[0]["data"],
  id: string
): { node: SystemJSON | ContainerJSON | ComponentJSON | PersonJSON; type: string } | null {
  if (!data) return null;
  const arch = data.architecture;

  // Check persons
  const person = arch.persons?.find((p) => p.id === id);
  if (person) return { node: person, type: "Person" };

  // Check systems and their children
  for (const system of arch.systems ?? []) {
    if (system.id === id) return { node: system, type: "System" };

    for (const container of system.containers ?? []) {
      if (container.id === id) return { node: container, type: "Container" };

      for (const component of container.components ?? []) {
        if (component.id === id) return { node: component, type: "Component" };
      }
    }
  }

  return null;
}
