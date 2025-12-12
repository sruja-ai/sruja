// packages/react-flow-architecture/src/components/nodes/CacheNode.tsx
// Cache node component for C4 cache datastores
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Zap } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';

type CacheNodeProps = NodeProps<Node<C4NodeData>>;

export function CacheNode({ data, selected }: CacheNodeProps) {
    const nodeData = data as C4NodeData;
    const colors = getNodeColors('datastore', nodeData.isExternal); // Use datastore colors for cache
    const isExternal = nodeData.isExternal === true;

    return (
        <div
            className={`c4-node cache-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <div className="node-icon">
                <Zap size={20} />
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








