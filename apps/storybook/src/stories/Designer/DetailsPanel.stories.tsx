import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { DetailsPanel } from "../../../../../apps/designer/src/components/Panels/DetailsPanel";
import {
  useArchitectureStore,
  useSelectionStore,
  useUIStore,
} from "../../../../../apps/designer/src/stores";
import { useEffect } from "react";

// Mock store data for details panel
const mockModel = {
  elements: {
    "web-app": {
      id: "web-app",
      title: "Web Application",
      kind: "system",
      technology: "React",
      description: "Main web application for users",
      tags: ["frontend", "public-facing"],
    },
    user: { id: "user", title: "External User", kind: "person", technology: "" },
    api: {
      id: "api",
      title: "API Server",
      kind: "container",
      technology: "Node.js",
      description: "RESTful API server",
      parent: "web-app",
    },
    database: {
      id: "database",
      title: "Database",
      kind: "container",
      technology: "PostgreSQL",
      description: "Main data storage",
      parent: "web-app",
    },
  },
  relations: [
    { id: "rel-1", from: "user", to: "web-app", technology: "HTTPS", title: "User access" },
    { id: "rel-2", from: "web-app", to: "api", technology: "REST", title: "API calls" },
    { id: "rel-3", from: "api", to: "database", technology: "SQL", title: "Data persistence" },
  ],
  sruja: {
    requirements: [
      {
        id: "req-1",
        title: "User Authentication",
        description: "System must authenticate users securely",
        priority: "high",
        status: "active",
        tags: ["web-app", "api"],
        type: "Security",
      },
      {
        id: "req-2",
        title: "Data Persistence",
        description: "All user data must be persisted reliably",
        priority: "high",
        status: "active",
        tags: ["database"],
        type: "Reliability",
      },
    ],
    adrs: [
      {
        id: "adr-1",
        title: "Choice of REST over GraphQL",
        context: "We needed to decide between REST and GraphQL for our API design",
        decision: "REST was chosen for its simplicity and existing developer familiarity",
        consequences: "Simpler client implementation, but more API calls required",
        status: "accepted",
        tags: ["api"],
      },
      {
        id: "adr-2",
        title: "Database Technology Decision",
        context: "Choosing between SQL and NoSQL databases",
        decision: "",
        consequences: "",
        status: "pending",
        tags: ["database"],
      },
    ],
    flows: [
      {
        id: "flow-1",
        title: "User Login Flow",
        description: "Complete user authentication process",
        steps: [
          { from: "user", to: "web-app", description: "User enters credentials" },
          { from: "web-app", to: "api", description: "Validate credentials" },
        ],
      },
    ],
    scenarios: [
      {
        id: "scenario-1",
        title: "User Registration",
        description: "New user signs up for the service",
        steps: [
          { from: "user", to: "web-app", description: "User submits registration form" },
          { from: "web-app", to: "api", description: "Create user account" },
          { from: "api", to: "database", description: "Store user data" },
        ],
      },
    ],
  },
};

const meta = {
  title: "Designer/DetailsPanel",
  component: DetailsPanel,
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  argTypes: {
    onClose: {
      description: "Callback when panel is closed (mobile)",
    },
  },
  args: {
    onClose: fn(),
  },
  decorators: [
    (Story: React.ComponentType) => {
      // Mock stores
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          updateArchitecture: async (_updater) => {},
        });
        useSelectionStore.setState({
          selectedNodeId: "web-app",
          selectNode: fn(),
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "400px",
            height: "600px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
} satisfies Meta<typeof DetailsPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const SystemNode: Story = {
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          updateArchitecture: async (_updater) => {},
        });
        useSelectionStore.setState({
          selectedNodeId: "web-app",
          selectNode: fn(),
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "400px",
            height: "600px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const ContainerNode: Story = {
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          updateArchitecture: async (_updater) => {},
        });
        useSelectionStore.setState({
          selectedNodeId: "api",
          selectNode: fn(),
          hoveredNodeId: null,
          setHoveredNodeId: fn(),
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "400px",
            height: "600px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const PersonNode: Story = {
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          updateArchitecture: async (_updater) => {},
        });
        useSelectionStore.setState({
          selectedNodeId: "user",
          selectNode: fn(),
          hoveredNodeId: null,
          setHoveredNodeId: fn(),
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "400px",
            height: "600px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const NodeWithNoData: Story = {
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: {
            elements: {
              "empty-node": {
                id: "empty-node",
                title: "Empty Node",
                kind: "system",
                technology: "",
                description: "",
                tags: [],
              },
            },
            relations: [],
            sruja: {
              requirements: [],
              adrs: [],
              flows: [],
              scenarios: [],
            },
          } as unknown as Record<string, unknown>,
          updateArchitecture: async (_updater) => {},
        });
        useSelectionStore.setState({
          selectedNodeId: "empty-node",
          selectNode: fn(),
          hoveredNodeId: null,
          setHoveredNodeId: fn(),
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "400px",
            height: "600px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const NoSelection: Story = {
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          updateArchitecture: async (_updater) => {},
        });
        useSelectionStore.setState({
          selectedNodeId: null,
          selectNode: fn(),
          hoveredNodeId: null,
          setHoveredNodeId: fn(),
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "400px",
            height: "600px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const NoModel: Story = {
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: null,
          updateArchitecture: async (_updater) => {},
        });
        useSelectionStore.setState({
          selectedNodeId: "web-app",
          selectNode: fn(),
        });
        useUIStore.setState({
          activeTab: "diagram",
          setActiveTab: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "400px",
            height: "600px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};
