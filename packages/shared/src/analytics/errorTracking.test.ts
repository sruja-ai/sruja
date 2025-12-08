// packages/shared/src/analytics/errorTracking.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { trackError, trackInteraction, trackPerformance } from './errorTracking';
import * as posthog from './posthog';

// Mock PostHog
vi.mock('./posthog', () => ({
  capture: vi.fn(),
}));

// Mock logger
vi.mock('../utils/logger', () => ({
  logger: {
    error: vi.fn(),
    warn: vi.fn(),
    info: vi.fn(),
    debug: vi.fn(),
  },
}));

describe('errorTracking', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('trackError', () => {
    it('should capture error to PostHog', () => {
      const error = new Error('test error');
      const context = {
        component: 'test',
        action: 'test_action',
        errorType: 'test_error',
      };

      trackError(error, context);

      expect(posthog.capture).toHaveBeenCalledWith(
        'error.test.test_action',
        expect.objectContaining({
          error_message: 'test error',
          error_type: 'Error',
          component: 'test',
          action: 'test_action',
        })
      );
    });

    it('should sanitize sensitive data in error messages', () => {
      const error = new Error('Bearer token12345 password=secret123');
      const context = { component: 'test', action: 'test' };

      trackError(error, context);

      const captureCall = (posthog.capture as any).mock.calls[0];
      expect(captureCall[1].error_message).not.toContain('token12345');
      expect(captureCall[1].error_message).not.toContain('secret123');
    });

    it('should include browser context when available', () => {
      // jsdom provides window, so we can test with it
      if (typeof window !== 'undefined') {
        // Mock window properties
        Object.defineProperty(window, 'navigator', {
          value: { userAgent: 'test-agent' },
          writable: true,
          configurable: true,
        });
        
        Object.defineProperty(window, 'screen', {
          value: { width: 1920, height: 1080 },
          writable: true,
          configurable: true,
        });
        
        Object.defineProperty(window, 'innerWidth', {
          value: 1280,
          writable: true,
          configurable: true,
        });
        
        Object.defineProperty(window, 'innerHeight', {
          value: 720,
          writable: true,
          configurable: true,
        });
        
        Object.defineProperty(window, 'location', {
          value: { href: 'https://test.com' },
          writable: true,
          configurable: true,
        });

        const error = new Error('test error');
        trackError(error, { component: 'test', action: 'test' });

        expect(posthog.capture).toHaveBeenCalled();
        const captureCall = (posthog.capture as any).mock.calls[0];
        const properties = captureCall[1];
        
        expect(properties.user_agent).toBe('test-agent');
        expect(properties.screen_size).toBe('1920x1080');
        expect(properties.viewport_size).toBe('1280x720');
        expect(properties.url).toBe('https://test.com');
      } else {
        // Skip test if window is not available
        expect(true).toBe(true);
      }
    });

    it('should handle non-Error objects', () => {
      const error = 'string error';
      trackError(error, { component: 'test', action: 'test' });

      expect(posthog.capture).toHaveBeenCalled();
    });
  });

  describe('trackInteraction', () => {
    it('should capture interaction to PostHog', () => {
      trackInteraction('click', 'button', { id: 'test-button' });

      expect(posthog.capture).toHaveBeenCalledWith(
        'interaction.button.click',
        expect.objectContaining({
          component: 'button',
          action: 'click',
          id: 'test-button',
        })
      );
    });

    it('should include timestamp when window is available', () => {
      Object.defineProperty(global, 'window', {
        value: {},
        writable: true,
      });

      trackInteraction('click', 'button');

      expect(posthog.capture).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          timestamp: expect.any(String),
        })
      );
    });
  });

  describe('trackPerformance', () => {
    it('should capture performance metric to PostHog', () => {
      trackPerformance('render_time', 150, 'ms', { component: 'viewer' });

      expect(posthog.capture).toHaveBeenCalledWith(
        'performance.render_time',
        expect.objectContaining({
          metric: 'render_time',
          value: 150,
          unit: 'ms',
          component: 'viewer',
        })
      );
    });

    it('should use default unit if not provided', () => {
      trackPerformance('load_time', 200);

      expect(posthog.capture).toHaveBeenCalledWith(
        'performance.load_time',
        expect.objectContaining({
          unit: 'ms',
        })
      );
    });
  });
});

