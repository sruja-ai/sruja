// packages/react-flow-architecture/src/components/nodes/ContainerBoundaryNode.tsx
// Container boundary node component for C4 container boundaries
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react'
import { Layers } from 'lucide-react'
import type { C4NodeData } from '../../types'
import { getNodeColors } from '../../utils/colorScheme'

type ContainerBoundaryNodeProps = NodeProps<Node<C4NodeData>>

export function ContainerBoundaryNode({ data, selected }: ContainerBoundaryNodeProps) {
  const nodeData = data as C4NodeData
  const colors = getNodeColors('container')
  return (
    <div
      className={`c4-node container-boundary-node ${selected ? 'selected' : ''}`}
      style={{
        backgroundColor: 'transparent',
        borderColor: colors.border,
        color: colors.text,
        borderStyle: 'dashed',
        borderWidth: 1,
        position: 'relative',
      }}
    >
      <Handle type="target" position={Position.Top} className="c4-handle" />
      <div className="boundary-titlebar">
        <span className="icon"><Layers size={12} /></span>
        <span className="title">{(nodeData as any)?.titleBar?.label ?? nodeData.label}</span>
        {typeof nodeData.childCount === 'number' && (
          <span className="count">{nodeData.childCount} {((nodeData as any)?.titleBar?.countLabel ?? 'components')}</span>
        )}
      </div>
      <Handle type="source" position={Position.Bottom} className="c4-handle" />
    </div>
  )
}
