import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { CommandPalette } from "../../../../../apps/designer/src/components/shared/CommandPalette";
import { type Command } from "../../../../../apps/designer/src/components/shared/CommandPalette";
import { Eye, Layout, List, FileCode, Download, Upload, Settings } from "lucide-react";

const meta = {
  title: "Designer/CommandPalette",
  component: CommandPalette,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {
    isOpen: {
      control: "boolean",
      description: "Whether the command palette is open",
    },
    commands: {
      description: "Array of available commands",
    },
    onClose: {
      description: "Callback when palette is closed",
    },
  },
  args: {
    isOpen: true,
    onClose: fn(),
    commands: [
      {
        id: "nav-overview",
        label: "Go to Overview",
        description: "View architecture overview",
        icon: <Eye size={16} />,
        category: "navigation",
        action: fn(),
        keywords: ["overview", "summary"],
      },
      {
        id: "nav-diagram",
        label: "Go to Diagram",
        description: "View architecture diagram",
        icon: <Layout size={16} />,
        category: "navigation",
        action: fn(),
        keywords: ["diagram", "visual", "graph"],
      },
      {
        id: "nav-details",
        label: "Go to Details",
        description: "View detailed information",
        icon: <List size={16} />,
        category: "navigation",
        action: fn(),
        keywords: ["details", "info"],
      },
      {
        id: "nav-code",
        label: "Go to Code",
        description: "View architecture DSL",
        icon: <FileCode size={16} />,
        category: "navigation",
        action: fn(),
        keywords: ["code", "dsl"],
      },
      {
        id: "import-file",
        label: "Import .sruja file",
        description: "Import architecture from file",
        icon: <Upload size={16} />,
        category: "actions",
        action: fn(),
        keywords: ["import", "open", "load"],
      },
      {
        id: "export-json",
        label: "Export .sruja",
        description: "Export architecture to file",
        icon: <Download size={16} />,
        category: "export",
        action: fn(),
        keywords: ["export", "save", "download"],
      },
      {
        id: "settings-prefs",
        label: "Preferences",
        description: "Application settings",
        icon: <Settings size={16} />,
        category: "settings",
        action: fn(),
        keywords: ["settings", "preferences", "config"],
      },
    ] as Command[],
  },
} satisfies Meta<typeof CommandPalette>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Open: Story = {
  args: {
    isOpen: true,
  },
};

export const Closed: Story = {
  args: {
    isOpen: false,
  },
};

export const WithSearch: Story = {
  args: {
    isOpen: true,
  },
  decorators: [
    (Story) => {
      return (
        <div style={{ minHeight: "600px" }}>
          <Story />
          <div
            style={{
              marginTop: "20px",
              padding: "16px",
              backgroundColor: "#f5f5f5",
              borderRadius: "8px",
              fontSize: "14px",
              color: "#666",
            }}
          >
            <strong>Note:</strong> To see the search functionality, click the input field and type.
            Try searching for {"\""}diagram{"\""}, {"\""}export{"\""}, or {"\""}import{"\""}.
          </div>
        </div>
      );
    },
  ],
};

export const MinimalCommands: Story = {
  args: {
    isOpen: true,
    commands: [
      {
        id: "save",
        label: "Save",
        description: "Save current architecture",
        category: "actions",
        action: fn(),
      },
      {
        id: "load",
        label: "Load",
        description: "Load architecture",
        category: "actions",
        action: fn(),
      },
    ] as Command[],
  },
};

export const NoCommands: Story = {
  args: {
    isOpen: true,
    commands: [],
  },
};

export const KeyboardNavigation: Story = {
  args: {
    isOpen: true,
  },
  decorators: [
    (Story) => {
      return (
        <div style={{ minHeight: "600px" }}>
          <Story />
          <div
            style={{
              marginTop: "20px",
              padding: "16px",
              backgroundColor: "#e3f2fd",
              borderRadius: "8px",
              fontSize: "14px",
            }}
          >
            <strong>Keyboard Navigation:</strong>
            <ul style={{ margin: "8px 0 0 0", paddingLeft: "20px" }}>
              <li>
                <kbd>↑</kbd> / <kbd>↓</kbd> - Navigate commands
              </li>
              <li>
                <kbd>Enter</kbd> - Execute selected command
              </li>
              <li>
                <kbd>Esc</kbd> - Close palette
              </li>
              <li>Start typing to filter commands</li>
            </ul>
          </div>
        </div>
      );
    },
  ],
};

export const WithCategories: Story = {
  args: {
    isOpen: true,
    commands: [
      // Navigation commands
      {
        id: "nav-diagram",
        label: "Go to Diagram",
        description: "View architecture diagram",
        icon: <Layout size={16} />,
        category: "navigation",
        action: fn(),
      },
      {
        id: "nav-details",
        label: "Go to Details",
        description: "View detailed information",
        icon: <List size={16} />,
        category: "navigation",
        action: fn(),
      },
      // Action commands
      {
        id: "new-project",
        label: "New Project",
        description: "Create a new architecture project",
        category: "actions",
        action: fn(),
      },
      {
        id: "import",
        label: "Import File",
        description: "Import architecture from file",
        icon: <Upload size={16} />,
        category: "actions",
        action: fn(),
      },
      // Export commands
      {
        id: "export-json",
        label: "Export .sruja",
        description: "Export architecture to file",
        icon: <Download size={16} />,
        category: "export",
        action: fn(),
      },
      {
        id: "export-png",
        label: "Export PNG",
        description: "Export diagram as image",
        icon: <Download size={16} />,
        category: "export",
        action: fn(),
      },
      // Settings commands
      {
        id: "settings",
        label: "Settings",
        description: "Application settings",
        icon: <Settings size={16} />,
        category: "settings",
        action: fn(),
      },
    ] as Command[],
  },
};
