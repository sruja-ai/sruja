import { forwardRef } from "react";
import { Switch as MantineSwitch } from "@mantine/core";
import type { SwitchProps as MantineSwitchProps } from "@mantine/core";

export type SwitchProps = Omit<MantineSwitchProps, "label"> & {
  label?: string;
};

export const Switch = forwardRef<HTMLInputElement, SwitchProps>(function Switch(
  { label, ...props },
  ref
) {
  return (
    <div className="flex items-center gap-2">
      {label && (
        <span
          className={`text-sm text-[var(--color-text-secondary)] ${props.disabled ? "opacity-50" : ""}`}
        >
          {label}
        </span>
      )}
      <MantineSwitch ref={ref} {...props} />
    </div>
  );
});
