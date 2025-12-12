// packages/react-flow-architecture/src/components/nodes/FileSystemNode.tsx
// File system node component for C4 file system datastores
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Folder } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';

type FileSystemNodeProps = NodeProps<Node<C4NodeData>>;

export function FileSystemNode({ data, selected }: FileSystemNodeProps) {
    const nodeData = data as C4NodeData;
    const colors = getNodeColors('datastore', nodeData.isExternal); // Use datastore colors
    const isExternal = nodeData.isExternal === true;

    return (
        <div
            className={`c4-node filesystem-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
            style={{
                backgroundColor: colors.bg,
                borderColor: colors.border,
                color: colors.text,
            }}
        >
            <Handle type="target" position={Position.Top} className="c4-handle" />

            <div className="node-icon">
                <Folder size={20} />
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








