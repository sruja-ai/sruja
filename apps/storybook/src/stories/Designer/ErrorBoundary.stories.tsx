import React from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { ErrorBoundary } from "../../../../../apps/designer/src/components/shared/ErrorBoundary";

// Component that throws errors for testing
const ThrowingComponent = ({
  shouldThrow = false,
  children,
}: {
  shouldThrow?: boolean;
  children?: React.ReactNode;
}) => {
  if (shouldThrow) {
    throw new Error("This is a test error for ErrorBoundary");
  }
  return <div>{children || "Normal content that renders fine"}</div>;
};

// Async error component
const AsyncErrorComponent = () => {
  const [shouldThrow, setShouldThrow] = React.useState(false);

  React.useEffect(() => {
    const timer = setTimeout(() => setShouldThrow(true), 100);
    return () => clearTimeout(timer);
  }, []);

  if (shouldThrow) {
    throw new Error("Async error occurred after component mounted");
  }

  return <div>This component will throw an error after 100ms</div>;
};

const meta = {
  title: "Designer/ErrorBoundary",
  component: ErrorBoundary,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {
    fallback: {
      description: "Custom fallback UI to render when an error occurs",
      control: "text",
    },
  },
  args: {
    fallback: undefined,
  },
} satisfies Meta<typeof ErrorBoundary>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NormalOperation: Story = {
  args: {
    children: <div className="test-content">Everything is working normally</div>,
  },
};

export const ThrowingError: Story = {
  args: {
    children: <ThrowingComponent shouldThrow={true} />,
  },
};

export const WithCustomFallback: Story = {
  args: {
    fallback: (
      <div
        style={{
          padding: "20px",
          backgroundColor: "#fee2e2",
          borderRadius: "8px",
          border: "1px solid #fecaca",
        }}
      >
        <h3>Custom Error Fallback</h3>
        <p>This is a custom error message instead of the default one.</p>
      </div>
    ),
    children: <ThrowingComponent shouldThrow={true} />,
  },
};

export const NestedErrorBoundary: Story = {
  args: {
    children: (
      <div>
        <h3>Parent ErrorBoundary (working)</h3>
        <ErrorBoundary
          fallback={
            <div
              style={{
                padding: "16px",
                backgroundColor: "#fef3c7",
                borderRadius: "8px",
                border: "1px solid #fde68a",
                margin: "16px 0",
              }}
            >
              <strong>Nested Error Caught</strong>
              <p>This error was caught by the nested ErrorBoundary.</p>
            </div>
          }
        >
          <div>
            <h4>Nested ErrorBoundary</h4>
            <ThrowingComponent shouldThrow={true}>
              <p>This content won't render due to the error</p>
            </ThrowingComponent>
          </div>
        </ErrorBoundary>
        <p>Parent content continues normally</p>
      </div>
    ),
  },
};

export const AsyncError: Story = {
  args: {
    children: <AsyncErrorComponent />,
  },
  decorators: [
    (Story) => {
      return (
        <div>
          <div
            style={{
              marginBottom: "16px",
              padding: "12px",
              backgroundColor: "#e0f2fe",
              borderRadius: "8px",
              fontSize: "14px",
            }}
          >
            <strong>Async Error Test:</strong> This story will throw an error after 100ms to
            simulate async errors.
          </div>
          <Story />
        </div>
      );
    },
  ],
};

export const ComplexComponentError: Story = {
  args: {
    children: (
      <div>
        <h2>Complex Component Tree</h2>
        <div>
          <ul>
            <li>Item 1</li>
            <li>Item 2</li>
            <li>
              <ThrowingComponent shouldThrow={true}>
                <span>Nested content that fails</span>
              </ThrowingComponent>
            </li>
            <li>Item 4</li>
          </ul>
        </div>
      </div>
    ),
  },
};

export const CustomErrorMessage: Story = {
  args: {
    children: <ThrowingComponent shouldThrow={true} />,
  },
  decorators: [
    (Story) => {
      return (
        <div>
          <div
            style={{
              marginBottom: "16px",
              padding: "12px",
              backgroundColor: "#f0f9ff",
              borderRadius: "8px",
              fontSize: "14px",
            }}
          >
            <strong>Test Error Details:</strong> The error below shows detailed information
            including the error message and component stack trace. Click "Error Details" to expand
            and see the full error information.
          </div>
          <Story />
        </div>
      );
    },
  ],
};
