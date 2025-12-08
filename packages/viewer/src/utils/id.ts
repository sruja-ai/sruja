import type { ArchitectureJSON } from '../types'

export function resolveNodeId(id: string, arch: ArchitectureJSON['architecture'], nodeIds: Set<string>): string | null {
  if (nodeIds.has(id)) return id
  if (arch?.systems) {
    for (const system of arch.systems) {
      const qualifiedId = `${system.id}.${id}`
      if (nodeIds.has(qualifiedId)) return qualifiedId
      if (system.containers) {
        for (const container of system.containers) {
          if (container.id === id) return `${system.id}.${id}`
          if (container.components) {
            for (const component of container.components) {
              if (component.id === id) return `${system.id}.${container.id}.${id}`
            }
          }
        }
      }
    }
  }
  return null
}

