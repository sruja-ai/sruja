import type { Meta, StoryObj } from "@storybook/react";
import { OverviewHero } from "../../../../apps/designer/src/components/Overview/OverviewHero";
import { fn } from "@storybook/test";

const meta = {
  title: "Overview/OverviewHero",
  component: OverviewHero,
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  argTypes: {
    title: { control: "text" },
    description: { control: "text" },
  },
  args: {
    onEdit: fn(),
  },
} satisfies Meta<typeof OverviewHero>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    title: "E-Commerce Platform",
    description: "A comprehensive platform for online retail operations.",
    metadata: [
      { key: "Version", value: "1.0.0", category: "General" },
      { key: "Author", value: "Team Sruja", category: "General" },
      { key: "Status", value: "Active", category: "General" },
    ],
    stats: {
      systems: 3,
      persons: 2,
      containers: 5,
      components: 12,
      requirements: 8,
      adrs: 4,
    },
  },
};

export const WithoutMetadata: Story = {
  args: {
    title: "Simple System",
    description: "Just a simple system without metadata.",
    metadata: [],
    stats: {
      systems: 1,
      persons: 1,
      containers: 0,
      components: 0,
      requirements: 0,
      adrs: 0,
    },
  },
};

export const EditMode: Story = {
  args: {
    ...Default.args,
    // Edit mode is controlled by store/context in the real app,
    // but the component might receive an edit prop or we might need to mock the store
  },
  parameters: {
    // If we need to mock the store, we can do it here with a decorator
  },
};
