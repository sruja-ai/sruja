// packages/react-flow-architecture/src/components/nodes/DeploymentNode.tsx
// Deployment node component for C4 deployment diagrams
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Cloud } from 'lucide-react';
import type { C4NodeData } from '../../types';

type DeploymentNodeProps = NodeProps<Node<C4NodeData>>;

export function DeploymentNode({ data, selected }: DeploymentNodeProps) {
    const nodeData = data as C4NodeData;
    // Deployment nodes use a distinct color scheme (darker blue-gray)
    const colors = {
        bg: '#6B7280',
        border: '#4B5563',
        text: '#FFFFFF',
    };
    const isExternal = nodeData.isExternal === true;

    return (
        <div
            className={`c4-node deployment-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
                borderStyle: 'dashed', // Deployment nodes often use dashed borders
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <div className="node-icon">
                <Cloud size={20} />
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








