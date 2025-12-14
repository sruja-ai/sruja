// apps/playground/src/components/shared/ErrorBoundary.tsx
import { Component } from "react";
import type { ErrorInfo, ReactNode } from "react";
import { AlertCircle, RefreshCw } from "lucide-react";

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
    console.error("ErrorBoundary caught an error:", error, errorInfo);
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
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            minHeight: "100vh",
            padding: "2rem",
            backgroundColor: "var(--bg-primary, #ffffff)",
            color: "var(--text-primary, #000000)",
          }}
        >
          <div
            style={{
              maxWidth: "600px",
              width: "100%",
              padding: "2rem",
              border: "1px solid var(--border-color, #e5e7eb)",
              borderRadius: "8px",
              backgroundColor: "var(--bg-secondary, #f9fafb)",
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "0.75rem",
                marginBottom: "1rem",
              }}
            >
              <AlertCircle size={24} color="#ef4444" />
              <h2
                style={{
                  margin: 0,
                  fontSize: "1.25rem",
                  fontWeight: 600,
                  color: "var(--text-primary)",
                }}
              >
                Something went wrong
              </h2>
            </div>

            <p
              style={{
                margin: "0 0 1rem 0",
                color: "var(--text-secondary)",
                lineHeight: "1.5",
              }}
            >
              An unexpected error occurred. Please try refreshing the page or contact support if the
              problem persists.
            </p>

            {this.state.error && (
              <details
                style={{
                  marginBottom: "1rem",
                  padding: "1rem",
                  backgroundColor: "var(--bg-tertiary, #f3f4f6)",
                  borderRadius: "4px",
                  fontSize: "0.875rem",
                }}
              >
                <summary
                  style={{
                    cursor: "pointer",
                    fontWeight: 500,
                    marginBottom: "0.5rem",
                  }}
                >
                  Error Details
                </summary>
                <pre
                  style={{
                    margin: 0,
                    padding: "0.5rem",
                    overflow: "auto",
                    backgroundColor: "var(--bg-primary)",
                    borderRadius: "4px",
                    fontSize: "0.75rem",
                    color: "#dc2626",
                  }}
                >
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

            <button
              onClick={this.handleReset}
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "0.5rem",
                padding: "0.5rem 1rem",
                backgroundColor: "var(--highlight-color, #3b82f6)",
                color: "white",
                border: "none",
                borderRadius: "6px",
                cursor: "pointer",
                fontSize: "0.875rem",
                fontWeight: 500,
                transition: "background-color 0.2s",
              }}
              onMouseOver={(e) => {
                e.currentTarget.style.backgroundColor = "var(--highlight-color-hover, #2563eb)";
              }}
              onMouseOut={(e) => {
                e.currentTarget.style.backgroundColor = "var(--highlight-color, #3b82f6)";
              }}
            >
              <RefreshCw size={16} />
              Try Again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
