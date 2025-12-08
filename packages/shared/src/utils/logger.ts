// packages/shared/src/utils/logger.ts
import { capture } from '../analytics/posthog';

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LogContext {
  [key: string]: unknown;
}

interface LogEntry {
  level: LogLevel;
  message: string;
  context?: LogContext;
  timestamp: string;
  service?: string;
}

let debugEnabled = false;
let serviceName: string | undefined;

const isProd = typeof process !== 'undefined' && process.env && process.env.NODE_ENV === 'production';

function isDebug(): boolean {
  if (debugEnabled) return true;
  if (typeof window !== 'undefined') {
    const v = (window as any).__SRUJA_DEBUG__;
    if (typeof v === 'boolean') return v;
    try {
      const ls = window.localStorage?.getItem('sruja:debug');
      if (ls === 'true') return true;
    } catch { void 0; }
  }
  return !isProd;
}

function formatLogEntry(entry: LogEntry): string {
  const parts = [
    entry.timestamp,
    `[${entry.level.toUpperCase()}]`,
    entry.service ? `[${entry.service}]` : '',
    entry.message,
  ].filter(Boolean);
  
  return parts.join(' ');
}

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
      console.error(formatted, context ? JSON.stringify(context, null, 2) : '');
      
      // Send errors to PostHog for tracking
      try {
        const component = (context?.component as string) || 'unknown';
        const action = (context?.action as string) || 'error';
        const eventName = `error.${component}.${action}`;
        
        const properties: Record<string, unknown> = {
          error_message: message,
          error_type: (context?.errorType as string) || 'unknown',
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
      } catch (e) {
        // Silently fail if PostHog is not available
      }
    } else {
      console.warn(formatted, context ? JSON.stringify(context, null, 2) : '');
    }
    return;
  }

  // Debug/info only in debug mode
  if (!isDebug()) return;

  const formatted = isProd ? formatStructured(entry) : formatLogEntry(entry);
  if (level === 'info') {
    console.info(formatted, context ? JSON.stringify(context, null, 2) : '');
  } else {
    console.debug(formatted, context ? JSON.stringify(context, null, 2) : '');
  }
}

export const logger = {
  /**
   * Set the service name for all subsequent logs
   */
  setService(name: string): void {
    serviceName = name;
  },

  /**
   * Enable debug logging
   */
  enableDebug(): void {
    debugEnabled = true;
  },

  /**
   * Disable debug logging
   */
  disableDebug(): void {
    debugEnabled = false;
  },

  /**
   * Log a debug message with optional context
   */
  debug(message: string, context?: LogContext): void {
    log('debug', message, context);
  },

  /**
   * Log an info message with optional context
   */
  info(message: string, context?: LogContext): void {
    log('info', message, context);
  },

  /**
   * Log a warning message with optional context
   */
  warn(message: string, context?: LogContext): void {
    log('warn', message, context);
  },

  /**
   * Log an error message with optional context
   */
  error(message: string, context?: LogContext): void {
    log('error', message, context);
  },
};
