// packages/shared/src/utils/logger.ts
// Structured logging utility with analytics integration

import { capture } from '../analytics/posthog';

/**
 * Log severity levels.
 * 
 * @public
 */
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

/**
 * Context object for structured logging.
 * 
 * @public
 * @remarks
 * Can contain any key-value pairs for additional context.
 * Common keys: component, action, errorType, userId, etc.
 */
export interface LogContext {
  readonly [key: string]: unknown;
}

/**
 * Internal log entry structure.
 * 
 * @internal
 */
interface LogEntry {
  readonly level: LogLevel;
  readonly message: string;
  readonly context?: LogContext;
  readonly timestamp: string;
  readonly service?: string;
}

let debugEnabled = false;
let serviceName: string | undefined;

/**
 * Check if running in production environment.
 * 
 * @internal
 */
const isProd =
  typeof process !== 'undefined' &&
  process.env !== undefined &&
  process.env.NODE_ENV === 'production';

/**
 * Check if debug logging is enabled.
 * 
 * @internal
 * @returns true if debug logging should be enabled
 * 
 * @remarks
 * Debug is enabled if:
 * - Explicitly enabled via enableDebug()
 * - window.__SRUJA_DEBUG__ is true
 * - localStorage 'sruja:debug' is 'true'
 * - Not in production environment
 */
function isDebug(): boolean {
  if (debugEnabled) {
    return true;
  }
  if (typeof window !== 'undefined') {
    const windowDebug = (window as { __SRUJA_DEBUG__?: boolean })
      .__SRUJA_DEBUG__;
    if (typeof windowDebug === 'boolean') {
      return windowDebug;
    }
    try {
      const ls = window.localStorage?.getItem('sruja:debug');
      if (ls === 'true') {
        return true;
      }
    } catch {
      // Silently fail if localStorage is unavailable
    }
  }
  return !isProd;
}

/**
 * Format log entry as human-readable string.
 * 
 * @internal
 * @param entry - The log entry to format
 * @returns Formatted log string
 */
function formatLogEntry(entry: LogEntry): string {
  const parts = [
    entry.timestamp,
    `[${entry.level.toUpperCase()}]`,
    entry.service ? `[${entry.service}]` : '',
    entry.message,
  ].filter(Boolean);

  return parts.join(' ');
}

/**
 * Format log entry as structured JSON.
 * 
 * @internal
 * @param entry - The log entry to format
 * @returns JSON string representation
 */
function formatStructured(entry: LogEntry): string {
  const structured: Record<string, unknown> = {
    timestamp: entry.timestamp,
    level: entry.level,
    message: entry.message,
  };

  if (entry.service) {
    structured.service = entry.service;
  }

  if (entry.context && Object.keys(entry.context).length > 0) {
    structured.context = entry.context;
  }

  return JSON.stringify(structured);
}

/**
 * Internal logging function.
 * 
 * @internal
 * @param level - Log severity level
 * @param message - Log message
 * @param context - Optional context object
 * 
 * @remarks
 * - Errors and warnings are always logged
 * - Debug and info are only logged in debug mode
 * - Errors are automatically sent to PostHog analytics
 */
function log(level: LogLevel, message: string, context?: LogContext): void {
  const timestamp = new Date().toISOString();
  const entry: LogEntry = {
    level,
    message,
    context,
    timestamp,
    service: serviceName,
  };

  // Always log errors and warnings
  if (level === 'error' || level === 'warn') {
    const formatted = isProd ? formatStructured(entry) : formatLogEntry(entry);
    if (level === 'error') {
      console.error(
        formatted,
        context ? JSON.stringify(context, null, 2) : ''
      );

      // Send errors to PostHog for tracking
      try {
        const component =
          (typeof context?.component === 'string'
            ? context.component
            : undefined) || 'unknown';
        const action =
          (typeof context?.action === 'string'
            ? context.action
            : undefined) || 'error';
        const eventName = `error.${component}.${action}`;

        const properties: Record<string, unknown> = {
          error_message: message,
          error_type:
            (typeof context?.errorType === 'string'
              ? context.errorType
              : undefined) || 'unknown',
          ...context,
        };

        // Add browser context
        if (typeof window !== 'undefined') {
          properties.user_agent = navigator.userAgent;
          properties.screen_size = `${window.screen.width}x${window.screen.height}`;
          properties.viewport_size = `${window.innerWidth}x${window.innerHeight}`;
          properties.url = window.location.href.substring(0, 200);
        }

        capture(eventName, properties);
      } catch {
        // Silently fail if PostHog is not available
      }
    } else {
      console.warn(
        formatted,
        context ? JSON.stringify(context, null, 2) : ''
      );
    }
    return;
  }

  // Debug/info only in debug mode
  if (!isDebug()) {
    return;
  }

  const formatted = isProd ? formatStructured(entry) : formatLogEntry(entry);
  if (level === 'info') {
    console.info(
      formatted,
      context ? JSON.stringify(context, null, 2) : ''
    );
  } else {
    console.debug(
      formatted,
      context ? JSON.stringify(context, null, 2) : ''
    );
  }
}

/**
 * Structured logger with analytics integration.
 * 
 * @public
 * @remarks
 * Provides structured logging with:
 * - Multiple log levels (debug, info, warn, error)
 * - Contextual information support
 * - Automatic error tracking via PostHog
 * - Environment-aware formatting (structured JSON in prod, human-readable in dev)
 * 
 * @example
 * logger.setService('my-service');
 * logger.info('User logged in', { userId: '123', action: 'login' });
 * logger.error('Failed to process request', { errorType: 'network', component: 'api' });
 */
export const logger = {
  /**
   * Set the service name for all subsequent logs.
   * 
   * @param name - Service identifier
   */
  setService(name: string): void {
    serviceName = name;
  },

  /**
   * Enable debug logging globally.
   * 
   * @remarks
   * Overrides environment-based debug detection.
   */
  enableDebug(): void {
    debugEnabled = true;
  },

  /**
   * Disable debug logging globally.
   * 
   * @remarks
   * Returns to environment-based debug detection.
   */
  disableDebug(): void {
    debugEnabled = false;
  },

  /**
   * Log a debug message with optional context.
   * 
   * @param message - Debug message
   * @param context - Optional context object
   */
  debug(message: string, context?: LogContext): void {
    log('debug', message, context);
  },

  /**
   * Log an info message with optional context.
   * 
   * @param message - Info message
   * @param context - Optional context object
   */
  info(message: string, context?: LogContext): void {
    log('info', message, context);
  },

  /**
   * Log a warning message with optional context.
   * 
   * @param message - Warning message
   * @param context - Optional context object
   */
  warn(message: string, context?: LogContext): void {
    log('warn', message, context);
  },

  /**
   * Log an error message with optional context.
   * 
   * @param message - Error message
   * @param context - Optional context object
   * 
   * @remarks
   * Errors are automatically sent to PostHog analytics for tracking.
   */
  error(message: string, context?: LogContext): void {
    log('error', message, context);
  },
} as const;
