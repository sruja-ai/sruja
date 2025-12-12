
import { layoutL1SystemContext } from './src/algorithms/l1-layout.ts';
import { buildHierarchy } from './src/algorithms/hierarchy.ts';
import { createC4Graph, C4Node, C4Relationship } from './src/c4-model.ts';
import { C4Id } from './src/brand.ts';

const createId = (id: string) => id as C4Id;

function runLayout(expanded: boolean) {
    console.log(`\n--- Running Layout (Expanded: ${expanded}) ---`);

    // 1. Create Nodes
    const nodes: C4Node[] = [
        { id: createId('MainSystem'), kind: 'SoftwareSystem', label: 'Main', level: 'context', tags: new Set() },
        { id: createId('Ext1'), kind: 'ExternalSystem', label: 'Ext1', level: 'context', tags: new Set() },
        { id: createId('Ext2'), kind: 'ExternalSystem', label: 'Ext2', level: 'context', tags: new Set() },
        { id: createId('Person1'), kind: 'Person', label: 'User', level: 'context', tags: new Set() },
    ];

    if (expanded) {
        // Add containers
        nodes.push(
            { id: createId('Container1'), kind: 'Container', label: 'C1', level: 'container', parentId: createId('MainSystem'), tags: new Set() },
            { id: createId('Container2'), kind: 'Container', label: 'C2', level: 'container', parentId: createId('MainSystem'), tags: new Set() }
        );
    }

    const relationships: C4Relationship[] = []; // No rels needed for L1 layout test

    const graph = createC4Graph(nodes, relationships);
    const tree = buildHierarchy(graph);

    // Mock Sizes
    const sizes = new Map<string, { width: number, height: number }>();
    sizes.set('MainSystem', expanded ? { width: 800, height: 600 } : { width: 250, height: 180 });
    sizes.set('Ext1', { width: 200, height: 150 });
    sizes.set('Ext2', { width: 200, height: 150 });
    sizes.set('Person1', { width: 150, height: 100 });
    if (expanded) {
        sizes.set('Container1', { width: 200, height: 150 });
        sizes.set('Container2', { width: 200, height: 150 });
    }

    // Run Layout
    const positions = layoutL1SystemContext(tree, sizes, [], {});

    // Analyze Satellites
    ['Ext1', 'Ext2', 'Person1'].forEach(id => {
        const pos = positions.get(id);
        if (pos) {
            const angle = Math.atan2(pos.bbox.y + pos.bbox.height / 2, pos.bbox.x + pos.bbox.width / 2);
            // Normalize angle to degrees
            const deg = (angle * 180 / Math.PI).toFixed(2);
            console.log(`Node ${id}: x=${pos.bbox.x.toFixed(0)}, y=${pos.bbox.y.toFixed(0)}, Angle=${deg} deg`);
        } else {
            console.log(`Node ${id}: MISSING`);
        }
    });
}

async function main() {
    runLayout(false); // Initial
    runLayout(true);  // Expanded
}

main();
