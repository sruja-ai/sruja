
import { describe, it, expect } from 'vitest'
import { layout } from '../c4-layout'
import { createC4Graph } from '../c4-model'
import { LandscapeView } from '../c4-view'
import { InteractivePreset } from '../c4-options'
import type { C4Node } from '../c4-model'
import type { C4Id } from '../brand'

describe('Deep Recursion Layout', () => {
    it('positions grandchildren correctly in grid layout', () => {
        // Grandchild (Component)
        const component: C4Node = {
            id: 'component1' as C4Id,
            kind: 'Component',
            label: 'Component 1',
            level: 'component',
            parentId: 'container1' as C4Id,
            tags: new Set()
        }

        // Child (Container)
        const container: C4Node = {
            id: 'container1' as C4Id,
            kind: 'Container',
            label: 'Container 1',
            level: 'container',
            parentId: 'system1' as C4Id,
            tags: new Set()
        }

        // Parent (System)
        const system: C4Node = {
            id: 'system1' as C4Id,
            kind: 'SoftwareSystem',
            label: 'System 1',
            level: 'context',
            tags: new Set()
        }

        const nodes = [system, container, component]
        const graph = createC4Graph(nodes, [])
        const view = LandscapeView()

        // Force include all kinds so hierarchy includes them
        if (view.filter && view.filter.includeKinds) {
            (view.filter.includeKinds as Set<string>).add('Container');
            (view.filter.includeKinds as Set<string>).add('Component');
        }

        // Use InteractivePreset which uses 'grid' strategy (via inheritance from Publication, but wait used to be grid/sugiyama depending on preset)
        // Actually InteractivePreset -> PublicationPreset. Publication uses ...?
        // Let's force grid strategy
        const options = { ...InteractivePreset, strategy: 'grid' as const }

        const result = layout(graph, view, options)

        // Check System position
        const sysPos = result.nodes.get('system1')
        expect(sysPos).toBeDefined()

        // Check Container position (should be inside System)
        const contPos = result.nodes.get('container1')
        expect(contPos).toBeDefined()
        expect(contPos!.bbox.x).toBeGreaterThanOrEqual(sysPos!.bbox.x)
        expect(contPos!.bbox.y).toBeGreaterThanOrEqual(sysPos!.bbox.y)

        // Check Component position (should be inside Container)
        const compPos = result.nodes.get('component1')
        expect(compPos).toBeDefined()

        // Ensure Component is NOT at 0,0 (unless parent is at 0,0, but even then padding should apply)
        // Actually, verifying it is strictly inside container
        expect(compPos!.bbox.x).toBeGreaterThanOrEqual(contPos!.bbox.x)
        expect(compPos!.bbox.y).toBeGreaterThanOrEqual(contPos!.bbox.y)

        // Also check if they are not all overlapping exactly
        expect(compPos!.bbox.x).not.toBe(sysPos!.bbox.x) // Component shouldn't be exactly at System top-left (padding)
    })
})
