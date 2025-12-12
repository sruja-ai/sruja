import type { NodeProps, Node } from '@xyflow/react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import type { C4NodeData } from '../../../types';
import { getNodeColors, getTagStyles } from '../../../utils/colorScheme';
import { useViewStore } from '../../../stores';
import { ConnectionPorts } from './ConnectionPorts';
import '../nodes.css';

interface BaseCompoundNodeProps extends NodeProps<Node<C4NodeData>> {
    icon: React.ReactNode;
    type: 'system' | 'container' | 'component' | 'person' | 'datastore' | 'queue';
    ports?: ('top' | 'right' | 'bottom' | 'left')[];
}

function getTypeLabel(type: string): string {
    switch (type) {
        case 'system': return 'Software System';
        case 'container': return 'Container';
        case 'component': return 'Component';
        case 'person': return 'Person';
        case 'datastore': return 'Database';
        case 'queue': return 'Queue';
        default: return type;
    }
}

export function BaseCompoundNode({
    data,
    selected,
    icon,
    type,
    ports,
    width,   // Added: React Flow passes measured/computed width
    height   // Added: React Flow passes measured/computed height
}: BaseCompoundNodeProps) {
    const nodeData = data as C4NodeData;
    const colors = getNodeColors(type, nodeData.isExternal);
    const tagStyles = getTagStyles((nodeData as any).tags as string[] | undefined);
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
            className={`c4-node ${type}-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''} ${isExpanded ? 'expanded' : ''}`}
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
                <div className="node-icon">
                    {icon}
                </div>

                <div className="node-content">
                    <div className="node-label">{nodeData.label}</div>
                    <div className="node-type-label">
                        [{isExternal ? 'External ' : ''}{getTypeLabel(type)}]
                    </div>
                    {!isExpanded && nodeData.description && (
                        <div className="node-description">{nodeData.description}</div>
                    )}
                    {!isExpanded && nodeData.technology && (
                        <div className="node-technology">[{nodeData.technology}]</div>
                    )}
                </div>

                {hasChildren && (
                    <button
                        className="expand-btn"
                        onClick={handleExpandClick}
                        title={isExpanded ? 'Collapse' : 'Expand'}
                    >
                        {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
                        <span className="child-count">{nodeData.childCount}</span>
                    </button>
                )}
            </div>

            {!isExpanded && hasChildren && (
                <div className="node-badge">
                    {nodeData.childCount} item{(nodeData.childCount ?? 0) > 1 ? 's' : ''}
                </div>
            )}

            {/* The content area for children is implicit for React Flow groups, 
                but we can create a visual frame if needed */}
        </div>
    );
}
