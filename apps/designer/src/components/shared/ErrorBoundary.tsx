// apps/playground/src/components/shared/ErrorBoundary.tsx
import { Component } from "react";
import type { ErrorInfo, ReactNode } from "react";
import { AlertCircle, RefreshCw } from "lucide-react";
import { Button } from "@sruja/ui";
import { logger } from "@sruja/shared";
import "./ErrorState.css";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
      errorInfo: null,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    logger.error("ErrorBoundary caught an error", {
      component: "ErrorBoundary",
      action: "componentDidCatch",
      error: {
        message: error.message,
        name: error.name,
        stack: error.stack,
      },
      errorInfo: {
        componentStack: errorInfo.componentStack,
      },
    });
    this.setState({
      error,
      errorInfo,
    });
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="error-state" style={{ minHeight: "100vh" }}>
          <div
            style={{
              maxWidth: "600px",
              width: "100%",
              padding: "var(--space-8, 2rem)",
              border: "1px solid var(--border-color, #e5e7eb)",
              borderRadius: "var(--radius-lg, 0.75rem)",
              backgroundColor: "var(--bg-secondary, #f9fafb)",
              boxShadow: "var(--elevation-3, 0 4px 6px -1px rgba(0, 0, 0, 0.1))",
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "var(--space-3, 0.75rem)",
                marginBottom: "var(--space-4, 1rem)",
              }}
            >
              <AlertCircle size={32} className="error-icon" />
              <h2 className="error-title">Something went wrong</h2>
            </div>

            <p className="error-message">
              An unexpected error occurred. Please try refreshing the page or contact support if the
              problem persists.
            </p>

            {this.state.error && (
              <details
                style={{
                  marginBottom: "var(--space-4, 1rem)",
                  padding: "var(--space-4, 1rem)",
                  backgroundColor: "var(--bg-tertiary, #f3f4f6)",
                  borderRadius: "var(--radius-md, 0.5rem)",
                  fontSize: "var(--text-sm, 0.875rem)",
                }}
              >
                <summary
                  style={{
                    cursor: "pointer",
                    fontWeight: "var(--font-medium, 500)",
                    marginBottom: "var(--space-2, 0.5rem)",
                  }}
                >
                  Error Details
                </summary>
                <pre className="error-code">
                  {this.state.error.toString()}
                  {this.state.errorInfo?.componentStack && (
                    <>
                      {"\n\nComponent Stack:"}
                      {this.state.errorInfo.componentStack}
                    </>
                  )}
                </pre>
              </details>
            )}

            <div className="error-actions">
              <Button
                variant="primary"
                size="md"
                onClick={this.handleReset}
                className="upload-btn"
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: "var(--space-2, 0.5rem)",
                }}
              >
                <RefreshCw size={16} />
                Try Again
              </Button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
