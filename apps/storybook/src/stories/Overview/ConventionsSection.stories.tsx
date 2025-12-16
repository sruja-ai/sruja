import type { Meta, StoryObj } from "@storybook/react";
import { ConventionsSection } from "../../../../apps/designer/src/components/Overview/ConventionsSection";
import { fn } from "@storybook/test";

const meta = {
  title: "Overview/ConventionsSection",
  component: ConventionsSection,
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  args: {
    onEdit: fn(),
  },
} satisfies Meta<typeof ConventionsSection>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    conventions: [
      {
        id: "conv-1",
        title: "Naming Strategy",
        description: "Use kebab-case for all API endpoints.",
        category: "API",
      },
      {
        id: "conv-2",
        title: "Git Branching",
        description: "Follow Trunk Based Development.",
        category: "Process",
      },
    ],
  },
};

export const Empty: Story = {
  args: {
    conventions: [],
  },
};
