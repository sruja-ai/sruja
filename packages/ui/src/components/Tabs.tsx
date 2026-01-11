import { Tabs as MantineTabs } from "@mantine/core";

export type TabsProps = {
  tabs: Array<{ id: string; label: string; content: React.ReactNode }>;
  defaultIndex?: number;
  onChange?: (index: number) => void;
};

export function Tabs({ tabs, defaultIndex = 0, onChange }: TabsProps) {
  return (
    <MantineTabs
      defaultValue={tabs[defaultIndex]?.id}
      onChange={(value) => {
        const index = tabs.findIndex((t) => t.id === value);
        if (index !== -1) onChange?.(index);
      }}
    >
      <MantineTabs.List className="border-b border-[var(--color-border)]">
        {tabs.map((t) => (
          <MantineTabs.Tab
            key={t.id}
            value={t.id}
            className="text-sm px-4 py-2 data-[active]:border-b-2 data-[active]:border-blue-500"
          >
            {t.label}
          </MantineTabs.Tab>
        ))}
      </MantineTabs.List>

      {tabs.map((t) => (
        <MantineTabs.Panel key={t.id} value={t.id}>
          {t.content}
        </MantineTabs.Panel>
      ))}
    </MantineTabs>
  );
}
