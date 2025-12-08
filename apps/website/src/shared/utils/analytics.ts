// apps/website/src/shared/utils/analytics.ts
import { capture, trackInteraction as posthogTrackInteraction } from '@sruja/shared';

/**
 * Track a custom event
 */
export function trackEvent(type: string, detail?: Record<string, any>): void {
  if (typeof window === 'undefined') return;
  
  try {
    window.dispatchEvent(
      new CustomEvent('sruja:event', {
        detail: { type, ...detail },
      })
    );
    
    // Also send to PostHog
    capture(`event.${type}`, detail || {});
  } catch (e) {
    console.warn('Failed to track event:', e);
  }
}

/**
 * Track a page view
 */
export function trackPageView(path: string, title?: string): void {
  trackEvent('page.view', { path, title });
}

/**
 * Track a user interaction
 */
export function trackInteraction(action: string, element: string, properties?: Record<string, any>): void {
  posthogTrackInteraction(`interaction.${action}`, { element, ...properties });
}
