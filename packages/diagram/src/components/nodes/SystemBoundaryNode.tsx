// packages/react-flow-architecture/src/components/nodes/SystemBoundaryNode.tsx
// System boundary node component for C4 system boundaries
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react'
import { Server } from 'lucide-react'
import type { C4NodeData } from '../../types'
import { getNodeColors } from '../../utils/colorScheme'

type SystemBoundaryNodeProps = NodeProps<Node<C4NodeData>>


export function SystemBoundaryNode({ data, selected }: SystemBoundaryNodeProps) {
  const nodeData = data as C4NodeData
  const colors = getNodeColors('system-boundary', nodeData.isExternal)

  return (
    <div
      className={`c4-node system-boundary-node ${selected ? 'selected' : ''}`}
      style={{
        backgroundColor: 'transparent',
        borderColor: colors.border,
        color: colors.text,
        borderStyle: 'dashed',
        borderWidth: 1.5,
        position: 'relative',
      }}
    >
      <Handle type="target" position={Position.Top} className="c4-handle" />
      <div className="boundary-titlebar">
        <span className="icon"><Server size={14} /></span>
        <span className="title">{(nodeData as any)?.titleBar?.label ?? nodeData.label}</span>
        {typeof nodeData.childCount === 'number' && (
          <span className="count">{nodeData.childCount} {((nodeData as any)?.titleBar?.countLabel ?? 'containers')}</span>
        )}
      </div>
      <Handle type="source" position={Position.Bottom} className="c4-handle" />
    </div>
  )
}
