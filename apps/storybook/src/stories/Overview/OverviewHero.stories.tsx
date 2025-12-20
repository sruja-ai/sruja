import type { Meta, StoryObj } from "@storybook/react";
import { OverviewHero } from "../../../../../apps/designer/src/components/Overview/OverviewHero";

const meta = {
  title: "Overview/OverviewHero",
  component: OverviewHero,
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  argTypes: {
    architectureName: { control: "text" },
    description: { control: "text" },
    onEditOverview: { action: "edit clicked" },
  },
  args: {
    onEditOverview: () => {},
  },
} satisfies Meta<typeof OverviewHero>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    architectureName: "E-Commerce Platform",
    description: "A comprehensive platform for online retail operations.",
    overview: {
      summary: "This architecture supports the full e-commerce lifecycle from product catalog to order fulfillment.",
    },
    archMetadata: [
      { key: "Version", value: "1.0.0" },
      { key: "Author", value: "Team Sruja" },
      { key: "Status", value: "Active" },
    ],
  },
};

export const WithoutMetadata: Story = {
  args: {
    architectureName: "Simple System",
    description: "Just a simple system without metadata.",
    overview: undefined,
    archMetadata: [],
  },
};

export const EditMode: Story = {
  args: {
    ...Default.args,
  },
  // Note: Edit mode is controlled by useFeatureFlagsStore in the component
  // To see the edit button, the store needs to be initialized with editMode: "edit"
  // This can be done via browser localStorage or by mocking the store
};
