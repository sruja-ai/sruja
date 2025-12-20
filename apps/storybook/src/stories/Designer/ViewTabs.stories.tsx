import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { ViewTabs } from "../../../../../apps/designer/src/components/ViewTabs";

const meta = {
  title: "Designer/ViewTabs",
  component: ViewTabs,
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  argTypes: {
    activeTab: {
      control: { type: "select" },
      options: ["builder", "diagram", "details", "code"],
      description: "Currently active tab",
    },
    counts: {
      control: { type: "object" },
      description: "Tab counts for badges",
    },
  },
  args: {
    onTabChange: fn(),
    counts: {
      requirements: 12,
      adrs: 5,
    },
  },
} satisfies Meta<typeof ViewTabs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const BuilderActive: Story = {
  args: {
    activeTab: "builder",
  },
};

export const DiagramActive: Story = {
  args: {
    activeTab: "diagram",
  },
};

export const DetailsActive: Story = {
  args: {
    activeTab: "details",
  },
};

export const CodeActive: Story = {
  args: {
    activeTab: "code",
  },
};

export const WithCounts: Story = {
  args: {
    activeTab: "details",
    counts: {
      requirements: 8,
      adrs: 3,
    },
  },
};

export const NoCounts: Story = {
  args: {
    activeTab: "details",
    counts: {
      requirements: 0,
      adrs: 0,
    },
  },
};

export const HighCounts: Story = {
  args: {
    activeTab: "details",
    counts: {
      requirements: 25,
      adrs: 15,
    },
  },
};
