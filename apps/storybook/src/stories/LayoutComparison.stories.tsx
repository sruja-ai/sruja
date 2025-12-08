// apps/storybook/src/stories/LayoutComparison.stories.tsx
// Comparison stories for ELK vs Sruja layout engines
import React, { useEffect, useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { ReactFlowProvider, ReactFlow, useNodesState, useEdgesState, Background, Controls, MiniMap } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { jsonToReactFlow, applyElkLayout, applySrujaLayout, nodeTypes } from '@sruja/react-flow-architecture';
import type { ArchitectureJSON, C4Level } from '@sruja/react-flow-architecture';

const meta: Meta = {
    title: 'Layout Engines/Comparison',
    parameters: {
        layout: 'padded',
        docs: {
            description: {
                component: `
# Layout Engine Comparison

Compare **ELK** (Eclipse Layout Kernel) and **Sruja Layout** engines side-by-side.

## Features

- **ELK Layout**: Industry-standard hierarchical layout algorithm
- **Sruja Layout**: Custom C4-optimized layout with better edge routing and node positioning

## Use Cases

- Compare layout quality and aesthetics
- Evaluate performance differences
- Test different architecture complexities
        `,
            },
        },
    },
};

export default meta;
type Story = StoryObj;

// Sample architecture data
const SIMPLE_ARCH: ArchitectureJSON = {
    metadata: { name: 'Simple System', version: '1.0.0', generated: new Date().toISOString() },
    architecture: {
        systems: [
            {
                id: 'WebApp',
                label: 'Web Application',
                description: 'Main web application',
                containers: [
                    { id: 'Frontend', label: 'Frontend', technology: 'React' },
                    { id: 'Backend', label: 'Backend API', technology: 'Node.js' },
                ],
            },
            {
                id: 'Database',
                label: 'Database',
                description: 'PostgreSQL database',
            },
        ],
        persons: [
            { id: 'User', label: 'User', description: 'End user' },
        ],
        relations: [
            { from: 'User', to: 'WebApp', label: 'Uses' },
            { from: 'WebApp', to: 'Database', label: 'Reads/Writes' },
        ],
    },
    navigation: { levels: ['L1', 'L2', 'L3'] },
};

const COMPLEX_ARCH: ArchitectureJSON = {
    metadata: { name: 'E-Commerce Platform', version: '1.0.0', generated: new Date().toISOString() },
    architecture: {
        systems: [
            {
                id: 'EcommercePlatform',
                label: 'E-Commerce Platform',
                description: 'Main e-commerce system',
                containers: [
                    { id: 'WebStore', label: 'Web Store', technology: 'Next.js' },
                    { id: 'API', label: 'API Gateway', technology: 'Express.js' },
                    { id: 'OrderService', label: 'Order Service', technology: 'Node.js' },
                    { id: 'PaymentService', label: 'Payment Service', technology: 'Go' },
                    { id: 'InventoryService', label: 'Inventory Service', technology: 'Python' },
                ],
                datastores: [
                    { id: 'UserDB', label: 'User Database' },
                    { id: 'ProductDB', label: 'Product Database' },
                    { id: 'OrderDB', label: 'Order Database' },
                ],
            },
            {
                id: 'PaymentGateway',
                label: 'Payment Gateway',
                description: 'External payment provider',
            },
        ],
        persons: [
            { id: 'Customer', label: 'Customer' },
            { id: 'Admin', label: 'Store Admin' },
        ],
        relations: [
            { from: 'Customer', to: 'EcommercePlatform.WebStore', label: 'Browses' },
            { from: 'Admin', to: 'EcommercePlatform.API', label: 'Manages' },
            { from: 'EcommercePlatform.WebStore', to: 'EcommercePlatform.API', label: 'API calls' },
            { from: 'EcommercePlatform.API', to: 'EcommercePlatform.OrderService', label: 'Routes' },
            { from: 'EcommercePlatform.API', to: 'EcommercePlatform.PaymentService', label: 'Routes' },
            { from: 'EcommercePlatform.API', to: 'EcommercePlatform.InventoryService', label: 'Queries' },
            { from: 'EcommercePlatform.PaymentService', to: 'PaymentGateway', label: 'Uses' },
            { from: 'EcommercePlatform.OrderService', to: 'EcommercePlatform.OrderDB', label: 'Stores' },
        ],
    },
    navigation: { levels: ['L1', 'L2', 'L3'] },
};

interface LayoutCanvasProps {
    architecture: ArchitectureJSON;
    level: C4Level;
    layoutType: 'elk' | 'sruja';
    direction?: 'TB' | 'LR' | 'RL';
}

function LayoutCanvas({ architecture, level, layoutType, direction = 'TB' }: LayoutCanvasProps) {
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        async function applyLayout() {
            try {
                setLoading(true);
                setError(null);

                // Convert architecture to React Flow format
                const { nodes: initialNodes, edges: initialEdges } = jsonToReactFlow(architecture, {
                    level,
                    focusedSystemId: level === 'L2' ? architecture.architecture.systems?.[0]?.id : undefined,
                    focusedContainerId: level === 'L3' ? architecture.architecture.systems?.[0]?.containers?.[0]?.id : undefined,
                });

                let result;
                if (layoutType === 'elk') {
                    result = await applyElkLayout(initialNodes, initialEdges, { direction });
                } else {
                    result = applySrujaLayout(initialNodes, initialEdges, architecture, {
                        level,
                        focusedSystemId: level === 'L2' ? architecture.architecture.systems?.[0]?.id : undefined,
                        focusedContainerId: level === 'L3' ? architecture.architecture.systems?.[0]?.containers?.[0]?.id : undefined,
                        direction,
                    });
                }

                setNodes(result.nodes);
                setEdges(result.edges);
            } catch (err) {
                setError(err instanceof Error ? err.message : 'Layout failed');
                console.error('Layout error:', err);
            } finally {
                setLoading(false);
            }
        }

        applyLayout();
    }, [architecture, level, layoutType, direction]);

    if (loading) {
        return (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#666' }}>
                Applying {layoutType.toUpperCase()} layout...
            </div>
        );
    }

    if (error) {
        return (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#d32f2f' }}>
                Error: {error}
            </div>
        );
    }

    return (
        <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            nodeTypes={nodeTypes}
            fitView
            fitViewOptions={{ padding: 0.2 }}
        >
            <Background />
            <Controls />
            <MiniMap />
        </ReactFlow>
    );
}

