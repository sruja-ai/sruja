// apps/website/src/shared/utils/analytics.test.ts
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { trackEvent, trackPageView } from './analytics';

describe('analytics utilities', () => {
  let dispatchEventSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    dispatchEventSpy = vi.spyOn(window, 'dispatchEvent');
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('trackEvent', () => {
    it('dispatches a custom event', () => {
      trackEvent('test.event', { key: 'value' });

      expect(dispatchEventSpy).toHaveBeenCalledTimes(1);
      const call = dispatchEventSpy.mock.calls[0][0] as CustomEvent;
      expect(call.detail.type).toBe('test.event');
      expect(call.detail.key).toBe('value');
    });

    it('works without detail object', () => {
      trackEvent('test.event');

      expect(dispatchEventSpy).toHaveBeenCalledTimes(1);
      const call = dispatchEventSpy.mock.calls[0][0] as CustomEvent;
      expect(call.detail.type).toBe('test.event');
    });

    it('does not throw if window is undefined', () => {
      const originalWindow = global.window;
      // @ts-expect-error - testing window unavailability
      delete global.window;
      
      expect(() => trackEvent('test.event')).not.toThrow();
      
      global.window = originalWindow;
    });
  });

  describe('trackPageView', () => {
    it('tracks page view with path and title', () => {
      trackPageView('/test-path', 'Test Page');

      expect(dispatchEventSpy).toHaveBeenCalledTimes(1);
      const call = dispatchEventSpy.mock.calls[0][0] as CustomEvent;
      expect(call.detail.type).toBe('page.view');
      expect(call.detail.path).toBe('/test-path');
      expect(call.detail.title).toBe('Test Page');
    });
  });
});
