// apps/designer/src/components/shared/SentryInit.tsx
import { useEffect } from "react";

export function SentryInit() {
  useEffect(() => {
    // Only initialize Sentry in browser environment
    if (typeof window === "undefined") return;

    const dsn = import.meta.env.VITE_SENTRY_DSN;
    if (!dsn) {
      // Sentry DSN not configured - skip initialization
      return;
    }

    // Dynamic import to avoid bundling Sentry if not needed
    import("@sentry/react")
      .then((Sentry) => {
        Sentry.init({
          dsn,
          environment: import.meta.env.MODE || "development",
          tracesSampleRate: import.meta.env.PROD ? 0.1 : 1.0,
          replaysSessionSampleRate: import.meta.env.PROD ? 0.1 : 1.0,
          replaysOnErrorSampleRate: 1.0,
          integrations: [
            Sentry.browserTracingIntegration(),
            Sentry.replayIntegration({
              maskAllText: true,
              blockAllMedia: true,
            }),
          ],
          // Release tracking (set during build if available)
          release: import.meta.env.SENTRY_RELEASE,

          // Filter out known non-critical errors
          beforeSend(event, hint) {
            // Don't send errors in development unless explicitly enabled
            if (import.meta.env.DEV && !import.meta.env.VITE_SENTRY_DEBUG) {
              return null;
            }

            // Filter out browser extension errors
            if (event.exception) {
              const error = hint.originalException;
              if (error && typeof error === "object" && "message" in error) {
                const message = String(error.message);
                if (
                  message.includes("chrome-extension://") ||
                  message.includes("moz-extension://") ||
                  message.includes("safari-extension://") ||
                  message.includes("Non-Error promise rejection")
                ) {
                  return null;
                }
              }
            }

            return event;
          },

          // Ignore specific errors
          ignoreErrors: [
            // Browser extensions
            "top.GLOBALS",
            "originalCreateNotification",
            "canvas.contentDocument",
            "MyApp_RemoveAllHighlights",
            "atomicFindClose",
            "fb_xd_fragment",
            "bmi_SafeAddOnload",
            "EBCallBackMessageReceived",
            "conduitPage",
            // Network errors that are often not actionable
            "NetworkError",
            "Network request failed",
            "Failed to fetch",
            "Load failed",
            // ResizeObserver errors (common, usually harmless)
            "ResizeObserver loop limit exceeded",
            "ResizeObserver loop completed with undelivered notifications",
          ],
        });

        // Make Sentry available globally for error tracking utilities
        (window as { Sentry?: typeof Sentry }).Sentry = Sentry;
      })
      .catch((err) => {
        // Silently fail if Sentry package isn't available
        console.warn("Failed to initialize Sentry:", err);
      });
  }, []);

  return null;
}
