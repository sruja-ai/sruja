// Architecture Visualizer Storybook Stories
// Demonstrates the ReactFlow-based C4 architecture viewer

import React, { useEffect } from 'react'
import type { Meta, StoryObj } from '@storybook/react'
import { ReactFlowProvider } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

// Import components from playground
import { ArchitectureCanvas } from '../../../playground/src/components/Canvas/ArchitectureCanvas'
import { NavigationPanel } from '../../../playground/src/components/Panels/NavigationPanel'
import { DetailsPanel } from '../../../playground/src/components/Panels/DetailsPanel'
import { FlowController } from '../../../playground/src/components/Flow/FlowController'
import { useArchitectureStore, useViewStore, useSelectionStore } from '../../../playground/src/stores'
import type { ArchitectureJSON } from '../../../playground/src/types'

// Import necessary CSS
import '../../../playground/src/components/Canvas/ArchitectureCanvas.css'
import '../../../playground/src/components/Panels/NavigationPanel.css'
import '../../../playground/src/components/Panels/DetailsPanel.css'
import '../../../playground/src/components/Flow/FlowController.css'
import '../../../playground/src/components/Nodes/nodes.css'

// Sample architecture data
const SIMPLE_ARCHITECTURE: ArchitectureJSON = {
    metadata: { name: 'Simple Web App', version: '1.0.0', generated: new Date().toISOString() },
    architecture: {
        systems: [
            {
                id: 'WebApp',
                label: 'Web Application',
                description: 'User-facing SPA',
                containers: [
                    { id: 'Frontend', label: 'Frontend SPA', technology: 'React', description: 'React-based UI' },
                    { id: 'Backend', label: 'API Server', technology: 'Node.js', description: 'REST API' },
                ],
            },
            {
                id: 'Database',
                label: 'Database',
                description: 'PostgreSQL database',
            },
        ],
        persons: [
            { id: 'User', label: 'End User', description: 'Customer using the web app' },
            { id: 'Admin', label: 'Administrator', description: 'System administrator' },
        ],
        relations: [
            { from: 'User', to: 'WebApp', label: 'Uses', verb: 'Uses' },
            { from: 'Admin', to: 'WebApp', label: 'Administers', verb: 'Administers' },
            { from: 'WebApp', to: 'Database', label: 'Reads/Writes', verb: 'Reads/Writes' },
        ],
    },
    navigation: { levels: ['L1', 'L2', 'L3'] },
}

