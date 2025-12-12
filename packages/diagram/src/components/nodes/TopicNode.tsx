// packages/react-flow-architecture/src/components/nodes/TopicNode.tsx
// Topic node component for C4 message topics
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { MessageSquare } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';

type TopicNodeProps = NodeProps<Node<C4NodeData>>;

export function TopicNode({ data, selected }: TopicNodeProps) {
    const nodeData = data as C4NodeData;
    const colors = getNodeColors('queue', nodeData.isExternal); // Use queue colors for topics
    const isExternal = nodeData.isExternal === true;

    return (
        <div
            className={`c4-node topic-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <div className="node-icon">
                <MessageSquare size={20} />
            </div>

            <div className="node-content">
                <div className="node-label">{nodeData.label}</div>
                {nodeData.technology && (
                    <div className="node-technology">[{nodeData.technology}]</div>
                )}
            </div>

            <Handle type="source" position={Position.Bottom} className="c4-handle" />
        </div>
    );
}








