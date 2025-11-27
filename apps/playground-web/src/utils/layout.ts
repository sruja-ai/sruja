// apps/playground-web/src/utils/layout.ts
import ELK from 'elkjs'

export type LayoutAlgorithm = 'elk' | 'mermaid-default'

export interface LayoutOptions {
  algorithm: LayoutAlgorithm
  direction?: 'TB' | 'BT' | 'LR' | 'RL'
  nodeSpacing?: number
  edgeSpacing?: number
  rankSpacing?: number
}

const elk = new ELK()

export async function optimizeLayout(
  mermaidCode: string,
  options: LayoutOptions = { algorithm: 'elk' }
): Promise<string> {
  if (options.algorithm === 'mermaid-default') {
    return mermaidCode
  }

  try {
    // Parse Mermaid code to extract graph structure
    const graph = parseMermaidToGraph(mermaidCode)
    
    if (graph.nodes.length === 0) {
      return mermaidCode
    }
    
    // Apply layout algorithms to analyze and optimize
    // Note: Mermaid handles its own rendering, but we can pre-process
    // the structure for better visual organization
    if (options.algorithm === 'elk') {
      await applyELKLayout(graph, mermaidCode, options)
    }
    
    // Return optimized code (currently returns original as Mermaid handles layout)
    // Future: Could inject layout hints or reorganize structure
    return mermaidCode
  } catch (err) {
    console.warn('Layout optimization failed, using default:', err)
    return mermaidCode
  }
}

interface GraphNode {
  id: string
  label: string
  type: string
  width?: number
  height?: number
}

interface GraphEdge {
  from: string
  to: string
  label?: string
}

interface Graph {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

function parseMermaidToGraph(mermaidCode: string): Graph {
  const nodes: GraphNode[] = []
  const edges: GraphEdge[] = []
  const nodeMap = new Map<string, GraphNode>()

  // Parse C4Context diagrams
  const lines = mermaidCode.split('\n').map(l => l.trim()).filter(l => l)
  
  for (const line of lines) {
    // Match node declarations: Person(id, "label"), System(id, "label"), etc.
    const nodeMatch = line.match(/(Person|System|Container|Component|SystemDb|SystemQueue|SystemExt)\(([^,]+),\s*"([^"]+)"(?:,\s*"([^"]+)")?\)/)
    if (nodeMatch) {
      const [, type, id, label, tech] = nodeMatch
      const node: GraphNode = {
        id: id.trim(),
        label: label,
        type,
        width: estimateNodeWidth(label, tech),
        height: estimateNodeHeight(type),
      }
      nodes.push(node)
      nodeMap.set(id.trim(), node)
      continue
    }

    // Match relations: Rel(from, to, "label")
    const relMatch = line.match(/Rel\(([^,]+),\s*([^,]+),\s*"([^"]+)"\)/)
    if (relMatch) {
      const [, from, to, label] = relMatch
      edges.push({
        from: from.trim(),
        to: to.trim(),
        label: label,
      })
      continue
    }
  }

  return { nodes, edges }
}

function estimateNodeWidth(label: string, tech?: string): number {
  const baseWidth = Math.max(label.length * 8, 120)
  return tech ? baseWidth + 40 : baseWidth
}

function estimateNodeHeight(type: string): number {
  const heights: Record<string, number> = {
    Person: 60,
    System: 80,
    Container: 100,
    Component: 80,
    SystemDb: 100,
    SystemQueue: 100,
    SystemExt: 80,
  }
  return heights[type] || 80
}

async function applyELKLayout(
  graph: Graph,
  originalCode: string,
  options: LayoutOptions
): Promise<string> {
  const elkGraph = {
    id: 'root',
    layoutOptions: {
      'elk.algorithm': 'layered',
      'elk.direction': options.direction || 'DOWN',
      'elk.spacing.nodeNode': String(options.nodeSpacing || 80),
      'elk.spacing.edgeNode': String(options.edgeSpacing || 40),
      'elk.spacing.edgeEdge': String(options.edgeSpacing || 20),
      'elk.layered.spacing.nodeNodeBetweenLayers': String(options.rankSpacing || 100),
      'elk.layered.nodePlacement.strategy': 'SIMPLE',
      'elk.layered.crossingMinimization.strategy': 'INTERACTIVE',
    },
    children: graph.nodes.map(node => ({
      id: node.id,
      width: node.width || 120,
      height: node.height || 80,
      labels: [{ text: node.label }],
    })),
    edges: graph.edges.map(edge => ({
      id: `${edge.from}-${edge.to}`,
      sources: [edge.from],
      targets: [edge.to],
      labels: edge.label ? [{ text: edge.label }] : [],
    })),
  }

  const layouted = await elk.layout(elkGraph)

  // Reorganize Mermaid code based on layout hierarchy
  // Group nodes by their layer/rank for better visual organization
  if (layouted.children) {
    const nodeMap = new Map(graph.nodes.map(n => [n.id, n]))
    const edgeMap = new Map(graph.edges.map(e => [`${e.from}-${e.to}`, e]))
    
    // Sort nodes by their Y position (layer)
    const sortedNodes = [...layouted.children].sort((a, b) => {
      const aY = a.y || 0
      const bY = b.y || 0
      return aY - bY
    })
    
    // Reconstruct code with optimized order
    return reconstructMermaidCode(sortedNodes, edgeMap, nodeMap, originalCode)
  }

  return originalCode
}

function reconstructMermaidCode(
  sortedNodes: Array<{ id: string; y?: number }>,
  _edgeMap: Map<string, GraphEdge>,
  nodeMap: Map<string, GraphNode>,
  originalCode: string
): string {
  // Extract header (C4Context, title, etc.)
  const lines = originalCode.split('\n')
  const header: string[] = []
  const nodeLines: string[] = []
  const edgeLines: string[] = []
  
  for (const line of lines) {
    const trimmed = line.trim()
    if (trimmed.startsWith('C4Context') || trimmed.startsWith('title')) {
      header.push(line)
    } else if (trimmed.match(/^(Person|System|Container|Component|SystemDb|SystemQueue|SystemExt)\(/)) {
      nodeLines.push(line)
    } else if (trimmed.startsWith('Rel(')) {
      edgeLines.push(line)
    }
  }
  
  // Reorder nodes based on layout
  const orderedNodeLines: string[] = []
  const processedNodes = new Set<string>()
  
  for (const { id } of sortedNodes) {
    const node = nodeMap.get(id)
    if (node) {
      // Find the original line for this node
      const nodeLine = nodeLines.find(line => 
        line.includes(`(${id},`) || line.includes(`(${id} `)
      )
      if (nodeLine && !processedNodes.has(id)) {
        orderedNodeLines.push(nodeLine)
        processedNodes.add(id)
      }
    }
  }
  
  // Add any remaining nodes not in sorted list
  for (const line of nodeLines) {
    if (!orderedNodeLines.includes(line)) {
      orderedNodeLines.push(line)
    }
  }
  
  // Reconstruct with optimized order
  return [
    ...header,
    ...orderedNodeLines,
    ...edgeLines,
  ].join('\n')
}

