import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { HeroUIProvider } from "@heroui/react";
import { ThemeProvider } from "@sruja/ui";
import "@sruja/ui/design-system/styles.css";
import App from "./App.tsx";
import { ErrorBoundary } from "./components/shared/ErrorBoundary";

// Simple navigation function for HeroUI components
// Uses window.history for client-side navigation without full page reload
const navigate = (path: string, options?: { replace?: boolean }) => {
  if (options?.replace) {
    window.history.replaceState({}, "", path);
  } else {
    window.history.pushState({}, "", path);
  }
  // Dispatch popstate event to trigger re-renders if needed
  window.dispatchEvent(new PopStateEvent("popstate"));
};

// Convert router hrefs to native hrefs (for browser compatibility)
const useHref = (href: string) => href;

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ErrorBoundary>
      <HeroUIProvider navigate={navigate} useHref={useHref}>
        <ThemeProvider>
          <App />
        </ThemeProvider>
      </HeroUIProvider>
    </ErrorBoundary>
  </StrictMode>
);
