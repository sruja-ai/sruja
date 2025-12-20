import type { NodeProps, Node } from "@xyflow/react";
import { ChevronDown, ChevronUp, AlertTriangle, Target, FileText, Play, Workflow } from "lucide-react";
import { Button } from "@sruja/ui";
import type { C4NodeData } from "../../../types";
import { isSystemNodeData, isContainerNodeData, type ContainerNodeData, getNodeTags } from "../../../types/nodeData";
import { getNodeColors, getTagStyles } from "../../../utils/colorScheme";
import { useViewStore } from "../../../stores";
import { useUIStore } from "../../../stores/uiStore";
import { ConnectionPorts } from "./ConnectionPorts";
import "../nodes.css";

interface BaseCompoundNodeProps extends NodeProps<Node<C4NodeData>> {
  icon: React.ReactNode;
  type: "system" | "container" | "component" | "person" | "datastore" | "queue";
  ports?: ("top" | "right" | "bottom" | "left")[];
}

function getTypeLabel(type: string): string {
  switch (type) {
    case "system":
      return "Software System";
    case "container":
      return "Container";
    case "component":
      return "Component";
    case "person":
      return "Person";
    case "datastore":
      return "Database";
    case "queue":
      return "Queue";
    default:
      return type;
  }
}

export function BaseCompoundNode({
  data,
  selected,
  icon,
  type,
  ports,
  width, // Added: React Flow passes measured/computed width
  height, // Added: React Flow passes measured/computed height
}: BaseCompoundNodeProps) {
  const nodeData = data as C4NodeData;
  const colors = getNodeColors(type, nodeData.isExternal);
  const tagStyles = getTagStyles(getNodeTags(nodeData));
  const isExternal = nodeData.isExternal === true;
  const isExpanded = nodeData.expanded === true;
  const hasChildren = (nodeData.childCount ?? 0) > 0;

  const toggleExpand = useViewStore((s) => s.toggleExpand);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  // Governance counts from enriched node data
  const requirementCount = isSystemNodeData(nodeData) || isContainerNodeData(nodeData)
    ? ((isSystemNodeData(nodeData) ? nodeData.requirementCount : (nodeData as ContainerNodeData).requirementCount) ?? 0)
    : 0;
  const adrCount = isSystemNodeData(nodeData) || isContainerNodeData(nodeData)
    ? ((isSystemNodeData(nodeData) ? nodeData.adrCount : (nodeData as ContainerNodeData).adrCount) ?? 0)
    : 0;
  const scenarioCount = isSystemNodeData(nodeData) || isContainerNodeData(nodeData)
    ? ((isSystemNodeData(nodeData) ? nodeData.scenarioCount : (nodeData as ContainerNodeData).scenarioCount) ?? 0)
    : 0;
  const flowCount = isSystemNodeData(nodeData) || isContainerNodeData(nodeData)
    ? ((isSystemNodeData(nodeData) ? nodeData.flowCount : (nodeData as ContainerNodeData).flowCount) ?? 0)
    : 0;
  const totalLinkedItems = requirementCount + adrCount + scenarioCount + flowCount;

  const handleExpandClick = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent node selection
    toggleExpand(nodeData.id);
  };

  const handleBadgeClick = (e: React.MouseEvent, tab: "requirements" | "adrs" | "scenarios" | "flows") => {
    e.stopPropagation();
    setActiveTab("details");
    // Small delay to ensure Details tab is mounted, then trigger sub-tab switch
    setTimeout(() => {
      const event = new CustomEvent("switch-details-subtab", { detail: tab });
      window.dispatchEvent(event);
    }, 100);
  };

  return (
    <div
      className={`c4-node ${type}-node ${selected ? "selected" : ""} ${isExternal ? "external" : ""} ${isExpanded ? "expanded" : ""}`}
      style={{
        backgroundColor: isExpanded ? undefined : colors.bg,
        borderColor: colors.border,
        color: colors.text,
        // CRITICAL: Use React Flow's computed dimensions for proper containment
        width: width || (isExpanded ? 400 : undefined),
        height: height || (isExpanded ? 300 : undefined),
        minWidth: isExpanded ? 400 : undefined,
        minHeight: isExpanded ? 300 : undefined,
        boxShadow: selected ? `0 0 0 2px ${colors.border}` : undefined,
        ...tagStyles,
      }}
    >
      <ConnectionPorts sides={ports} />

      <div className="node-header">
        <div className="node-icon">{icon}</div>

        <div className="node-content">
          <div className="node-label">{nodeData.label}</div>
          <div className="node-type-label">
            [{isExternal ? "External " : ""}
            {getTypeLabel(type)}]
          </div>
          {!isExpanded && nodeData.description && (
            <div className="node-description">{nodeData.description}</div>
          )}
          {!isExpanded && nodeData.technology && (
            <div className="node-technology">[{nodeData.technology}]</div>
          )}
        </div>

        {hasChildren && (
          <Button
            variant="ghost"
            size="sm"
            className="expand-btn"
            onClick={handleExpandClick}
            title={isExpanded ? "Collapse" : "Expand"}
          >
            {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
            <span className="child-count">{nodeData.childCount}</span>
          </Button>
        )}
      </div>

      {nodeData.pendingActionCount && nodeData.pendingActionCount > 0 && (
        <div
          className="node-warning-badge"
          aria-hidden="true"
          title={`${nodeData.pendingActionCount} pending action${nodeData.pendingActionCount > 1 ? "s" : ""} - Action required`}
        >
          <AlertTriangle size={12} />
          <span>{nodeData.pendingActionCount}</span>
        </div>
      )}
      {!isExpanded && hasChildren && (
        <div className="node-badge">
          {nodeData.childCount} item{(nodeData.childCount ?? 0) > 1 ? "s" : ""}
        </div>
      )}

      {/* Governance badges - show linked items */}
      {totalLinkedItems > 0 && !isExpanded && (
        <div className="governance-badges">
          {requirementCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="governance-badge requirement-badge"
              onClick={(e) => handleBadgeClick(e, "requirements")}
              title={`${requirementCount} requirement${requirementCount > 1 ? "s" : ""} - Click to view`}
              aria-label={`${requirementCount} requirement${requirementCount > 1 ? "s" : ""}`}
            >
              <Target size={10} />
              <span>{requirementCount}</span>
            </Button>
          )}
          {adrCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="governance-badge adr-badge"
              onClick={(e) => handleBadgeClick(e, "adrs")}
              title={`${adrCount} ADR${adrCount > 1 ? "s" : ""} - Click to view`}
              aria-label={`${adrCount} ADR${adrCount > 1 ? "s" : ""}`}
            >
              <FileText size={10} />
              <span>{adrCount}</span>
            </Button>
          )}
          {scenarioCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="governance-badge scenario-badge"
              onClick={(e) => handleBadgeClick(e, "scenarios")}
              title={`${scenarioCount} scenario${scenarioCount > 1 ? "s" : ""} - Click to view`}
              aria-label={`${scenarioCount} scenario${scenarioCount > 1 ? "s" : ""}`}
            >
              <Play size={10} />
              <span>{scenarioCount}</span>
            </Button>
          )}
          {flowCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="governance-badge flow-badge"
              onClick={(e) => handleBadgeClick(e, "flows")}
              title={`${flowCount} flow${flowCount > 1 ? "s" : ""} - Click to view`}
              aria-label={`${flowCount} flow${flowCount > 1 ? "s" : ""}`}
            >
              <Workflow size={10} />
              <span>{flowCount}</span>
            </Button>
          )}
        </div>
      )}

      {/* The content area for children is implicit for React Flow groups, 
                but we can create a visual frame if needed */}
    </div>
  );
}
