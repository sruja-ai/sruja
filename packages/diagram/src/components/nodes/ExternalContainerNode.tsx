// packages/react-flow-architecture/src/components/nodes/ExternalContainerNode.tsx
// External container node component for C4 external containers
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Box } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';

type ExternalContainerNodeProps = NodeProps<Node<C4NodeData>>;

export function ExternalContainerNode({ data, selected }: ExternalContainerNodeProps) {
    const nodeData = data as C4NodeData;
    const colors = getNodeColors('container', true); // Force external styling

    return (
        <div
            className={`c4-node external-container-node ${selected ? 'selected' : ''} external`}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
                borderStyle: 'dashed', // External elements use dashed borders
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <div className="node-icon">
                <Box size={20} />
            </div>

            <div className="node-content">
                <div className="node-label">{nodeData.label}</div>
                {nodeData.technology && (
                    <div className="node-technology">[{nodeData.technology}]</div>
                )}
                {nodeData.description && (
                    <div className="node-description">{nodeData.description}</div>
                )}
            </div>

            <Handle type="source" position={Position.Bottom} className="c4-handle" />
        </div>
    );
}








