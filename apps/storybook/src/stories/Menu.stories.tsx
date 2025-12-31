import { Menu } from "../../../../packages/ui/src/components/Menu";
import { Button } from "../../../../packages/ui/src/components/Button";

const meta = {
  title: "UI/Menu",
  component: Menu,
  tags: ["autodocs"],
  argTypes: {
    trigger: { control: false },
    items: { control: false },
    placement: {
      control: { type: "select" },
      options: ["top", "bottom", "left", "right"],
    },
  },
};

export default meta;

export const Basic = {
  args: {
    trigger: <Button>Open Menu</Button>,
    items: [
      { label: "Item One", onClick: () => {} },
      { label: "Item Two", onClick: () => {}, divider: true },
      { label: "Delete", onClick: () => {}, danger: true },
    ],
  },
  render: (_Story: React.ComponentType) => (
    <div style={{ padding: 32 }}>
      <Menu {...args} />
    </div>
  ),
};
