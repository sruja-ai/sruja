import { useMemo, useState, useEffect } from 'react'
import ReactFlow, { Background } from '@xyflow/react'
import { nodeTypes, edgeTypes, jsonToReactFlow, applySrujaLayout, Legend, type ArchitectureJSON } from '@sruja/diagram'

export function C4View({ architecture, level = 'L2', systemId, containerId }: { architecture: ArchitectureJSON; level?: 'L1' | 'L2' | 'L3'; systemId?: string; containerId?: string }) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())
  const initial = useMemo(() => jsonToReactFlow(architecture, { level, focusedSystemId: systemId, focusedContainerId: containerId, expandedNodes: expanded }), [architecture, level, systemId, containerId, expanded])
  const [nodes, setNodes] = useState(initial.nodes)
  const [edges, setEdges] = useState(initial.edges)

  useEffect(() => {
    const withHandlers = initial.nodes.map(n => ({
      ...n,
      data: {
        ...n.data, onToggleExpand: (id: string) => {
          const next = new Set(expanded)
          if (next.has(id)) next.delete(id); else next.add(id)
          setExpanded(next)
        }
      }
    }))
    const { nodes: n, edges: e } = applySrujaLayout(withHandlers, initial.edges, architecture, { level, focusedSystemId: systemId, focusedContainerId: containerId, direction: 'TB' })
    setNodes(n)
    setEdges(e)
  }, [initial, architecture, level, systemId, containerId])

  return (
    <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} edgeTypes={edgeTypes} fitView>
      <Background />
      <Legend />
    </ReactFlow>
  )
}
