// apps/website/src/features/playground/components/Playground.tsx
// Wrapper component to use playground in the website
import React, { lazy, Suspense } from "react";
import { SrujaLoader, MantineProvider, ThemeProvider } from "@sruja/ui";
import "@sruja/ui/design-system/styles.css";
import AlgoliaSearch from "@/features/search/components/AlgoliaSearch";

// Lazy load the designer App component
// CSS files are imported within the App component itself
const PlaygroundApp = lazy(() =>
  import("@sruja/designer").then((m) => ({ default: m.default || m.App }))
);

class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { hasError: boolean }> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }
  static getDerivedStateFromError() {
    return { hasError: true };
  }
  componentDidCatch() {}
  render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-center justify-center h-full w-full">
          <div className="text-center" style={{ padding: 24 }}>
            <p className="text-[var(--color-text-secondary)]">Failed to load Playground.</p>
            <p className="text-[var(--color-text-tertiary)] mt-2">
              Please reload or try again later.
            </p>
          </div>
        </div>
      );
    }
    return this.props.children as React.ReactElement;
  }
}

export default function Playground() {
  return (
    <MantineProvider>
      <ThemeProvider defaultMode="system">
        <ErrorBoundary>
        <Suspense
          fallback={
            <div className="flex items-center justify-center h-full w-full">
              <div className="text-center">
                <SrujaLoader size={48} />
                <p className="text-[var(--color-text-secondary)] mt-4">Loading Playground...</p>
              </div>
            </div>
          }
        >
          <PlaygroundApp />
        </Suspense>
      </ErrorBoundary>
      </ThemeProvider>
    </MantineProvider>
  );
}
