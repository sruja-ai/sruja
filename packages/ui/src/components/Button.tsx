// packages/ui/src/components/Button.tsx
import { forwardRef } from "react";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import { Button as MantineButton } from "@mantine/core";
import type { ButtonProps as MantineButtonProps } from "@mantine/core";

export interface ButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "color" | "size"> {
  /** Button content */
  children: ReactNode;
  /** Button variant */
  variant?: "primary" | "secondary" | "outline" | "ghost" | "danger";
  /** Button size */
  size?: "sm" | "md" | "lg";
  /** Whether the button is in loading state */
  isLoading?: boolean;
  /** Enable or disable auto tracking */
  track?: boolean;
  /** Optional tracking name */
  trackName?: string;
}

const variantMap: Record<NonNullable<ButtonProps["variant"]>, MantineButtonProps["variant"]> = {
  primary: "filled",
  secondary: "default",
  outline: "outline",
  ghost: "subtle",
  danger: "filled",
};

const colorMap: Record<NonNullable<ButtonProps["variant"]>, MantineButtonProps["color"]> = {
  primary: "blue",
  secondary: "gray",
  outline: "blue",
  ghost: "gray",
  danger: "red",
};

const sizeMap: Record<NonNullable<ButtonProps["size"]>, MantineButtonProps["size"]> = {
  sm: "xs",
  md: "sm",
  lg: "md",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    children,
    variant = "primary",
    size = "md",
    isLoading = false,
    track = true,
    trackName,
    disabled,
    className = "",
    ...props
  },
  ref
) {
  return (
    <MantineButton
      ref={ref}
      variant={variantMap[variant]}
      color={colorMap[variant]}
      size={sizeMap[size]}
      loading={isLoading}
      disabled={disabled}
      className={className}
      data-track={track ? "click" : undefined}
      data-component="button"
      data-name={trackName}
      {...props}
    >
      {children}
    </MantineButton>
  );
});
