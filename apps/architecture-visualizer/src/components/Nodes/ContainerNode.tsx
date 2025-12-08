import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { Box } from 'lucide-react';
import type { C4NodeData } from '../../types';
import { getNodeColors } from '../../utils/colorScheme';
import './nodes.css';

type ContainerNodeProps = NodeProps<Node<C4NodeData>>;

export function ContainerNode({ data, selected }: ContainerNodeProps) {
  const nodeData = data as C4NodeData;
  const colors = getNodeColors('container', nodeData.isExternal);
  const isExternal = nodeData.isExternal === true;

  return (
    <div
      className={`c4-node container-node ${selected ? 'selected' : ''} ${isExternal ? 'external' : ''}`}
      style={{
        backgroundColor: colors.bg,
        borderColor: colors.border,
        color: colors.text,
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
        {nodeData.childCount !== undefined && nodeData.childCount > 0 && (
          <div className="node-badge">
            {nodeData.childCount} component{nodeData.childCount > 1 ? 's' : ''}
          </div>
        )}
      </div>

      <Handle type="source" position={Position.Bottom} className="c4-handle" />
    </div>
  );
}
