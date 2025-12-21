import type { Meta, StoryObj } from "@storybook/react";
import { useEffect } from "react";
import { ConstraintsSection } from "../../../../../apps/designer/src/components/Overview/ConstraintsSection";
import { useFeatureFlagsStore } from "../../../../../apps/designer/src/stores/featureFlagsStore";
import { fn } from "@storybook/test";

const meta = {
  title: "Overview/ConstraintsSection",
  component: ConstraintsSection,
  decorators: [
    (Story) => {
      const setFlag = useFeatureFlagsStore((s) => s.setFlag);
      useEffect(() => {
        setFlag("constraints", true);
      }, [setFlag]);
      return <Story />;
    },
  ],
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  args: {
    onAddConstraint: fn(),
    onEditConstraint: fn(),
    onDeleteConstraint: fn(),
  },
} satisfies Meta<typeof ConstraintsSection>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    constraints: [
      {
        key: "Budget",
        value: "Monthly cloud spend must not exceed $5000.",
      },
      {
        key: "Compliance",
        value: "Must be GDPR compliant.",
      },
    ],
  },
};

export const Empty: Story = {
  args: {
    constraints: [],
  },
};
