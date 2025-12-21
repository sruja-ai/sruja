import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { LikeC4Canvas } from "../../../../../apps/designer/src/components/Canvas/LikeC4Canvas";
import type { SrujaModelDump } from "@sruja/shared";
import {
  useArchitectureStore,
  useViewStore,
  useSelectionStore,
  useUIStore,
  useFeatureFlagsStore,
} from "../../../../../apps/designer/src/stores";
import { useEffect } from "react";

// Comprehensive mock model with all element types and Sruja extensions
// ... [mockLikec4Model remains same]
const mockLikec4Model: SrujaModelDump = {
  _stage: "parsed",
  projectId: "storybook-project",
  project: {
    id: "storybook-project",
    name: "Storybook Architecture",
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
    customer: {
      id: "customer",
      kind: "person",
      title: "Customer",
      description: "End user of the system",
      technology: "",
      tags: [],
      links: [],
      style: {},
    },
    admin: {
      id: "admin",
      kind: "person",
      title: "Administrator",
      description: "System administrator",
      technology: "",
      tags: [],
      links: [],
      style: {},
    },
    "web-app": {
      id: "web-app",
      kind: "system",
      title: "Web Application",
      description: "Main web application for users",
      technology: "React",
      tags: ["frontend", "public"],
      links: [],
      style: {},
    },
    "api-service": {
      id: "api-service",
      kind: "system",
      title: "API Service",
      description: "Backend API service",
      technology: "Node.js",
      tags: ["backend", "api"],
      links: [],
      style: {},
    },
    "web-app.frontend": {
      id: "web-app.frontend",
      kind: "container",
      title: "Frontend Container",
      description: "React frontend application",
      technology: "React",
      tags: [],
      links: [],
      style: {},
    },
    "web-app.api": {
      id: "web-app.api",
      kind: "container",
      title: "API Container",
      description: "REST API server",
      technology: "Express.js",
      tags: [],
      links: [],
      style: {},
    },
    "web-app.database": {
      id: "web-app.database",
      kind: "database",
      title: "PostgreSQL Database",
      description: "Main data storage",
      technology: "PostgreSQL",
      tags: [],
      links: [],
      style: {},
    },
    "web-app.queue": {
      id: "web-app.queue",
      kind: "queue",
      title: "Message Queue",
      description: "Async message processing",
      technology: "RabbitMQ",
      tags: [],
      links: [],
      style: {},
    },
    "web-app.frontend.auth": {
      id: "web-app.frontend.auth",
      kind: "component",
      title: "Auth Component",
      description: "Authentication UI component",
      technology: "React",
      tags: [],
      links: [],
      style: {},
    },
  },
  relations: [
    {
      id: "rel-1",
      source: { model: "customer" },
      target: { model: "web-app" },
      title: "Uses",
      technology: "HTTPS",
    },
    {
      id: "rel-2",
      source: { model: "admin" },
      target: { model: "web-app" },
      title: "Manages",
      technology: "HTTPS",
    },
    {
      id: "rel-3",
      source: { model: "web-app" },
      target: { model: "api-service" },
      title: "Calls",
      technology: "REST",
    },
  ],
  views: {
    L1: {
      id: "L1",
      title: "System Context (L1)",
      rules: [
        {
          include: [
            { ref: { model: "customer" } },
            { ref: { model: "admin" } },
            { ref: { model: "web-app" } },
            { ref: { model: "api-service" } },
          ],
        },
      ],
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
  sruja: {
    requirements: [
      {
        id: "REQ-001",
        title: "User Authentication",
        description: "System must authenticate users securely",
        type: "security",
        tags: ["web-app", "api-service"],
      },
      {
        id: "REQ-002",
        title: "Data Persistence",
        description: "All user data must be persisted reliably",
        type: "reliability",
        tags: ["web-app.database"],
      },
    ],
    adrs: [
      {
        id: "ADR-001",
        title: "Choice of React for Frontend",
        context: "We needed to decide between React and Vue for our frontend",
        decision: "React was chosen for its ecosystem and team familiarity",
        consequences: "Large ecosystem, but steeper learning curve for new developers",
        status: "accepted",
        tags: ["web-app"],
      },
      {
        id: "ADR-002",
        title: "Database Technology Decision",
        context: "Choosing between SQL and NoSQL databases",
        decision: "",
        consequences: "",
        status: "pending",
        tags: ["web-app.database"],
      },
    ],
    scenarios: [
      {
        id: "scenario-login",
        title: "User Login Flow",
        description: "Complete user authentication process",
        steps: [
          { from: "customer", to: "web-app", description: "User enters credentials" },
          { from: "web-app", to: "api-service", description: "Validate credentials" },
          { from: "api-service", to: "web-app.database", description: "Check user data" },
        ],
      },
    ],
    flows: [
      {
        id: "flow-order",
        title: "Order Processing Flow",
        description: "How orders are processed through the system",
        steps: [
          { from: "customer", to: "web-app", description: "Submit order" },
          { from: "web-app", to: "web-app.queue", description: "Queue order" },
          { from: "web-app.queue", to: "api-service", description: "Process order" },
        ],
      },
    ],
    policies: [],
    constraints: [],
    conventions: [],
  },
};

const meta = {
  title: "Designer/LikeC4Canvas",
  component: LikeC4Canvas,
  parameters: {
    layout: "fullscreen",
    docs: {
      description: {
        component:
          "The main diagram canvas component that renders LikeC4 diagrams with node color coding, selection, and navigation sync.",
      },
    },
  },
  tags: ["autodocs"],
  argTypes: {
    viewId: {
      control: "text",
      description: "Specific view ID to display (L1, L2, L3, or custom)",
    },
    onNodeClick: {
      description: "Callback when a node is clicked",
    },
  },
  args: {
    viewId: undefined,
    onNodeClick: fn(),
  },
  decorators: [
    (Story) => {
      // Mock stores with proper structure using useEffect to avoid sync issues
      useEffect(() => {
        useArchitectureStore.setState({
          likec4Model: mockLikec4Model,
          updateArchitecture: async (_updater) => {}, // Simple stub
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useSelectionStore.setState({
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
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
          setEditMode: fn(),
        });
      }, []);

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
} satisfies Meta<typeof LikeC4Canvas>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {},
};

export const WithNodeSelection: Story = {
  args: {},
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          likec4Model: mockLikec4Model,
          updateArchitecture: async (_updater) => {},
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useSelectionStore.setState({
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
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
          setEditMode: fn(),
        });
      }, []);

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
  parameters: {
    docs: {
      description: {
        story:
          "Shows the canvas with a selected node. The selected node should be highlighted and clicking it shows related requirements, ADRs, scenarios, and flows in the Details panel.",
      },
    },
  },
};

export const L2ContainerView: Story = {
  args: {},
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          likec4Model: mockLikec4Model,
          updateArchitecture: async (_updater) => {},
        });
        useViewStore.setState({
          currentLevel: "L2",
          focusedSystemId: "web-app",
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useSelectionStore.setState({
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
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
          setEditMode: fn(),
        });
      }, []);

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
  parameters: {
    docs: {
      description: {
        story:
          "Shows the L2 (Container) view focused on the 'web-app' system, displaying its containers, databases, and queues.",
      },
    },
  },
};

export const L3ComponentView: Story = {
  args: {},
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          likec4Model: mockLikec4Model,
          updateArchitecture: async (_updater) => {},
        });
        useViewStore.setState({
          currentLevel: "L3",
          focusedSystemId: "web-app",
          focusedContainerId: "frontend",
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useSelectionStore.setState({
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
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
          setEditMode: fn(),
        });
      }, []);

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
  parameters: {
    docs: {
      description: {
        story:
          "Shows the L3 (Component) view focused on the 'frontend' container within 'web-app', displaying its internal components.",
      },
    },
  },
};

export const WithNodeColors: Story = {
  args: {},
  parameters: {
    docs: {
      description: {
        story:
          "Demonstrates node color coding by element type: Person (pink), System (blue), Container (green), Component (yellow), Database (purple), Queue (orange).",
      },
    },
  },
};

export const WithAnimation: Story = {
  args: {},
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          likec4Model: mockLikec4Model,
          updateArchitecture: async (_updater) => {},
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useSelectionStore.setState({
          selectedNodeId: null,
          selectNode: fn(),
          activeFlow: mockLikec4Model.sruja?.flows?.[0] || null,
          flowStep: 1,
          isFlowPlaying: true,
          playFlow: fn(),
          pauseFlow: fn(),
          nextStep: fn(),
          prevStep: fn(),
          setFlowStep: fn(),
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
          setEditMode: fn(),
        });
      }, []);

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
  parameters: {
    docs: {
      description: {
        story:
          "Shows the canvas with an active flow animation. Animation controls appear at the bottom to control playback.",
      },
    },
  },
};
