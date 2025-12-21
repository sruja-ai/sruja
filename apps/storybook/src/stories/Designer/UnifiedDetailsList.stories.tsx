import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { UnifiedDetailsList } from "../../../../../apps/designer/src/components/Details/UnifiedDetailsList";
import {
  useSelectionStore,
  useArchitectureStore,
  useUIStore,
} from "../../../../../apps/designer/src/stores";
import { useEffect } from "react";
import type { FilterState } from "../../../../../apps/designer/src/components/Details/DetailsSidebarFilters";
import type { SrujaModelDump } from "@sruja/shared";

// Mock model with comprehensive Sruja extensions
const mockLikec4Model: SrujaModelDump = {
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
      {
        id: "REQ-003",
        title: "Data Encryption",
        description: "All data must be encrypted at rest",
        type: "security",
        tags: ["web-app", "api-service"],
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
      {
        id: "ADR-002",
        title: "API Design",
        context: "REST vs GraphQL",
        decision: "",
        consequences: "",
        status: "pending",
        tags: ["api-service"],
      },
    ],
    scenarios: [
      {
        id: "scenario-login",
        title: "User Login",
        description: "User authentication flow",
        steps: [
          { from: "customer", to: "web-app", description: "Enter credentials" },
          { from: "web-app", to: "api-service", description: "Validate" },
        ],
      },
    ],
    flows: [
      {
        id: "flow-order",
        title: "Order Processing",
        description: "Order processing flow",
        steps: [
          { from: "customer", to: "web-app", description: "Submit order" },
          { from: "web-app", to: "api-service", description: "Process" },
        ],
      },
    ],
    policies: [],
    constraints: [],
    conventions: [],
  },
};

const defaultFilters: FilterState = {
  types: new Set(),
  statuses: new Set(),
  tags: new Set(),
  searchQuery: "",
};

const meta = {
  title: "Designer/UnifiedDetailsList",
  component: UnifiedDetailsList,
  parameters: {
    layout: "padded",
    docs: {
      description: {
        component:
          "Unified list showing requirements, ADRs, scenarios, and flows. Automatically filters by selected node when a node is clicked in the diagram.",
      },
    },
  },
  tags: ["autodocs"],
  argTypes: {
    filters: {
      description: "Filter state for the list",
    },
    selectedNodeId: {
      control: "text",
      description: "Selected node ID - automatically filters items tagged with this node",
    },
    onItemClick: {
      description: "Callback when an item is clicked",
    },
  },
  args: {
    filters: defaultFilters,
    selectedNodeId: null,
    onItemClick: fn(),
  },
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          likec4Model: mockLikec4Model,
        });
        useUIStore.setState({
          setActiveTab: fn(),
        });
        useSelectionStore.setState({
          selectedNodeId: null,
        });
      }, []);

      return (
        <div
          style={{
            width: "600px",
            height: "500px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #e5e7eb",
            borderRadius: "8px",
            padding: "16px",
          }}
        >
          <Story />
        </div>
      );
    },
  ],
} satisfies Meta<typeof UnifiedDetailsList>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    filters: defaultFilters,
  },
};

export const FilteredByNode: Story = {
  args: {
    filters: defaultFilters,
    selectedNodeId: "web-app",
  },
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          likec4Model: mockLikec4Model,
        });
        useUIStore.setState({
          setActiveTab: fn(),
        });
        useSelectionStore.setState({
          selectedNodeId: "web-app",
        });
      }, []);

      return (
        <div
          style={{
            width: "600px",
            height: "500px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #e5e7eb",
            borderRadius: "8px",
            padding: "16px",
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
          "Shows items filtered by selected node 'web-app'. Only requirements, ADRs, scenarios, and flows tagged with or referencing 'web-app' are displayed.",
      },
    },
  },
};

export const FilteredByType: Story = {
  args: {
    filters: {
      types: new Set(["requirement"]),
      statuses: new Set(),
      tags: new Set(),
      searchQuery: "",
    },
  },
  parameters: {
    docs: {
      description: {
        story: "Shows only requirements when filtered by type.",
      },
    },
  },
};

export const FilteredByStatus: Story = {
  args: {
    filters: {
      types: new Set(["adr"]),
      statuses: new Set(["partial"]),
      tags: new Set(),
      searchQuery: "",
    },
  },
  parameters: {
    docs: {
      description: {
        story: "Shows only ADRs with partial status (pending decisions).",
      },
    },
  },
};

export const WithSearch: Story = {
  args: {
    filters: {
      types: new Set(),
      statuses: new Set(),
      tags: new Set(),
      searchQuery: "authentication",
    },
  },
  parameters: {
    docs: {
      description: {
        story: "Shows items matching the search query 'authentication'.",
      },
    },
  },
};

export const EmptyState: Story = {
  args: {
    filters: {
      types: new Set(["requirement"]),
      statuses: new Set(),
      tags: new Set(),
      searchQuery: "nonexistent",
    },
  },
  parameters: {
    docs: {
      description: {
        story: "Shows empty state when no items match the filters.",
      },
    },
  },
};

export const EmptyStateWithNode: Story = {
  args: {
    filters: defaultFilters,
    selectedNodeId: "nonexistent-node",
  },
  decorators: [
    (Story) => {
      useEffect(() => {
        useArchitectureStore.setState({
          likec4Model: mockLikec4Model,
        });
        useUIStore.setState({
          setActiveTab: fn(),
        });
        useSelectionStore.setState({
          selectedNodeId: "nonexistent-node",
        });
      }, []);

      return (
        <div
          style={{
            width: "600px",
            height: "500px",
            position: "relative",
            overflow: "auto",
            border: "1px solid #e5e7eb",
            borderRadius: "8px",
            padding: "16px",
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
        story: "Shows empty state message when no items are tagged with the selected node.",
      },
    },
  },
};
