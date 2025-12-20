// apps/designer/src/components/shared/forms/__tests__/FormField.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { FormField } from "../FormField";

// Mock Mantine components
vi.mock("@sruja/ui", () => ({
  Input: ({ label, value, onChange, error, disabled, helperText, ...props }: any) => (
    <div data-testid="input-wrapper">
      {label && <label>{label}</label>}
      <input
        data-testid="input"
        value={value}
        onChange={onChange}
        disabled={disabled}
        {...props}
      />
      {error && <span data-testid="error">{error}</span>}
      {helperText && <span data-testid="helper">{helperText}</span>}
    </div>
  ),
  Textarea: ({ label, value, onChange, error, disabled, helperText, rows, ...props }: any) => (
    <div data-testid="textarea-wrapper">
      {label && <label>{label}</label>}
      <textarea
        data-testid="textarea"
        value={value}
        onChange={onChange}
        disabled={disabled}
        rows={rows}
        {...props}
      />
      {error && <span data-testid="error">{error}</span>}
      {helperText && <span data-testid="helper">{helperText}</span>}
    </div>
  ),
}));

describe("FormField", () => {
  const defaultProps = {
    label: "Test Field",
    name: "testField",
    value: "",
    onChange: vi.fn(),
  };

  describe("Input field", () => {
    it("should render input field by default", () => {
      render(<FormField {...defaultProps} />);
      expect(screen.getByTestId("input")).toBeInTheDocument();
    });

    it("should render label", () => {
      render(<FormField {...defaultProps} />);
      expect(screen.getByText("Test Field")).toBeInTheDocument();
    });

    it("should display value", () => {
      render(<FormField {...defaultProps} value="Test Value" />);
      const input = screen.getByTestId("input") as HTMLInputElement;
      expect(input.value).toBe("Test Value");
    });

    it("should call onChange when value changes", async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<FormField {...defaultProps} onChange={onChange} />);

      const input = screen.getByTestId("input");
      await user.type(input, "New Value");

      expect(onChange).toHaveBeenCalled();
    });

    it("should display error message", () => {
      render(<FormField {...defaultProps} error="This field is required" />);
      expect(screen.getByText("This field is required")).toBeInTheDocument();
    });

    it("should display helper text", () => {
      render(<FormField {...defaultProps} description="Helpful hint" />);
      expect(screen.getByText("Helpful hint")).toBeInTheDocument();
    });

    it("should be disabled when disabled prop is true", () => {
      render(<FormField {...defaultProps} disabled />);
      const input = screen.getByTestId("input") as HTMLInputElement;
      expect(input.disabled).toBe(true);
    });

    it("should support email type", () => {
      render(<FormField {...defaultProps} type="email" />);
      const input = screen.getByTestId("input") as HTMLInputElement;
      expect(input.type).toBe("email");
    });

    it("should support url type", () => {
      render(<FormField {...defaultProps} type="url" />);
      const input = screen.getByTestId("input") as HTMLInputElement;
      expect(input.type).toBe("url");
    });

    it("should show required indicator when required", () => {
      render(<FormField {...defaultProps} required />);
      const input = screen.getByTestId("input");
      expect(input).toHaveAttribute("required");
    });

    it("should display placeholder", () => {
      render(<FormField {...defaultProps} placeholder="Enter value" />);
      const input = screen.getByTestId("input") as HTMLInputElement;
      expect(input.placeholder).toBe("Enter value");
    });
  });

  describe("Textarea field", () => {
    it("should render textarea when type is textarea", () => {
      render(<FormField {...defaultProps} type="textarea" />);
      expect(screen.getByTestId("textarea")).toBeInTheDocument();
      expect(screen.queryByTestId("input")).not.toBeInTheDocument();
    });

    it("should display textarea value", () => {
      render(<FormField {...defaultProps} type="textarea" value="Long text value" />);
      const textarea = screen.getByTestId("textarea") as HTMLTextAreaElement;
      expect(textarea.value).toBe("Long text value");
    });

    it("should call onChange when textarea value changes", async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<FormField {...defaultProps} type="textarea" onChange={onChange} />);

      const textarea = screen.getByTestId("textarea");
      await user.type(textarea, "New text");

      expect(onChange).toHaveBeenCalled();
    });

    it("should support rows prop", () => {
      render(<FormField {...defaultProps} type="textarea" rows={5} />);
      const textarea = screen.getByTestId("textarea") as HTMLTextAreaElement;
      expect(textarea.rows).toBe(5);
    });

    it("should display error message for textarea", () => {
      render(<FormField {...defaultProps} type="textarea" error="Textarea error" />);
      expect(screen.getByText("Textarea error")).toBeInTheDocument();
    });

    it("should display helper text for textarea", () => {
      render(<FormField {...defaultProps} type="textarea" description="Textarea hint" />);
      expect(screen.getByText("Textarea hint")).toBeInTheDocument();
    });

    it("should be disabled when disabled prop is true", () => {
      render(<FormField {...defaultProps} type="textarea" disabled />);
      const textarea = screen.getByTestId("textarea") as HTMLTextAreaElement;
      expect(textarea.disabled).toBe(true);
    });
  });
});
