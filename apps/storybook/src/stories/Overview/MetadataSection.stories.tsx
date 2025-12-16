import type { Meta, StoryObj } from "@storybook/react";
import { MetadataSection } from "../../../../apps/designer/src/components/Overview/MetadataSection";
import { fn } from "@storybook/test";

const meta = {
  title: "Overview/MetadataSection",
  component: MetadataSection,
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  argTypes: {},
  args: {
    onAdd: fn(),
    onUpdate: fn(),
    onRemove: fn(),
  },
} satisfies Meta<typeof MetadataSection>;

export default meta;
type Story = StoryObj<typeof meta>;

// Mock hook usage if necessary, but this component mostly takes props
// If it uses useFeatureFlagsStore, we might need a decorator

export const Default: Story = {
  args: {
    metadata: [
      { key: "Version", value: "2.0.0", category: "General" },
      { key: "Owner", value: "Platform Team", category: "General" },
      {
        key: "Repo",
        value: "github.com/sruja/platform",
        category: "Links",
        link: "https://github.com/sruja/platform",
      },
      {
        key: "Docs",
        value: "wiki.internal/platform",
        category: "Links",
        link: "https://wiki.internal/platform",
      },
      { key: "React", value: "18.2.0", category: "Tech Stack" },
      { key: "TypeScript", value: "5.0.0", category: "Tech Stack" },
    ],
  },
};

export const Empty: Story = {
  args: {
    metadata: [],
  },
};
