import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { MantineProvider, ThemeProvider } from "@sruja/ui";
import "@sruja/ui/design-system/styles.css";
import App from "./App.tsx";
import { ErrorBoundary } from "./components/shared/ErrorBoundary";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ErrorBoundary>
      <MantineProvider>
        <ThemeProvider>
          <App />
        </ThemeProvider>
      </MantineProvider>
    </ErrorBoundary>
  </StrictMode>
);
