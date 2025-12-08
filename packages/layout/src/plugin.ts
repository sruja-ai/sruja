export type LayoutAlgorithm = (input: any) => any
export type PostProcessor = (nodes: Map<string, any>, edges: any[]) => void

const algorithms = new Map<string, LayoutAlgorithm>()
const postprocessors: PostProcessor[] = []

export function registerAlgorithm(name: string, algorithm: LayoutAlgorithm) { algorithms.set(name, algorithm) }
export function getAlgorithm(name: string) { return algorithms.get(name) }
export function addPostProcessor(processor: PostProcessor) { postprocessors.push(processor) }
export function applyPostProcessors(nodes: Map<string, any>, edges: any[]) { for (const p of postprocessors) p(nodes, edges) }
