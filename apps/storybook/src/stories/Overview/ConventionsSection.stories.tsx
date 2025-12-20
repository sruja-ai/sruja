import type { Meta, StoryObj } from "@storybook/react";
import { useEffect } from "react";
import { ConventionsSection } from "../../../../../apps/designer/src/components/Overview/ConventionsSection";
import { useFeatureFlagsStore } from "../../../../../apps/designer/src/stores/featureFlagsStore";
import { fn } from "@storybook/test";

const meta = {
  title: "Overview/ConventionsSection",
  component: ConventionsSection,
  decorators: [
    (Story) => {
      const setFlag = useFeatureFlagsStore((s) => s.setFlag);
      useEffect(() => {
        setFlag("conventions", true);
      }, [setFlag]);
      return <Story />;
    },
  ],
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  args: {
    onAddConvention: fn(),
    onEditConvention: fn(),
    onDeleteConvention: fn(),
  },
} satisfies Meta<typeof ConventionsSection>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    conventions: [
      {
        key: "Naming Strategy",
        value: "Use kebab-case for all API endpoints.",
      },
      {
        key: "Git Branching",
        value: "Follow Trunk Based Development.",
      },
    ],
  },
};

export const Empty: Story = {
  args: {
    conventions: [],
  },
};