const ECOMMERCE_ARCHITECTURE: ArchitectureJSON = {
    metadata: { name: 'E-Commerce Platform', version: '1.0.0', generated: new Date().toISOString() },
    architecture: {
        systems: [
            {
                id: 'EcommercePlatform',
                label: 'E-Commerce Platform',
                description: 'Main e-commerce system',
                containers: [
                    { id: 'WebStore', label: 'Web Store', technology: 'Next.js', description: 'Customer-facing storefront' },
                    { id: 'API', label: 'API Gateway', technology: 'Express.js', description: 'Central API' },
                    { id: 'OrderService', label: 'Order Service', technology: 'Node.js', description: 'Order processing' },
                    { id: 'PaymentService', label: 'Payment Service', technology: 'Go', description: 'Payment processing' },
                    { id: 'InventoryService', label: 'Inventory Service', technology: 'Python', description: 'Stock management' },
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
                description: 'External payment provider (Stripe)',
            },
            {
                id: 'ShippingProvider',
                label: 'Shipping Provider',
                description: 'External shipping service',
            },
        ],
        persons: [
            { id: 'Customer', label: 'Customer', description: 'Online shopper' },
            { id: 'Admin', label: 'Store Admin', description: 'Store administrator' },
            { id: 'Supplier', label: 'Supplier', description: 'Product supplier' },
        ],
        relations: [
            { from: 'Customer', to: 'EcommercePlatform.WebStore', label: 'Browses & Purchases', verb: 'Uses' },
            { from: 'Admin', to: 'EcommercePlatform.API', label: 'Manages store', verb: 'Manages' },
            { from: 'EcommercePlatform.WebStore', to: 'EcommercePlatform.API', label: 'API calls', verb: 'Calls' },
            { from: 'EcommercePlatform.API', to: 'EcommercePlatform.OrderService', label: 'Orders', verb: 'Routes' },
            { from: 'EcommercePlatform.API', to: 'EcommercePlatform.PaymentService', label: 'Payments', verb: 'Routes' },
            { from: 'EcommercePlatform.API', to: 'EcommercePlatform.InventoryService', label: 'Inventory', verb: 'Queries' },
            { from: 'EcommercePlatform.PaymentService', to: 'PaymentGateway', label: 'Processes payment', verb: 'Uses' },
            { from: 'EcommercePlatform.OrderService', to: 'ShippingProvider', label: 'Ships order', verb: 'Uses' },
        ],
        flows: [
            {
                id: 'checkout-flow',
                title: 'Checkout Flow',
                description: 'Customer completes a purchase',
                steps: [
                    { from: 'Customer', to: 'EcommercePlatform.WebStore', description: 'Customer adds items to cart' },
                    { from: 'EcommercePlatform.WebStore', to: 'EcommercePlatform.API', description: 'Submit order' },
                    { from: 'EcommercePlatform.API', to: 'EcommercePlatform.InventoryService', description: 'Check stock availability' },
                    { from: 'EcommercePlatform.API', to: 'EcommercePlatform.PaymentService', description: 'Process payment' },
                    { from: 'EcommercePlatform.PaymentService', to: 'PaymentGateway', description: 'Charge credit card' },
                    { from: 'EcommercePlatform.API', to: 'EcommercePlatform.OrderService', description: 'Create order' },
                    { from: 'EcommercePlatform.OrderService', to: 'ShippingProvider', description: 'Schedule delivery' },
                ],
            },
        ],
    },
    navigation: { levels: ['L1', 'L2', 'L3'] },
}

const MICROSERVICES_ARCHITECTURE: ArchitectureJSON = {
    metadata: { name: 'Microservices Demo', version: '1.0.0', generated: new Date().toISOString() },
    architecture: {
        systems: [
            {
                id: 'Gateway',
                label: 'API Gateway',
                description: 'Kong API Gateway',
                containers: [
                    { id: 'Auth', label: 'Auth Service', technology: 'Keycloak' },
                    { id: 'Routing', label: 'Routing', technology: 'Kong' },
                ],
            },
            {
                id: 'UserService',
                label: 'User Service',
                description: 'User management microservice',
                containers: [
                    { id: 'UserAPI', label: 'User API', technology: 'Go' },
                    { id: 'UserDB', label: 'User DB', technology: 'PostgreSQL' },
                ],
            },
            {
                id: 'ProductService',
                label: 'Product Service',
                description: 'Product catalog microservice',
                containers: [
                    { id: 'ProductAPI', label: 'Product API', technology: 'Python' },
                    { id: 'ProductDB', label: 'Product DB', technology: 'MongoDB' },
                    { id: 'SearchIndex', label: 'Search Index', technology: 'Elasticsearch' },
                ],
            },
            {
                id: 'NotificationService',
                label: 'Notification Service',
                description: 'Email and push notifications',
                containers: [
                    { id: 'NotifyAPI', label: 'Notification API', technology: 'Node.js' },
                    { id: 'Queue', label: 'Message Queue', technology: 'RabbitMQ' },
                ],
            },
        ],
        persons: [
            { id: 'Developer', label: 'Developer', description: 'API Consumer' },
            { id: 'EndUser', label: 'Mobile User', description: 'Mobile app user' },
        ],
        relations: [
            { from: 'EndUser', to: 'Gateway', label: 'Mobile requests', verb: 'Uses' },
            { from: 'Developer', to: 'Gateway', label: 'API integration', verb: 'Uses' },
            { from: 'Gateway', to: 'UserService', label: 'User ops', verb: 'Routes' },
            { from: 'Gateway', to: 'ProductService', label: 'Product ops', verb: 'Routes' },
            { from: 'UserService', to: 'NotificationService', label: 'Send notifications', verb: 'Publishes' },
            { from: 'ProductService', to: 'NotificationService', label: 'Product alerts', verb: 'Publishes' },
        ],
    },
    navigation: { levels: ['L1', 'L2', 'L3'] },
}

// Wrapper component to initialize store and provide ReactFlow context
interface StoryWrapperProps {
    data: ArchitectureJSON
    showNavigation?: boolean
    showDetails?: boolean
    showFlow?: boolean
    height?: string
    selectNodeId?: string
}

function StoryWrapper({
    data,
    showNavigation = false,
    showDetails = false,
    showFlow = false,
    height = '600px',
    selectNodeId,
}: StoryWrapperProps) {
    const loadFromJSON = useArchitectureStore((s) => s.loadFromJSON)
    const selectNode = useSelectionStore((s) => s.selectNode)
    const goToRoot = useViewStore((s) => s.goToRoot)

    useEffect(() => {
        goToRoot()
        loadFromJSON(data)
        if (selectNodeId) {
            // Delay selection to allow layout to complete
            setTimeout(() => selectNode(selectNodeId), 500)
        }
        return () => {
            // Cleanup on unmount
            useArchitectureStore.getState().reset()
            useViewStore.getState().goToRoot()
            useSelectionStore.getState().selectNode(null)
        }
    }, [data, loadFromJSON, goToRoot, selectNode, selectNodeId])

    return (
        <ReactFlowProvider>
            <div style={{
                display: 'flex',
                height,
                border: '1px solid var(--color-border, #e2e8f0)',
                borderRadius: 8,
                overflow: 'hidden',
                backgroundColor: 'var(--color-surface, #fff)',
            }}>
                {showNavigation && (
                    <div style={{ width: 260, borderRight: '1px solid var(--color-border, #e2e8f0)', overflow: 'auto' }}>
                        <NavigationPanel />
                    </div>
                )}
                <div style={{ flex: 1, position: 'relative' }}>
                    <ArchitectureCanvas />
                    {showFlow && (
                        <div style={{ position: 'absolute', bottom: 16, left: '50%', transform: 'translateX(-50%)' }}>
                            <FlowController />
                        </div>
                    )}
                </div>
                {showDetails && (
                    <div style={{ width: 300, borderLeft: '1px solid var(--color-border, #e2e8f0)' }}>
                        <DetailsPanel />
                    </div>
                )}
            </div>
        </ReactFlowProvider>
    )
}

const meta: Meta = {
    title: 'Components/Architecture Visualizer',
    parameters: {
        layout: 'fullscreen',
        docs: {
            description: {
                component: `
# Architecture Visualizer

A **ReactFlow-based** interactive C4 architecture viewer with support for:
- **Multi-level navigation** (L1 System Context → L2 Container → L3 Component)
- **Custom C4 node types** (System, Container, Component, Person, DataStore, Queue)
- **ELK auto-layout** for automatic node positioning
- **Flow/Scenario playback** with animated edge highlighting
- **Interactive selection** with detail panels

## Viewport Testing

Use the viewport toolbar to test responsive behavior. Architecture diagrams are best viewed on desktop or larger screens.
        `,
            },
        },
    },
    // Default viewport for all stories in this file (can be overridden per-story)
    globals: {
        viewport: { value: 'desktop', isRotated: false },
    },
}

export default meta
type Story = StoryObj

// ========================================
// Main Canvas Stories
// ========================================

export const SimpleArchitecture: Story = {
    render: () => <StoryWrapper data={SIMPLE_ARCHITECTURE} height="500px" />,
    parameters: {
        docs: {
            description: {
                story: 'A simple web application architecture with a frontend, backend, and database. Shows basic C4 visualization at the System Context (L1) level.',
            },
        },
    },
}

export const EcommercePlatform: Story = {
    render: () => <StoryWrapper data={ECOMMERCE_ARCHITECTURE} height="600px" />,
    parameters: {
        docs: {
            description: {
                story: 'A complex e-commerce platform with multiple microservices, external integrations (payment gateway, shipping), and a checkout flow. Double-click on a system to drill down to the Container (L2) level.',
            },
        },
    },
}

export const MicroservicesDemo: Story = {
    render: () => <StoryWrapper data={MICROSERVICES_ARCHITECTURE} height="600px" />,
    parameters: {
        docs: {
            description: {
                story: 'A microservices architecture with an API gateway, multiple services (User, Product, Notification), and inter-service communication.',
            },
        },
    },
}

// ========================================
// With Navigation Panel
// ========================================

export const WithNavigationPanel: Story = {
    render: () => (
        <StoryWrapper
            data={ECOMMERCE_ARCHITECTURE}
            showNavigation={true}
            height="650px"
        />
    ),
    parameters: {
        docs: {
            description: {
                story: 'Architecture canvas with the navigation sidebar. The sidebar shows view levels (L1/L2/L3), systems tree, actors, and available flows. Click on systems in the tree to drill down.',
            },
        },
    },
}

// ========================================
// With Details Panel
// ========================================

export const WithDetailsPanel: Story = {
    render: () => (
        <StoryWrapper
            data={ECOMMERCE_ARCHITECTURE}
            showDetails={true}
            selectNodeId="EcommercePlatform"
            height="600px"
        />
    ),
    parameters: {
        docs: {
            description: {
                story: 'Architecture canvas with the details panel shown on the right. Click any node to see its details including type, ID, description, technology, and child counts.',
            },
        },
    },
}

// ========================================
// With Flow Controller
// ========================================

export const WithFlowController: Story = {
    render: () => (
        <StoryWrapper
            data={ECOMMERCE_ARCHITECTURE}
            showFlow={true}
            height="650px"
        />
    ),
    parameters: {
        docs: {
            description: {
                story: 'Architecture canvas with the flow controller. The e-commerce architecture includes a "Checkout Flow" that can be played step-by-step with animated edge highlighting. Click on the flow to start playback.',
            },
        },
    },
}

// ========================================
// Full Application Layout
// ========================================

export const FullLayout: Story = {
    render: () => (
        <StoryWrapper
            data={ECOMMERCE_ARCHITECTURE}
            showNavigation={true}
            showDetails={true}
            showFlow={true}
            height="700px"
        />
    ),
    // Force this story to always render in Large Monitor viewport
    globals: {
        viewport: { value: 'largeMonitor', isRotated: false },
    },
    parameters: {
        docs: {
            description: {
                story: 'Complete application layout with navigation panel, canvas, details panel, and flow controller. This shows all components working together as they would in the full Architecture Visualizer app. **Locked to Large Monitor (1920x1080) viewport.**',
            },
        },
    },
}

// ========================================
// Empty State
// ========================================

export const EmptyState: Story = {
    render: () => {
        const EmptyCanvas = () => {
            useEffect(() => {
                useArchitectureStore.getState().reset()
            }, [])
            return (
                <ReactFlowProvider>
                    <div style={{
                        height: '400px',
                        border: '1px solid var(--color-border, #e2e8f0)',
                        borderRadius: 8,
                        overflow: 'hidden'
                    }}>
                        <ArchitectureCanvas />
                    </div>
                </ReactFlowProvider>
            )
        }
        return <EmptyCanvas />
    },
    parameters: {
        docs: {
            description: {
                story: 'The canvas in its empty state, showing the placeholder message prompting users to load architecture data.',
            },
        },
    },
}

// ========================================
// Navigation Panel Standalone
// ========================================

export const NavigationPanelStandalone: Story = {
    render: () => {
        const Wrapper = () => {
            const loadFromJSON = useArchitectureStore((s) => s.loadFromJSON)
            useEffect(() => {
                loadFromJSON(ECOMMERCE_ARCHITECTURE)
            }, [loadFromJSON])
            return (
                <div style={{
                    width: 280,
                    height: '500px',
                    border: '1px solid var(--color-border, #e2e8f0)',
                    borderRadius: 8,
                    overflow: 'hidden',
                    backgroundColor: 'var(--color-surface, #fff)',
                }}>
                    <NavigationPanel />
                </div>
            )
        }
        return <Wrapper />
    },
    parameters: {
        docs: {
            description: {
                story: 'The navigation panel component shown standalone. Displays view level selector, systems tree with expandable items, actors list, and available flows.',
            },
        },
    },
}

// ========================================
// Details Panel Standalone
// ========================================

export const DetailsPanelStandalone: Story = {
    render: () => {
        const Wrapper = () => {
            const loadFromJSON = useArchitectureStore((s) => s.loadFromJSON)
            const selectNode = useSelectionStore((s) => s.selectNode)
            useEffect(() => {
                loadFromJSON(ECOMMERCE_ARCHITECTURE)
                setTimeout(() => selectNode('EcommercePlatform'), 100)
            }, [loadFromJSON, selectNode])
            return (
                <div style={{
                    width: 320,
                    border: '1px solid var(--color-border, #e2e8f0)',
                    borderRadius: 8,
                    overflow: 'hidden',
                    backgroundColor: 'var(--color-surface, #fff)',
                }}>
                    <DetailsPanel />
                </div>
            )
        }
        return <Wrapper />
    },
    parameters: {
        docs: {
            description: {
                story: 'The details panel component shown standalone. Displays selected node information including type badge, ID, description, technology, and child counts.',
            },
        },
    },
}

// ========================================
// Flow Controller Standalone  
// ========================================

export const FlowControllerStandalone: Story = {
    render: () => {
        const Wrapper = () => {
            const loadFromJSON = useArchitectureStore((s) => s.loadFromJSON)
            useEffect(() => {
                loadFromJSON(ECOMMERCE_ARCHITECTURE)
            }, [loadFromJSON])
            return (
                <div style={{
                    padding: 16,
                    backgroundColor: 'var(--color-surface, #fff)',
                    border: '1px solid var(--color-border, #e2e8f0)',
                    borderRadius: 8,
                }}>
                    <FlowController />
                </div>
            )
        }
        return <Wrapper />
    },
    parameters: {
        docs: {
            description: {
                story: 'The flow controller component shown standalone. Select a flow to see the playback controls with play/pause, step forward/back, and current step information.',
            },
        },
    },
}

// ========================================
// Dark Mode Support
// ========================================

export const DarkModePreview: Story = {
    render: () => (
        <div style={{
            padding: 24,
            backgroundColor: '#1a1a2e',
            minHeight: '100vh',
        }}>
            <style>{`
        :root {
          --color-surface: #16213e;
          --color-border: #374151;
          --color-text-primary: #e5e7eb;
          --color-text-secondary: #9ca3af;
          --c4-person: #08a045;
          --c4-system: #438dd5;
          --c4-container: #438dd5;
          --c4-component: #85bbf0;
        }
      `}</style>
            <StoryWrapper
                data={ECOMMERCE_ARCHITECTURE}
                showNavigation={true}
                showFlow={true}
                height="600px"
            />
        </div>
    ),
    parameters: {
        docs: {
            description: {
                story: 'Preview of the architecture visualizer in dark mode. The component respects CSS custom properties for theming.',
            },
        },
        backgrounds: { default: 'Dark Surface' },
    },
}

// ========================================
// Responsive View Stories (Locked Viewports)
// ========================================

export const TabletView: Story = {
    render: () => (
        <StoryWrapper
            data={SIMPLE_ARCHITECTURE}
            showNavigation={true}
            height="100%"
        />
    ),
    // Force tablet landscape viewport
    globals: {
        viewport: { value: 'tabletLandscape', isRotated: false },
    },
    parameters: {
        docs: {
            description: {
                story: 'Architecture canvas on a tablet-sized viewport (1024x768). Navigation panel is shown but may need scrolling. **Viewport is locked and cannot be changed.**',
            },
        },
    },
}

export const MobileView: Story = {
    render: () => (
        <StoryWrapper
            data={SIMPLE_ARCHITECTURE}
            height="100%"
        />
    ),
    // Force mobile viewport
    globals: {
        viewport: { value: 'mobile1', isRotated: false },
    },
    parameters: {
        docs: {
            description: {
                story: 'Architecture canvas on a mobile viewport (320x568). Demonstrates how the diagram scales on small screens - note that architecture diagrams are not optimized for mobile viewing. **Viewport is locked and cannot be changed.**',
            },
        },
    },
}

export const PresentationView: Story = {
    render: () => (
        <StoryWrapper
            data={ECOMMERCE_ARCHITECTURE}
            height="100%"
        />
    ),
    // Force presentation viewport (16:9)
    globals: {
        viewport: { value: 'presentation', isRotated: false },
    },
    parameters: {
        docs: {
            description: {
                story: 'Architecture canvas optimized for presentations (1600x900, 16:9 aspect ratio). Ideal for screen sharing or embedding in slides. **Viewport is locked and cannot be changed.**',
            },
        },
    },
}
