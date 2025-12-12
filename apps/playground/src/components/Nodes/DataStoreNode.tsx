import { type NodeProps, type Node } from '@xyflow/react';
import { Database } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';
import { ConnectionPorts } from './BaseNode/ConnectionPorts';
import './nodes.css';

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
            {/* Databases typically receive connections from above */}
            <ConnectionPorts sides={['top', 'left', 'right']} />

            <div className="node-icon">
                <Database size={20} />
            </div>

            <div className="node-content">
                <div className="node-label">{nodeData.label}</div>
                {nodeData.technology && (
                    <div className="node-technology">[{nodeData.technology}]</div>
                )}
            </div>
        </div>
    );
}
