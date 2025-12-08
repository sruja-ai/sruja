import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Building2, ChevronDown, ChevronUp } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';
import { useViewStore } from '../../stores';
import './nodes.css';

type SystemNodeProps = NodeProps<Node<C4NodeData>>;

export function SystemNode({ data, selected }: SystemNodeProps) {
    const nodeData = data as C4NodeData;
    const colors = getNodeColors('system', nodeData.isExternal);
    const isExternal = nodeData.isExternal === true;
    const isExpanded = nodeData.expanded === true;
    const hasChildren = (nodeData.childCount ?? 0) > 0;

    const toggleExpand = useViewStore((s) => s.toggleExpand);

    const handleExpandClick = (e: React.MouseEvent) => {
        e.stopPropagation(); // Prevent node selection
        toggleExpand(nodeData.id);
    };

    return (
        <div
            className={`c4-node system-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''} ${isExpanded ? 'expanded' : ''}`}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <div className="node-header">
                <div className="node-icon">
                    <Building2 size={24} />
                </div>

                <div className="node-content">
                    <div className="node-label">{nodeData.label}</div>
                    {nodeData.description && (
                        <div className="node-description">{nodeData.description}</div>
                    )}
                </div>

                {hasChildren && (
                    <button
                        className="expand-btn"
                        onClick={handleExpandClick}
                        title={isExpanded ? 'Collapse' : 'Expand containers'}
                    >
                        {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
                        <span className="child-count">{nodeData.childCount}</span>
                    </button>
                )}
            </div>

            {!isExpanded && hasChildren && (
                <div className="node-badge">
                    {nodeData.childCount} container{(nodeData.childCount ?? 0) > 1 ? 's' : ''}
                </div>
            )}

            <Handle type="source" position={Position.Bottom} className="c4-handle" />
        </div>
    );
}

