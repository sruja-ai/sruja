import { vi } from "vitest";
import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { LikeC4Canvas } from "../../../../../apps/designer/src/components/Canvas/LikeC4Canvas";
import type { SrujaModelDump } from "@sruja/shared";

/**
 * @deprecated Use LikeC4Canvas.stories.tsx instead
 * This file is kept for backward compatibility but LikeC4Canvas.stories.tsx has the updated implementation
 */

// Mock store data matching SrujaModelDump interface
const mockLikec4Model: SrujaModelDump = {
  _stage: "parsed",
  projectId: "storybook-project",
  project: {
    id: "storybook-project",
    name: "Sample Architecture",
  },
  specification: {
    elements: {},
    tags: {},
    relationships: {},
  },
  globals: {
    predicates: {},
    dynamicPredicates: {},
    styles: {},
  },
  elements: {
    user: {
      id: "user",
      title: "External User",
      kind: "person",
      technology: "",
      tags: [],
      links: [],
      style: {},
    },
    "web-app": {
      id: "web-app",
      title: "Web Application",
      kind: "system",
      technology: "React",
      description: "Main web application for users",
      tags: [],
      links: [],
      style: {},
    },
    api: {
      id: "api",
      title: "API Server",
      kind: "system",
      technology: "Node.js",
      description: "RESTful API server",
      tags: [],
      links: [],
      style: {},
    },
    database: {
      id: "database",
      title: "Database",
      kind: "system",
      technology: "PostgreSQL",
      description: "Main data storage",
      tags: [],
      links: [],
      style: {},
    },
  },
  relations: [
    {
      id: "rel-1",
      source: { model: "user" },
      target: { model: "web-app" },
      technology: "HTTPS",
    },
    {
      id: "rel-2",
      source: { model: "web-app" },
      target: { model: "api" },
      technology: "REST",
    },
    {
      id: "rel-3",
      source: { model: "api" },
      target: { model: "database" },
      technology: "SQL",
    },
  ],
  views: {
    L1: {
      id: "L1",
      title: "System Context (L1)",
      rules: [{ include: [{ ref: { model: "user" } }, { ref: { model: "web-app" } }, { ref: { model: "api" } }, { ref: { model: "database" } }] }],
      nodes: [],
      edges: [],
    },
    index: {
      id: "index",
      title: "Index",
      rules: [{ include: [{ wildcard: true }] }],
      nodes: [],
      edges: [],
    },
  },
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
          setLevel: fn(),
        }),
        useSelectionStore: () => ({
          selectedNodeId: null,
          selectNode: fn(),
          activeFlow: null,
          flowStep: 0,
          isFlowPlaying: false,
          playFlow: fn(),
          pauseFlow: fn(),
          nextStep: fn(),
          prevStep: fn(),
          setFlowStep: fn(),
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
          setLevel: fn(),
        }),
        useSelectionStore: () => ({
          selectedNodeId: "web-app",
          selectNode: fn(),
          activeFlow: null,
          flowStep: 0,
          isFlowPlaying: false,
          playFlow: fn(),
          pauseFlow: fn(),
          nextStep: fn(),
          prevStep: fn(),
          setFlowStep: fn(),
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
  args: {},
  decorators: [
    (Story) => {

      const complexModel: SrujaModelDump = {
        _stage: "parsed",
        projectId: "complex-project",
        project: {
          id: "complex-project",
          name: "Complex Microservices Architecture",
        },
        specification: { elements: {}, tags: {}, relationships: {} },
        globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
        elements: {
          user: {
            id: "user",
            title: "External User",
            kind: "person",
            technology: "",
            tags: [],
            links: [],
            style: {},
          },
          frontend: {
            id: "frontend",
            title: "Frontend",
            kind: "system",
            technology: "React + TypeScript",
            description: "User interface layer",
            tags: [],
            links: [],
            style: {},
          },
          "api-gateway": {
            id: "api-gateway",
            title: "API Gateway",
            kind: "system",
            technology: "Kong",
            description: "API routing and security",
            tags: [],
            links: [],
            style: {},
          },
          "auth-service": {
            id: "auth-service",
            title: "Auth Service",
            kind: "system",
            technology: "Node.js",
            description: "Authentication and authorization",
            tags: [],
            links: [],
            style: {},
          },
          "user-service": {
            id: "user-service",
            title: "User Service",
            kind: "system",
            technology: "Java Spring",
            description: "User management",
            tags: [],
            links: [],
            style: {},
          },
          "payment-service": {
            id: "payment-service",
            title: "Payment Service",
            kind: "system",
            technology: "Python Django",
            description: "Payment processing",
            tags: [],
            links: [],
            style: {},
          },
          "notification-service": {
            id: "notification-service",
            title: "Notification Service",
            kind: "system",
            technology: "Go",
            description: "Email and push notifications",
            tags: [],
            links: [],
            style: {},
          },
          "user-db": {
            id: "user-db",
            title: "User Database",
            kind: "database",
            technology: "PostgreSQL",
            description: "User data storage",
            tags: [],
            links: [],
            style: {},
          },
          "payment-db": {
            id: "payment-db",
            title: "Payment Database",
            kind: "database",
            technology: "MongoDB",
            description: "Payment transaction storage",
            tags: [],
            links: [],
            style: {},
          },
        },
        relations: [
          { id: "rel-1", source: { model: "user" }, target: { model: "frontend" }, technology: "HTTPS" },
          { id: "rel-2", source: { model: "frontend" }, target: { model: "api-gateway" }, technology: "REST" },
          { id: "rel-3", source: { model: "api-gateway" }, target: { model: "auth-service" }, technology: "REST" },
          { id: "rel-4", source: { model: "api-gateway" }, target: { model: "user-service" }, technology: "REST" },
          { id: "rel-5", source: { model: "api-gateway" }, target: { model: "payment-service" }, technology: "REST" },
          { id: "rel-6", source: { model: "api-gateway" }, target: { model: "notification-service" }, technology: "gRPC" },
          { id: "rel-7", source: { model: "auth-service" }, target: { model: "user-db" }, technology: "JDBC" },
          { id: "rel-8", source: { model: "user-service" }, target: { model: "user-db" }, technology: "JDBC" },
          { id: "rel-9", source: { model: "payment-service" }, target: { model: "payment-db" }, technology: "Mongo Driver" },
          { id: "rel-10", source: { model: "notification-service" }, target: { model: "user-db" }, technology: "JDBC" },
        ],
        views: {
          L1: {
            id: "L1",
            title: "System Context (L1)",
            rules: [{ include: [{ wildcard: true }] }],
            nodes: [],
            edges: [],
          },
          index: {
            id: "index",
            title: "Index",
            rules: [{ include: [{ wildcard: true }] }],
            nodes: [],
            edges: [],
          },
        },
      };

      vi.doMock("../../../../../apps/designer/src/stores", () => ({
        useArchitectureStore: () => ({
          likec4Model: complexModel,
          updateArchitecture: fn(),
        }),
        useViewStore: () => ({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        }),
        useSelectionStore: () => ({
          selectedNodeId: null,
          selectNode: fn(),
          activeFlow: null,
          flowStep: 0,
          isFlowPlaying: false,
          playFlow: fn(),
          pauseFlow: fn(),
          nextStep: fn(),
          prevStep: fn(),
          setFlowStep: fn(),
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
            border: "1px solid #e5e7eb",
            borderRadius: "8px",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const Empty: Story = {
  args: {},
  decorators: [
    (Story) => {

      const emptyModel: SrujaModelDump = {
        _stage: "parsed",
        projectId: "empty-project",
        project: {
          id: "empty-project",
          name: "Empty Architecture",
        },
        specification: { elements: {}, tags: {}, relationships: {} },
        globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
        elements: {},
        relations: [],
        views: {
          index: {
            id: "index",
            title: "Index",
            rules: [{ include: [{ wildcard: true }] }],
            nodes: [],
            edges: [],
          },
        },
      };

      vi.doMock("../../../../../apps/designer/src/stores", () => ({
        useArchitectureStore: () => ({
          likec4Model: emptyModel,
          updateArchitecture: fn(),
        }),
        useViewStore: () => ({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        }),
        useSelectionStore: () => ({
          selectedNodeId: null,
          selectNode: fn(),
          activeFlow: null,
          flowStep: 0,
          isFlowPlaying: false,
          playFlow: fn(),
          pauseFlow: fn(),
          nextStep: fn(),
          prevStep: fn(),
          setFlowStep: fn(),
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
            border: "1px solid #e5e7eb",
            borderRadius: "8px",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
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
