// packages/react-flow-architecture/src/components/nodes/EnterpriseBoundaryNode.tsx
// Enterprise boundary node component for C4 enterprise boundaries
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Building2 } from 'lucide-react';
import type { C4NodeData } from '../../types';

type EnterpriseBoundaryNodeProps = NodeProps<Node<C4NodeData>>;

export function EnterpriseBoundaryNode({ data, selected }: EnterpriseBoundaryNodeProps) {
    const nodeData = data as C4NodeData;
    // Enterprise boundaries use a distinct darker color scheme
    const colors = {
        border: '#1E40AF', // Darker blue for enterprise
        text: '#1E40AF',
    };

    return (
        <div
            className={`c4-node enterprise-boundary-node ${selected ? 'selected' : ''}`}
            style={{
                backgroundColor: 'transparent',
                borderColor: colors.border,
                color: colors.text,
                borderStyle: 'dashed',
                borderWidth: 2,
                position: 'relative',
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />
            <div className="boundary-titlebar">
                <span className="icon"><Building2 size={14} /></span>
                <span className="title">{(nodeData as any)?.titleBar?.label ?? nodeData.label}</span>
                {typeof nodeData.childCount === 'number' && (
                    <span className="count">{nodeData.childCount} {((nodeData as any)?.titleBar?.countLabel ?? 'systems')}</span>
                )}
            </div>
            <Handle type="source" position={Position.Bottom} className="c4-handle" />
        </div>
    );
}








