import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { BuilderWizard } from "../../../../../apps/designer/src/components/Wizard/BuilderWizard";
import {
  useArchitectureStore,
  useViewStore,
  useSelectionStore,
  useUIStore,
  useFeatureFlagsStore,
} from "../../../../../apps/designer/src/stores";
import { useEffect } from "react";

// Mock store data for builder wizard
const mockModel = {
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
    { id: "rel-1", from: "user", to: "web-app", technology: "HTTPS" },
    { id: "rel-2", from: "web-app", to: "api", technology: "REST" },
    { id: "rel-3", from: "api", to: "database", technology: "SQL" },
  ],
  sruja: {
    requirements: [
      {
        id: "req-1",
        title: "User Authentication",
        description: "System must authenticate users securely",
        priority: "high",
        status: "active",
        category: "Security",
      },
    ],
    scenarios: [
      {
        id: "scenario-1",
        title: "User Login Flow",
        description: "User logs into the system",
        steps: [
          "User enters credentials",
          "System validates credentials",
          "System returns session token",
        ],
      },
    ],
  },
};

const meta = {
  title: "Designer/BuilderWizard",
  component: BuilderWizard,
  parameters: {
    layout: "fullscreen",
  },
  tags: ["autodocs"],
  decorators: [
    (Story: React.ComponentType) => {
      // Mock stores
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          dslSource: "// Generated DSL content",
          setDslSource: fn(),
          updateArchitecture: async (_updater) => {},
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
        });
        useSelectionStore.setState({
          selectedNodeId: null,
          setSelectedNodeId: fn(),
          hoveredNodeId: null,
          setHoveredNodeId: fn(),
        });
        useUIStore.setState({
          activeTab: "builder",
          setActiveTab: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "edit",
          setEditMode: fn(),
        });
      }, []);

      // Mock localStorage for step persistence
      const originalLocalStorage = global.localStorage;
      global.localStorage = {
        ...originalLocalStorage,
        getItem: () => null,
        setItem: fn(),
        removeItem: fn(),
      };

      return (
        <div
          style={{
            width: "100%",
            height: "800px",
            position: "relative",
            overflow: "auto",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
} satisfies Meta<typeof BuilderWizard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  decorators: [
    (Story: React.ComponentType) => {
      // Mock with full model
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          dslSource: "// Generated DSL content",
          setDslSource: fn(),
          updateArchitecture: async (_updater) => {},
        });
        useFeatureFlagsStore.setState({
          editMode: "edit",
          setEditMode: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "100%",
            height: "800px",
            position: "relative",
            overflow: "auto",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const ViewMode: Story = {
  decorators: [
    (Story: React.ComponentType) => {
      // Mock with view mode
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          dslSource: "// Generated DSL content",
          setDslSource: fn(),
          updateArchitecture: async (_updater) => {},
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
            height: "800px",
            position: "relative",
            overflow: "auto",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const EmptyModel: Story = {
  decorators: [
    (Story: React.ComponentType) => {
      // Mock with empty model
      useEffect(() => {
        useArchitectureStore.setState({
          model: null,
          dslSource: "// No DSL content",
          setDslSource: fn(),
          updateArchitecture: async (_updater) => {},
        });
        useFeatureFlagsStore.setState({
          editMode: "edit",
          setEditMode: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "100%",
            height: "800px",
            position: "relative",
            overflow: "auto",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const WithSidebarOpen: Story = {
  decorators: [
    (Story: React.ComponentType) => {
      // Mock with sidebar preference
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          dslSource: "// Generated DSL content",
          setDslSource: fn(),
          updateArchitecture: async (_updater) => {},
        });
        useFeatureFlagsStore.setState({
          editMode: "edit",
          setEditMode: fn(),
        });
      }, []);

      // Mock localStorage with sidebar preference
      const originalLocalStorage = global.localStorage;
      global.localStorage = {
        ...originalLocalStorage,
        getItem: (key: string) => {
          if (key === "playground:previewSidebar") return "on";
          return null;
        },
        setItem: fn(),
        removeItem: fn(),
      };

      return (
        <div
          style={{
            width: "100%",
            height: "800px",
            position: "relative",
            overflow: "auto",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const Mobile: Story = {
  decorators: [
    (Story: React.ComponentType) => {
      // Mock for mobile view
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as unknown as Record<string, unknown>,
          dslSource: "// Generated DSL content",
          setDslSource: fn(),
          updateArchitecture: async (_updater) => {},
        });
        useFeatureFlagsStore.setState({
          editMode: "edit",
          setEditMode: fn(),
        });
      }, []);

      return (
        <div
          style={{
            width: "375px",
            height: "800px",
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
