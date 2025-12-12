import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { User } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';

type PersonNodeProps = NodeProps<Node<C4NodeData>>;

export function PersonNode({ data, selected }: PersonNodeProps) {
    const nodeData = data as C4NodeData;
    const colors = getNodeColors('person', nodeData.isExternal);
    const isExternal = nodeData.isExternal === true;

    return (
        <div
            className={`c4-node person-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <div className="person-icon-circle">
                <User size={24} />
            </div>

            <div className="node-content">
                <div className="node-label">{nodeData.label}</div>
                {nodeData.description && (
                    <div className="node-description">{nodeData.description}</div>
                )}
            </div>

            <Handle type="source" position={Position.Bottom} className="c4-handle" />
        </div>
    );
}
