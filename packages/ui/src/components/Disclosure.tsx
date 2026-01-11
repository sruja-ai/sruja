import { Accordion as MantineAccordion } from "@mantine/core";

export type DisclosureProps = {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
};

export function Disclosure({ title, children, defaultOpen }: DisclosureProps) {
  return (
    <MantineAccordion
      defaultValue={defaultOpen ? "0" : undefined}
      classNames={{
        item: "border border-[var(--color-border)] rounded-md overflow-hidden",
        control: "px-4 py-3 hover:bg-[var(--color-surface)] text-left",
        content: "px-4 py-3 bg-[var(--color-surface)]",
        label: "text-[var(--color-text-primary)] font-medium",
      }}
    >
      <MantineAccordion.Item value="0">
        <MantineAccordion.Control>{title}</MantineAccordion.Control>
        <MantineAccordion.Panel>{children}</MantineAccordion.Panel>
      </MantineAccordion.Item>
    </MantineAccordion>
  );
}
