// apps/website/src/shared/components/SentryInit.tsx
import { useEffect } from 'react';
import * as Sentry from '@sentry/react';
import { envConfig } from '../../config/env';

export function SentryInit() {
  useEffect(() => {
    if (!envConfig.sentry?.dsn) return;
    
    Sentry.init({
      dsn: envConfig.sentry.dsn,
      environment: envConfig.sentry.environment,
      tracesSampleRate: envConfig.sentry.tracesSampleRate,
      replaysSessionSampleRate: envConfig.sentry.replaysSessionSampleRate,
      replaysOnErrorSampleRate: envConfig.sentry.replaysOnErrorSampleRate,
      integrations: [
        Sentry.browserTracingIntegration(),
        Sentry.replayIntegration({
          maskAllText: true,
          blockAllMedia: true,
        }),
      ],
      // Release tracking (set during build)
      release: import.meta.env.SENTRY_RELEASE,
      
      // Filter out known non-critical errors
      beforeSend(event, hint) {
        // Don't send errors in development unless explicitly enabled
        if (envConfig.env === 'development' && !import.meta.env.PUBLIC_SENTRY_DEBUG) {
          return null;
        }
        
        // Filter out browser extension errors
        if (event.exception) {
          const error = hint.originalException;
          if (error && typeof error === 'object' && 'message' in error) {
            const message = String(error.message);
            if (
              message.includes('chrome-extension://') ||
              message.includes('moz-extension://') ||
              message.includes('safari-extension://') ||
              message.includes('Non-Error promise rejection')
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
        'top.GLOBALS',
        'originalCreateNotification',
        'canvas.contentDocument',
        'MyApp_RemoveAllHighlights',
        'atomicFindClose',
        'fb_xd_fragment',
        'bmi_SafeAddOnload',
        'EBCallBackMessageReceived',
        'conduitPage',
        // Network errors that are often not actionable
        'NetworkError',
        'Network request failed',
        'Failed to fetch',
        'Load failed',
        // ResizeObserver errors (common, usually harmless)
        'ResizeObserver loop limit exceeded',
        'ResizeObserver loop completed with undelivered notifications',
      ],
    });
    
    // Make Sentry available globally for error tracking
    if (typeof window !== 'undefined') {
      (window as { Sentry?: typeof Sentry }).Sentry = Sentry;
    }
  }, []);

  return null;
}

