import type { Meta, StoryObj } from "@storybook/react";
import { DetailsView } from "../../../../../apps/designer/src/components/Views/DetailsView";
import { useArchitectureStore, useSelectionStore } from "../../../../../apps/designer/src/stores";
import { useEffect } from "react";
import type { SrujaModelDump } from "@sruja/shared";

// Mock model with comprehensive Sruja extensions
const mockModel: SrujaModelDump = {
  _stage: "parsed",
  projectId: "storybook-project",
  project: { id: "storybook-project", name: "Storybook Architecture" },
  specification: { elements: {}, tags: {}, relationships: {} },
  globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
  elements: {
    "web-app": {
      id: "web-app",
      kind: "system",
      title: "Web Application",
      description: "Main web application",
      technology: "React",
      tags: [],
      links: [],
      style: {},
    },
    "api-service": {
      id: "api-service",
      kind: "system",
      title: "API Service",
      description: "Backend API",
      technology: "Node.js",
      tags: [],
      links: [],
      style: {},
    },
  },
  relations: [],
  views: {},
  sruja: {
    requirements: [
      {
        id: "REQ-001",
        title: "User Authentication",
        description: "System must authenticate users securely",
        type: "security",
        tags: ["web-app"],
      },
      {
        id: "REQ-002",
        title: "API Rate Limiting",
        description: "API must implement rate limiting",
        type: "performance",
        tags: ["api-service"],
      },
    ],
    adrs: [
      {
        id: "ADR-001",
        title: "Choice of React",
        context: "Frontend framework decision",
        decision: "React chosen for ecosystem",
        consequences: "Large ecosystem available",
        status: "accepted",
        tags: ["web-app"],
      },
    ],
    scenarios: [
      {
        id: "scenario-login",
        title: "User Login",
        description: "User authentication flow",
        steps: [{ from: "customer", to: "web-app", description: "Enter credentials" }],
      },
    ],
    flows: [
      {
        id: "flow-order",
        title: "Order Processing",
        description: "Order processing flow",
        steps: [{ from: "customer", to: "web-app", description: "Submit order" }],
      },
    ],
    policies: [],
    constraints: [],
    conventions: [],
  },
};

const meta = {
  title: "Designer/DetailsView",
  component: DetailsView,
  parameters: {
    layout: "fullscreen",
    docs: {
      description: {
        component:
          "Unified details view showing requirements, ADRs, scenarios, and flows with filtering capabilities. Automatically filters by selected node when a node is clicked in the diagram.",
      },
    },
  },
  tags: ["autodocs"],
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel,
        });
        useSelectionStore.setState({
          selectedNodeId: null,
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
} satisfies Meta<typeof DetailsView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  parameters: {
    docs: {
      description: {
        story: "Shows all requirements, ADRs, scenarios, and flows with filter sidebar.",
      },
    },
  },
};

export const WithNodeSelection: Story = {
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel,
        });
        useSelectionStore.setState({
          selectedNodeId: "web-app",
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
          "Shows items filtered by selected node 'web-app'. The list automatically filters to show only items tagged with or referencing the selected node.",
      },
    },
  },
};

export const WithTypeFilter: Story = {
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          model: mockModel,
        });
        useSelectionStore.setState({
          selectedNodeId: null,
        });
      }, []);

      // Simulate type filter being set via sub-tab switch event
      useEffect(() => {
        const timer = setTimeout(() => {
          window.dispatchEvent(
            new CustomEvent("switch-details-subtab", { detail: "requirements" })
          );
        }, 100);
        return () => clearTimeout(timer);
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
          "Shows only requirements when filtered by type (simulated via sub-tab switch event).",
      },
    },
  },
};
