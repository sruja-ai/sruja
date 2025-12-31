import { Popover } from "../../../../packages/ui/src/components/Popover";
import { Button } from "../../../../packages/ui/src/components/Button";
import type { Meta, StoryObj } from "@storybook/react";

const meta: Meta<typeof Popover> = {
  title: "Components/Popover",
  component: Popover,
  tags: ["autodocs"],
  parameters: { layout: "centered" },
  argTypes: {
    trigger: { control: false },
    placement: {
      control: { type: "select" },
      options: ["top", "bottom", "left", "right"],
    },
    children: { control: false },
  },
};

export default meta;
type Story = StoryObj<typeof Popover>;

export const Basic: Story = {
  args: {
    trigger: <Button>Toggle Popover</Button>,
    placement: "bottom" as const,
    children: <div>Popover content</div>,
  },
  render: (args) => (
    <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
      <Popover {...args} />
    </div>
  ),
};

export const Showcase: Story = {
  render: () => (
    <div className="grid grid-cols-2 gap-6 max-w-xl">
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
        <Popover trigger={<Button>Bottom</Button>} placement="bottom">
          <div className="text-sm text-[var(--color-text-secondary)]">Bottom content</div>
        </Popover>
      </div>
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
        <Popover trigger={<Button>Top</Button>} placement="top">
          <div className="text-sm text-[var(--color-text-secondary)]">Top content</div>
        </Popover>
      </div>
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
        <Popover trigger={<Button>Left</Button>} placement="left">
          <div className="text-sm text-[var(--color-text-secondary)]">Left content</div>
        </Popover>
      </div>
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
        <Popover trigger={<Button>Right</Button>} placement="right">
          <div className="text-sm text-[var(--color-text-secondary)]">Right content</div>
        </Popover>
      </div>
    </div>
  ),
};
