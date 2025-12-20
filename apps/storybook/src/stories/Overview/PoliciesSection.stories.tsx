import type { Meta, StoryObj } from "@storybook/react";
import { useEffect } from "react";
import { PoliciesSection } from "../../../../../apps/designer/src/components/Overview/PoliciesSection";
import { useFeatureFlagsStore } from "../../../../../apps/designer/src/stores/featureFlagsStore";
import { fn } from "@storybook/test";

const meta = {
  title: "Overview/PoliciesSection",
  component: PoliciesSection,
  decorators: [
    (Story) => {
      const setFlag = useFeatureFlagsStore((s) => s.setFlag);
      useEffect(() => {
        setFlag("policies", true);
      }, [setFlag]);
      return <Story />;
    },
  ],
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  args: {
    onAddPolicy: fn(),
    onEditPolicy: fn(),
    policyCount: 2,
  },
} satisfies Meta<typeof PoliciesSection>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    policyCount: 2,
    policies: [
      {
        id: "pol-1",
        label: "Data Privacy",
        description: "All PII must be encrypted at rest and in transit.",
        category: "Security",
      },
      {
        id: "pol-2",
        label: "Logging",
        description: "All services must log structured JSON to the central aggregator.",
        category: "Operations",
      },
    ],
  },
};

export const Empty: Story = {
  args: {
    policies: [],
  },
};
