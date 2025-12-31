import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { NavigationPanel } from "../../../../../apps/designer/src/components/Panels/NavigationPanel";
import {
  useArchitectureStore,
  useViewStore,
  useFeatureFlagsStore,
} from "../../../../../apps/designer/src/stores";
import { useEffect } from "react";

// Mock store data
const mockModel = {
  elements: {
    user: { id: "user", title: "External User", kind: "person" },
    "web-app": {
      id: "web-app",
      title: "Web Application",
      kind: "system",
      parent: null,
    },
    api: {
      id: "api",
      title: "API Server",
      kind: "system",
      parent: null,
    },
    frontend: {
      id: "frontend",
      title: "Frontend",
      kind: "container",
      parent: "web-app",
    },
    backend: {
      id: "backend",
      title: "Backend Service",
      kind: "container",
      parent: "web-app",
    },
    "ui-component": {
      id: "ui-component",
      title: "Dashboard UI",
      kind: "component",
      parent: "frontend",
    },
  },
  relations: [],
};

const meta = {
  title: "Designer/NavigationPanel",
  component: NavigationPanel,
  parameters: {
    layout: "padded",
    backgrounds: {
      default: "light",
    },
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
} satisfies Meta<typeof NavigationPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  decorators: [
    (Story) => {
      // Mock the stores
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as any,
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view", // isEditMode returns false if editMode !== 'edit'
        });
      }, []);

      return (
        <div style={{ height: "500px", position: "relative" }}>
          <Story />
        </div>
      );
    },
  ],
};

export const Collapsed: Story = {
  decorators: [
    (Story) => {
      // Mock localStorage to start collapsed
      const originalLocalStorage = global.localStorage;
      global.localStorage = {
        ...originalLocalStorage,
        getItem: () => "true",
      };

      // Mock stores
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as any,
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
        });
      }, []);

      return (
        <div style={{ height: "500px", position: "relative" }}>
          <Story />
        </div>
      );
    },
  ],
};

export const WithFocus: Story = {
  decorators: [
    (Story) => {
      // Mock stores with focused system
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as any,
        });
        useViewStore.setState({
          currentLevel: "L2",
          focusedSystemId: "web-app",
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
        });
      }, []);

      return (
        <div style={{ height: "500px", position: "relative" }}>
          <Story />
        </div>
      );
    },
  ],
};

export const WithEditMode: Story = {
  decorators: [
    (Story) => {
      // Mock stores with edit mode enabled
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as any,
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "edit",
        });
      }, []);

      return (
        <div style={{ height: "500px", position: "relative" }}>
          <Story />
        </div>
      );
    },
  ],
};

export const Mobile: Story = {
  decorators: [
    (Story) => {
      // Mock stores
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel as any,
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
        });
      }, []);

      return (
        <div
          style={{
            width: "375px",
            height: "667px",
            position: "relative",
            border: "1px solid #ccc",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
};

export const Empty: Story = {
  decorators: [
    (Story) => {
      // Mock stores with empty model
      useEffect(() => {
        useArchitectureStore.setState({
          model: null,
        });
        useViewStore.setState({
          currentLevel: "L1",
          focusedSystemId: null,
          focusedContainerId: null,
          drillDown: fn(),
          goToRoot: fn(),
          setLevel: fn(),
        });
        useFeatureFlagsStore.setState({
          editMode: "view",
        });
      }, []);

      return (
        <div style={{ height: "500px", position: "relative" }}>
          <Story />
        </div>
      );
    },
  ],
};
