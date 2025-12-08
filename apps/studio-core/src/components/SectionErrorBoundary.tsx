// apps/studio-core/src/components/SectionErrorBoundary.tsx
import React, { Component, ErrorInfo, ReactNode } from 'react';
import { logger } from '@sruja/shared';

interface SectionErrorBoundaryProps {
  children: ReactNode;
  sectionName: string;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface SectionErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

/**
 * SectionErrorBoundary provides error boundaries for specific sections of the app
 * This allows one section to fail without crashing the entire application
 */
export class SectionErrorBoundary extends Component<SectionErrorBoundaryProps, SectionErrorBoundaryState> {
  public state: SectionErrorBoundaryState = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): SectionErrorBoundaryState {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    const errorContext = {
      component: 'studio',
      action: 'section_error_boundary',
      section: this.props.sectionName,
      errorType: error?.constructor?.name || 'unknown',
      error: error?.message || String(error),
      errorInfo: errorInfo?.componentStack?.substring(0, 500),
    };

    logger.error(`Error in ${this.props.sectionName} section`, errorContext);

    // Call optional error handler (e.g., for analytics)
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex items-center justify-center h-full p-8 bg-[var(--color-background)]">
          <div className="text-center max-w-md">
            <div className="mb-4">
              <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-[var(--color-error-50)] flex items-center justify-center">
                <svg
                  className="w-8 h-8 text-[var(--color-error-500)]"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
              </div>
            </div>
            <h2 className="text-lg font-semibold text-[var(--color-text-primary)] mb-2">
              Error in {this.props.sectionName}
            </h2>
            <p className="text-sm text-[var(--color-text-secondary)] mb-4">
              {this.state.error?.message || 'An unexpected error occurred in this section'}
            </p>
            <div className="flex gap-2 justify-center">
              <button
                onClick={this.handleReset}
                className="px-4 py-2 bg-[var(--color-info-500)] text-white rounded-md hover:bg-[var(--color-info-600)] transition-colors text-sm"
              >
                Try Again
              </button>
              <button
                onClick={() => window.location.reload()}
                className="px-4 py-2 bg-[var(--color-surface)] text-[var(--color-text-primary)] rounded-md hover:bg-[var(--color-neutral-200)] transition-colors text-sm border border-[var(--color-border)]"
              >
                Reload Page
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}



