import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { LikeC4Canvas } from "../../../../../apps/designer/src/components/Canvas/LikeC4Canvas";

// Mock store data matching SrujaModelDump interface
const mockLikec4Model = {
  specification: {
    architecture: {
      elements: {},
      relationships: [],
    },
  },
  _metadata: {
    name: "Sample Architecture",
    version: "1.0.0",
    description: "A sample architecture model",
  },
  elements: {
    user: { id: "user", title: "External User", kind: "person", technology: "" },
    "web-app": {
      id: "web-app",
      title: "Web Application",
      kind: "system",
      technology: "React",
      description: "Main web application for users",
    },
    api: {
      id: "api",
      title: "API Server",
      kind: "system",
      technology: "Node.js",
      description: "RESTful API server",
    },
    database: {
      id: "database",
      title: "Database",
      kind: "system",
      technology: "PostgreSQL",
      description: "Main data storage",
    },
  },
  relations: [
    { id: "rel-1", from: "user", to: "web-app", technology: "HTTPS" },
    { id: "rel-2", from: "web-app", to: "api", technology: "REST" },
    { id: "rel-3", from: "api", to: "database", technology: "SQL" },
  ],
};

const meta = {
  title: "Designer/ArchitectureCanvas",
  component: LikeC4Canvas,
  parameters: {
    layout: "fullscreen",
  },
  tags: ["autodocs"],
  argTypes: {
    model: {
      description: "Architecture model to render",
    },
    dragEnabled: {
      control: "boolean",
      description: "Whether dragging nodes is enabled",
    },
    selectedNodeId: {
      control: "text",
      description: "ID of currently selected node",
    },
    hoveredNodeId: {
      control: "text",
      description: "ID of currently hovered node",
    },
  },
  args: {
    model: mockLikec4Model,
    dragEnabled: false,
    selectedNodeId: null,
    hoveredNodeId: null,
    onNodeSelect: fn(),
    onNodeHover: fn(),
    onNodeMove: fn(),
    onNodeAdd: fn(),
    onNodeDelete: fn(),
    onNodeEdit: fn(),
    onEdgeAdd: fn(),
    onEdgeDelete: fn(),
    onZoomChange: fn(),
    onFitView: fn(),
  },
  decorators: [
    (Story) => {
      // Mock stores
      const { vi } = require("vitest");

      vi.doMock("../../../../../apps/designer/src/stores", () => ({
        useArchitectureStore: () => ({
          likec4Model: mockLikec4Model,
          updateArchitecture: fn(),
        }),
        useViewStore: () => ({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
        }),
        useSelectionStore: () => ({
          selectedNodeId: null,
          setSelectedNodeId: fn(),
          hoveredNodeId: null,
          setHoveredNodeId: fn(),
        }),
        useUIStore: () => ({
          activeTab: "diagram",
          setActiveTab: fn(),
        }),
      }));

      vi.doMock("../../../../../apps/designer/src/stores/featureFlagsStore", () => ({
        useFeatureFlagsStore: () => ({
          editMode: "view",
          setEditMode: fn(),
          isEditMode: () => false,
        }),
      }));

      return (
        <div
          style={{
            width: "100%",
            height: "600px",
            position: "relative",
            overflow: "hidden",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
} satisfies Meta<typeof LikeC4Canvas>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    model: mockLikec4Model,
    dragEnabled: false,
  },
};

export const WithDragging: Story = {
  args: {
    model: mockLikec4Model,
    dragEnabled: true,
  },
  decorators: [
    (Story) => {
      // Mock with edit mode
      const { vi } = require("vitest");

      vi.doMock("../../../../../apps/designer/src/stores/featureFlagsStore", () => ({
        useFeatureFlagsStore: () => ({
          editMode: "edit",
          setEditMode: fn(),
          isEditMode: () => true,
        }),
      }));

      return (
        <div
          style={{
            width: "100%",
            height: "600px",
            position: "relative",
            overflow: "hidden",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const WithSelection: Story = {
  args: {
    model: mockLikec4Model,
    dragEnabled: false,
    selectedNodeId: "web-app",
  },
  decorators: [
    (Story) => {
      // Mock with selected node
      const { vi } = require("vitest");

      vi.doMock("../../../../../apps/designer/src/stores", () => ({
        useArchitectureStore: () => ({
          likec4Model: mockLikec4Model,
          updateArchitecture: fn(),
        }),
        useViewStore: () => ({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
        }),
        useSelectionStore: () => ({
          selectedNodeId: "web-app",
          setSelectedNodeId: fn(),
          hoveredNodeId: null,
          setHoveredNodeId: fn(),
        }),
        useUIStore: () => ({
          activeTab: "diagram",
          setActiveTab: fn(),
        }),
      }));

      vi.doMock("../../../../../apps/designer/src/stores/featureFlagsStore", () => ({
        useFeatureFlagsStore: () => ({
          editMode: "view",
          setEditMode: fn(),
          isEditMode: () => false,
        }),
      }));

      return (
        <div
          style={{
            width: "100%",
            height: "600px",
            position: "relative",
            overflow: "hidden",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const ComplexArchitecture: Story = {
  args: {
    model: {
      specification: {
        architecture: {
          elements: {},
          relationships: [],
        },
      },
      _metadata: {
        name: "Complex Microservices Architecture",
        version: "2.0.0",
        description: "A complex microservices architecture",
      },
      elements: {
        user: { id: "user", title: "External User", kind: "person", technology: "" },
        frontend: {
          id: "frontend",
          title: "Frontend",
          kind: "system",
          technology: "React + TypeScript",
          description: "User interface layer",
        },
        "api-gateway": {
          id: "api-gateway",
          title: "API Gateway",
          kind: "system",
          technology: "Kong",
          description: "API routing and security",
        },
        "auth-service": {
          id: "auth-service",
          title: "Auth Service",
          kind: "system",
          technology: "Node.js",
          description: "Authentication and authorization",
        },
        "user-service": {
          id: "user-service",
          title: "User Service",
          kind: "system",
          technology: "Java Spring",
          description: "User management",
        },
        "payment-service": {
          id: "payment-service",
          title: "Payment Service",
          kind: "system",
          technology: "Python Django",
          description: "Payment processing",
        },
        "notification-service": {
          id: "notification-service",
          title: "Notification Service",
          kind: "system",
          technology: "Go",
          description: "Email and push notifications",
        },
        "user-db": {
          id: "user-db",
          title: "User Database",
          kind: "system",
          technology: "PostgreSQL",
          description: "User data storage",
        },
        "payment-db": {
          id: "payment-db",
          title: "Payment Database",
          kind: "system",
          technology: "MongoDB",
          description: "Payment transaction storage",
        },
      },
      relations: [
        { id: "rel-1", from: "user", to: "frontend", technology: "HTTPS" },
        { id: "rel-2", from: "frontend", to: "api-gateway", technology: "REST" },
        { id: "rel-3", from: "api-gateway", to: "auth-service", technology: "REST" },
        { id: "rel-4", from: "api-gateway", to: "user-service", technology: "REST" },
        { id: "rel-5", from: "api-gateway", to: "payment-service", technology: "REST" },
        { id: "rel-6", from: "api-gateway", to: "notification-service", technology: "gRPC" },
        { id: "rel-7", from: "auth-service", to: "user-db", technology: "JDBC" },
        { id: "rel-8", from: "user-service", to: "user-db", technology: "JDBC" },
        { id: "rel-9", from: "payment-service", to: "payment-db", technology: "Mongo Driver" },
        { id: "rel-10", from: "notification-service", to: "user-db", technology: "JDBC" },
      ],
    },
    dragEnabled: false,
  },
};

export const Empty: Story = {
  args: {
    model: {
      specification: {
        architecture: {
          elements: {},
          relationships: [],
        },
      },
      _metadata: {
        name: "Empty Architecture",
        version: "1.0.0",
        description: "An empty architecture model",
      },
      elements: {},
      relations: [],
    },
    dragEnabled: false,
  },
};

export const Mobile: Story = {
  args: {
    model: mockLikec4Model,
    dragEnabled: false,
  },
  decorators: [
    (Story) => {
      return (
        <div
          style={{
            width: "375px",
            height: "600px",
            position: "relative",
            overflow: "hidden",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};
