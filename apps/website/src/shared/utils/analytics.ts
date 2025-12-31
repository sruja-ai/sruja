// apps/website/src/shared/utils/analytics.ts
import { capture, trackInteraction as posthogTrackInteraction } from "@sruja/shared";

/**
 * Track a custom event
 */
export function trackEvent(name: string, properties: Record<string, unknown> = {}) {
  if (typeof window === "undefined") return;

  try {
    window.dispatchEvent(
      new CustomEvent("sruja:event", {
        detail: { name, ...properties },
      })
    );

    // Also send to PostHog
    capture(`event.${name}`, properties);
  } catch (e) {
    console.warn("Failed to track event:", e);
  }
}

/**
 * Track a page view
 */
export function trackPageView(path: string, title?: string): void {
  trackEvent("page.view", { path, title });
}

/**
 * Track a user interaction
 */
export function trackInteraction(
  action: string,
  element: string,
  properties: Record<string, unknown> = {}
): void {
  posthogTrackInteraction(action, element, properties);
}
