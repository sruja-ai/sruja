import React from "react";
import { Handle, Position } from "@xyflow/react";
import type { C4NodeData, C4NodeType } from "../../types";
import { getNodeColors } from "../../utils/colorScheme";
import { GovernanceBadge } from "./GovernanceBadge";

/**
 * Props for the BaseNode component
 * @template T - Extended node data type, defaults to C4NodeData
 */
export interface BaseNodeProps {
    /** Node data containing label, type, etc. */
    data: C4NodeData;
    /** Whether the node is currently selected */
    selected: boolean;
    /** The C4 node type for styling */
    nodeType: C4NodeType;
    /** Icon component to render */
    icon: React.ReactNode;
    /** Additional CSS class names */
    className?: string;
    /** Whether to show the description field */
    showDescription?: boolean;
    /** Whether to show the technology field */
    showTechnology?: boolean;
    /** Additional content to render after the node content */
    children?: React.ReactNode;
    /** Custom style overrides */
    style?: React.CSSProperties;
}

/**
 * Base node component that provides common structure for all C4 node types.
 * Eliminates ~80% code duplication across 15 node components.
 * 
 * Features:
 * - Consistent handle positioning (top/bottom)
 * - Governance badges for requirements/ADRs
 * - Color scheme based on node type
 * - External node styling support
 * - Selected state styling
 * 
 * @example
 * ```tsx
 * <BaseNode
 *   data={nodeData}
 *   selected={selected}
 *   nodeType="container"
 *   icon={<Box size={20} />}
 * />
 * ```
 */
export const BaseNode = React.memo(function BaseNode({
    data,
    selected,
    nodeType,
    icon,
    className = "",
    showDescription = true,
    showTechnology = true,
    children,
    style,
}: BaseNodeProps) {
    const colors = getNodeColors(nodeType, data.isExternal);
    const isExternal = data.isExternal === true;

    const nodeClassName = [
        "c4-node",
        `${nodeType}-node`,
        selected && "selected",
        isExternal && "external",
        className,
    ]
        .filter(Boolean)
        .join(" ");

    return (
        <div
            className={nodeClassName}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
                ...style,
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <GovernanceBadge
                requirementCount={data.requirementCount}
                adrCount={data.adrCount}
            />

            <div className="node-icon">{icon}</div>

            <div className="node-content">
                <div className="node-label">{data.label}</div>
                {showTechnology && data.technology && (
                    <div className="node-technology">[{data.technology}]</div>
                )}
                {showDescription && data.description && (
                    <div className="node-description">{data.description}</div>
                )}
            </div>

            {children}

            <Handle type="source" position={Position.Bottom} className="c4-handle" />
        </div>
    );
});

/**
 * Higher-order component for creating node components with BaseNode
 * Provides consistent structure while allowing customization
 */
export function createNodeComponent(
    nodeType: C4NodeType,
    icon: React.ReactNode,
    options: Partial<Pick<BaseNodeProps, "showDescription" | "showTechnology" | "className">> = {}
) {
    const NodeComponent = React.memo(function NodeComponent({
        data,
        selected,
    }: {
        data: C4NodeData;
        selected: boolean;
    }) {
        return (
            <BaseNode
                data={data}
                selected={selected}
                nodeType={nodeType}
                icon={icon}
                {...options}
            />
        );
    });

    NodeComponent.displayName = `${nodeType.charAt(0).toUpperCase() + nodeType.slice(1)}Node`;
    return NodeComponent;
}
