import type { ArchitectureJSON } from './types'
import type { ElementDefinition } from 'cytoscape'

export function convertToCytoscape(data: ArchitectureJSON): ElementDefinition[] {
  const elements: ElementDefinition[] = []
  const nodeIds = new Set<string>()
  const arch = data.architecture || {}
  const layout = (data.metadata as any)?.layout || {}

  const getPos = (id: string) => {
    if (layout[id]) {
      return { x: layout[id].x, y: layout[id].y }
    }
    return undefined
  }

  if (arch.persons) {
    for (const person of arch.persons) {
      nodeIds.add(person.id)
      elements.push({
        data: { id: person.id, label: person.label || person.id, type: 'person', metadata: person.metadata },
        position: getPos(person.id)
      })
    }
  }

  if (arch.systems) {
    for (const system of arch.systems) {
      nodeIds.add(system.id)
      elements.push({
        data: { id: system.id, label: system.label || system.id, type: 'system', metadata: system.metadata },
        position: getPos(system.id)
      })
      if (system.containers) {
        for (const container of system.containers) {
          const containerId = `${system.id}.${container.id}`
          nodeIds.add(containerId)
          elements.push({
            data: { id: containerId, label: container.label || container.id, type: 'container', parent: system.id, metadata: container.metadata },
            position: getPos(containerId)
          })
        }
      }
      if (system.datastores) {
        for (const ds of system.datastores) {
          const dsId = `${system.id}.${ds.id}`
          nodeIds.add(dsId)
          elements.push({
            data: { id: dsId, label: ds.label || ds.id, type: 'datastore', parent: system.id, metadata: ds.metadata },
            position: getPos(dsId)
          })
        }
      }
      if (system.queues) {
        for (const queue of system.queues) {
          const qId = `${system.id}.${queue.id}`
          nodeIds.add(qId)
          elements.push({
            data: { id: qId, label: queue.label || queue.id, type: 'queue', parent: system.id, metadata: queue.metadata },
          })
        }
      }
    }
  }

  if (arch.containers) {
    for (const container of arch.containers) {
      nodeIds.add(container.id)
      elements.push({
        data: { id: container.id, label: container.label || container.id, type: 'container', metadata: container.metadata },
      })
    }
  }

  if (arch.datastores) {
    for (const ds of arch.datastores) {
      nodeIds.add(ds.id)
      elements.push({
        data: { id: ds.id, label: ds.label || ds.id, type: 'datastore', metadata: ds.metadata },
      })
    }
  }

  if (arch.queues) {
    for (const queue of arch.queues) {
      nodeIds.add(queue.id)
      elements.push({
        data: { id: queue.id, label: queue.label || queue.id, type: 'queue', metadata: queue.metadata },
      })
    }
  }

  if (arch.requirements) {
    for (const req of arch.requirements) {
      nodeIds.add(req.id)
      elements.push({
        data: { id: req.id, label: req.title || req.id, type: 'requirement', metadata: req.metadata as any },
        position: getPos(req.id)
      })
    }
  }

  if (arch.adrs) {
    for (const adr of arch.adrs) {
      nodeIds.add(adr.id)
      elements.push({
        data: { id: adr.id, label: adr.title || adr.id, type: 'adr', metadata: adr.metadata as any },
        position: getPos(adr.id)
      })
    }
  }

  if (arch.deployment) {
    for (const node of arch.deployment) {
      nodeIds.add(node.id)
      elements.push({
        data: { id: node.id, label: node.label || node.id, type: 'deployment', metadata: node.metadata as any },
        position: getPos(node.id)
      })
    }
  }

  const resolveNodeId = (id: string): string | null => {
    if (nodeIds.has(id)) return id
    if (arch.systems) {
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

  if (arch.relations) {
    for (const relation of arch.relations) {
      const fromId = resolveNodeId(relation.from)
      const toId = resolveNodeId(relation.to)
      if (fromId && toId) {
        elements.push({
          data: { id: `${fromId}-${toId}`, source: fromId, target: toId, label: relation.label || relation.verb || '' },
        })
      }
    }
  }

  if (arch.systems) {
    for (const system of arch.systems) {
      if (system.relations) {
        for (const relation of system.relations) {
          const fromId = `${system.id}.${relation.from}`
          const toId = `${system.id}.${relation.to}`
          if (nodeIds.has(fromId) && nodeIds.has(toId)) {
            elements.push({
              data: { id: `${fromId}-${toId}`, source: fromId, target: toId, label: relation.label || relation.verb || '' },
            })
          }
        }
      }
    }
  }

  return elements
}

