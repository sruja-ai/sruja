import { type NodeProps, type Node } from '@xyflow/react';
import type { C4NodeData } from '../../types';
import './nodes.css';

type BoundaryNodeProps = NodeProps<Node<C4NodeData>>;

export function BoundaryNode({ data, selected }: BoundaryNodeProps) {
    return (
        <div className={`c4-node boundary-node ${selected ? 'selected' : ''}`}>
            <div className="boundary-label">{data.label}</div>
            {/* Handles are needed for edges to pass through? No, group nodes don't usually have handles unless connecting TO the group */}
            {/* But React Flow might warn if missing handles? No. */}
        </div>
    );
}