interface ComparisonViewProps {
    architecture: ArchitectureJSON;
    level?: C4Level;
    direction?: 'TB' | 'LR' | 'RL';
}

function ComparisonView({ architecture, level = 'L1', direction = 'TB' }: ComparisonViewProps) {
    return (
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, height: '600px' }}>
            <div style={{ border: '1px solid #e2e8f0', borderRadius: 8, overflow: 'hidden', position: 'relative' }}>
                <div style={{ position: 'absolute', top: 8, left: 8, zIndex: 10, background: 'white', padding: '4px 8px', borderRadius: 4, fontSize: 12, fontWeight: 'bold' }}>
                    ELK Layout
                </div>
                <ReactFlowProvider>
                    <LayoutCanvas architecture={architecture} level={level} layoutType="elk" direction={direction} />
                </ReactFlowProvider>
            </div>
            <div style={{ border: '1px solid #e2e8f0', borderRadius: 8, overflow: 'hidden', position: 'relative' }}>
                <div style={{ position: 'absolute', top: 8, left: 8, zIndex: 10, background: 'white', padding: '4px 8px', borderRadius: 4, fontSize: 12, fontWeight: 'bold' }}>
                    Sruja Layout
                </div>
                <ReactFlowProvider>
                    <LayoutCanvas architecture={architecture} level={level} layoutType="sruja" direction={direction} />
                </ReactFlowProvider>
            </div>
        </div>
    );
}

export const SimpleArchitecture: Story = {
    render: () => <ComparisonView architecture={SIMPLE_ARCH} level="L1" />,
    parameters: {
        docs: {
            description: {
                story: 'Side-by-side comparison of ELK and Sruja layouts for a simple architecture with a user, web app, and database.',
            },
        },
    },
};

export const ComplexArchitecture: Story = {
    render: () => <ComparisonView architecture={COMPLEX_ARCH} level="L1" />,
    parameters: {
        docs: {
            description: {
                story: 'Comparison for a complex e-commerce platform with multiple systems, containers, and relationships.',
            },
        },
    },
};

export const ContainerLevel: Story = {
    render: () => <ComparisonView architecture={COMPLEX_ARCH} level="L2" />,
    parameters: {
        docs: {
            description: {
                story: 'Comparison at the Container (L2) level, showing containers within the e-commerce platform system.',
            },
        },
    },
};

export const LeftToRight: Story = {
    render: () => <ComparisonView architecture={COMPLEX_ARCH} level="L1" direction="LR" />,
    parameters: {
        docs: {
            description: {
                story: 'Comparison with left-to-right layout direction. Both engines support different directions.',
            },
        },
    },
};

export const RightToLeft: Story = {
    render: () => <ComparisonView architecture={COMPLEX_ARCH} level="L1" direction="RL" />,
    parameters: {
        docs: {
            description: {
                story: 'Comparison with right-to-left layout direction.',
            },
        },
    },
};

// Individual layout stories
export const ELKOnly: Story = {
    render: () => (
        <div style={{ height: '600px', border: '1px solid #e2e8f0', borderRadius: 8, overflow: 'hidden' }}>
            <ReactFlowProvider>
                <LayoutCanvas architecture={COMPLEX_ARCH} level="L1" layoutType="elk" />
            </ReactFlowProvider>
        </div>
    ),
    parameters: {
        docs: {
            description: {
                story: 'ELK layout engine only. Uses the Eclipse Layout Kernel for hierarchical graph layout.',
            },
        },
    },
};

export const SrujaOnly: Story = {
    render: () => (
        <div style={{ height: '600px', border: '1px solid #e2e8f0', borderRadius: 8, overflow: 'hidden' }}>
            <ReactFlowProvider>
                <LayoutCanvas architecture={COMPLEX_ARCH} level="L1" layoutType="sruja" />
            </ReactFlowProvider>
        </div>
    ),
    parameters: {
        docs: {
            description: {
                story: 'Sruja layout engine only. Custom C4-optimized layout with improved edge routing and node positioning.',
            },
        },
    },
};

