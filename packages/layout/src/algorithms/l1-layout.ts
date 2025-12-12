import type { HierarchyTree, HierarchyNode } from './hierarchy'
import type { C4LayoutOptions } from '../c4-options'
import type { PositionedC4Node } from '../c4-layout'
import type { Point } from '../geometry/point'
import type { Rect } from '../geometry/rect'


interface Size { width: number; height: number }

export function layoutL1SystemContext(
    tree: HierarchyTree,
    sizes: Map<string, Size>,
    _relationships: { from: string; to: string }[],
    _options: C4LayoutOptions
): Map<string, PositionedC4Node> {
    const positions = new Map<string, PositionedC4Node>()

    // 1. Identify "Main System" vs "Satellites"
    // Candidates for Main System: High-level SoftwareSystems that are NOT external
    const systems: HierarchyNode[] = []
    const satellites: HierarchyNode[] = []

    for (const root of tree.roots) {
        // If it's a generic SoftwareSystem and not marked external -> Main Candidates
        // Note: C4Kind 'SoftwareSystem' is generic. 'ExternalSystem' is explicit.
        if (root.node.kind === 'SoftwareSystem') {
            systems.push(root)
        } else {
            satellites.push(root)
        }
    }

    // Fallback: if no systems, treat everything as satellites (or circle layout)
    if (systems.length === 0 && satellites.length > 0) {
        // Treat the largest node as center?? Or just use circle layout for all?
        // Let's use circle layout for all if no clear center.
        layoutCircle(tree.roots, positions, sizes, { x: 0, y: 0 })
        return positions
    }

    // 2. Place Main System(s) in Center
    // If multiple systems, arrange them in a simple row/grid in the center
    const centerGroup = layoutCentralCluster(systems, sizes, positions)

    // 3. Place Satellites in Orbit
    if (satellites.length > 0) {
        layoutSatellitesRadial(satellites, centerGroup, sizes, positions, _relationships)
    }

    return positions
}

function layoutCentralCluster(
    nodes: HierarchyNode[],
    sizes: Map<string, Size>,
    positions: Map<string, PositionedC4Node>
): Rect {
    // Simple horizontal row for now (improve to grid if > 3 systems?)
    const spacing = 100
    let totalWidth = 0
    let maxHeight = 0

    for (const node of nodes) {
        const s = sizes.get(node.id) || { width: 200, height: 150 }
        totalWidth += s.width
        maxHeight = Math.max(maxHeight, s.height)
    }
    totalWidth += (nodes.length - 1) * spacing

    let currentX = -(totalWidth / 2)
    const centerY = 0

    for (const node of nodes) {
        const s = sizes.get(node.id) || { width: 200, height: 150 }
        const x = currentX
        const y = centerY - s.height / 2

        positions.set(node.id, createPositionedNode(node, { x, y, ...s }))
        currentX += s.width + spacing
    }

    return {
        x: -(totalWidth / 2),
        y: -(maxHeight / 2),
        width: totalWidth,
        height: maxHeight
    }
}

function layoutSatellitesRadial(
    nodes: HierarchyNode[],
    centerBBox: Rect,
    sizes: Map<string, Size>,
    positions: Map<string, PositionedC4Node>,
    _relationships: { from: string; to: string }[]
) {
    // Sort satellites by type (Person vs System) or connectivity to keep diagram tidy
    // Heuristic: Group Persons together, External Systems together
    nodes.sort((a, b) => {
        const k = a.node.kind.localeCompare(b.node.kind)
        return k !== 0 ? k : (a.node.id as string).localeCompare(b.node.id as string)
    })

    const count = nodes.length
    // Radius calculation:
    // Must be large enough to clear the center box corners + satellite sizes
    const maxSatelliteDim = nodes.reduce((max, node) => {
        const s = sizes.get(node.id) || { width: 0, height: 0 }
        return Math.max(max, Math.sqrt(s.width * s.width + s.height * s.height))
    }, 0)

    const centerDiag = Math.sqrt(centerBBox.width * centerBBox.width + centerBBox.height * centerBBox.height) / 2
    const minRadius = centerDiag + maxSatelliteDim + 100 // + padding

    // Arc distribution:
    // If we have few nodes, we might not want a full 360 circle.
    // But standard "Star" usually distributes evenly.
    // 0 degrees is "East".
    // Let's start from -90 (North) and go around.

    const angleStep = (2 * Math.PI) / count
    let currentAngle = -Math.PI / 2 // Start North

    for (const node of nodes) {
        const s = sizes.get(node.id) || { width: 150, height: 100 }

        // Position center of satellite
        const cx = Math.cos(currentAngle) * minRadius
        const cy = Math.sin(currentAngle) * minRadius

        // Convert to top-left for rect
        const x = cx - s.width / 2
        const y = cy - s.height / 2

        positions.set(node.id, createPositionedNode(node, { x, y, width: s.width, height: s.height }))

        currentAngle += angleStep
    }
}

function layoutCircle(
    nodes: HierarchyNode[],
    positions: Map<string, PositionedC4Node>,
    sizes: Map<string, Size>,
    center: Point
) {
    // Just a generic circle layout if no clear center
    const count = nodes.length
    const radius = count * 150 / (2 * Math.PI) + 200 // heuristic

    let angle = -Math.PI / 2
    const step = (2 * Math.PI) / count

    for (const node of nodes) {
        const s = sizes.get(node.id) || { width: 150, height: 100 }
        const cx = center.x + Math.cos(angle) * radius
        const cy = center.y + Math.sin(angle) * radius

        positions.set(node.id, createPositionedNode(node, {
            x: cx - s.width / 2,
            y: cy - s.height / 2,
            width: s.width,
            height: s.height
        }))
        angle += step
    }
}

function createPositionedNode(hNode: HierarchyNode, rect: Rect): PositionedC4Node {
    return {
        nodeId: hNode.id,
        bbox: rect,
        contentBox: rect,
        labelBox: rect,
        parentId: undefined,
        childrenIds: [], // L1 implies we collapsed children or they are not shown
        depth: 0,
        level: 'context',
        collapsed: false, // L1 nodes are usually collapsed in context view? Spec implies "High level boxes"
        visible: true,
        zIndex: 0,
        ports: []
    }
}
