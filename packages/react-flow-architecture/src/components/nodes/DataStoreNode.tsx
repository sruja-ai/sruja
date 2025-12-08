import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Database } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';

type DataStoreNodeProps = NodeProps<Node<C4NodeData>>;

export function DataStoreNode({ data, selected }: DataStoreNodeProps) {
    const nodeData = data as C4NodeData;
    const colors = getNodeColors('datastore', nodeData.isExternal);
    const isExternal = nodeData.isExternal === true;

    return (
        <div
            className={`c4-node datastore-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <div className="node-icon">
                <Database size={20} />
            </div>

            <div className="node-content">
                <div className="node-label">{nodeData.label}</div>
            </div>

            <Handle type="source" position={Position.Bottom} className="c4-handle" />
        </div>
    );
}
