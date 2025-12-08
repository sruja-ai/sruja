// packages/shared/src/analytics/errorTracking.ts
import { capture } from './posthog';
import { logger } from '../utils/logger';

export interface ErrorContext {
  component?: string;
  action?: string;
  errorType?: string;
  errorCode?: string;
  severity?: 'error' | 'warning' | 'info';
  [key: string]: unknown;
}

/**
 * Sanitizes error messages to remove sensitive information
 */
function sanitizeError(error: unknown): string {
  if (error instanceof Error) {
    let message = error.message;
    
    // Remove common sensitive patterns
    const sensitivePatterns = [
      /Bearer\s+[\w-]+/gi,
      /token["\s:=]+[\w-]+/gi,
      /password["\s:=]+[^\s"']+/gi,
      /secret["\s:=]+[^\s"']+/gi,
      /api[_-]?key["\s:=]+[^\s"']+/gi,
      /access[_-]?token["\s:=]+[^\s"']+/gi,
      /client[_-]?secret["\s:=]+[^\s"']+/gi,
    ];
    
    sensitivePatterns.forEach((pattern) => {
      message = message.replace(pattern, '[REDACTED]');
    });
    
    return message;
  }
  
  return String(error);
}

/**
 * Captures an error event to PostHog with context
 * Also logs the error using the structured logger
 */
export function trackError(
  error: unknown,
  context: ErrorContext = {}
): void {
  const errorMessage = sanitizeError(error);
  const errorType = error instanceof Error ? error.constructor.name : typeof error;
  const errorStack = error instanceof Error ? error.stack : undefined;
  
  // Build error properties for PostHog
  const properties: Record<string, unknown> = {
    error_message: errorMessage,
    error_type: errorType,
    ...context,
  };
  
  // Add stack trace if available (truncated to first 500 chars)
  if (errorStack) {
    properties.error_stack = errorStack.substring(0, 500);
  }
  
  // Add browser/environment context
  if (typeof window !== 'undefined') {
    properties.user_agent = navigator.userAgent;
    properties.screen_size = `${window.screen.width}x${window.screen.height}`;
    properties.viewport_size = `${window.innerWidth}x${window.innerHeight}`;
    properties.url = window.location.href.substring(0, 200); // Limit URL length
  }
  
  // Determine event name based on context
  const eventName = context.component 
    ? `error.${context.component}.${context.action || 'unknown'}`
    : 'error.unknown';
  
  // Capture to PostHog
  capture(eventName, properties);
  
  // Log using structured logger
  const logContext: Record<string, unknown> = {
    ...context,
    errorType,
    errorMessage,
  };
  
  if (errorStack) {
    logContext.stack = errorStack.substring(0, 200);
  }
  
  const severity = context.severity || 'error';
  if (severity === 'error') {
    logger.error(`[${context.component || 'unknown'}] ${context.action || 'error'}: ${errorMessage}`, logContext);
  } else if (severity === 'warning') {
    logger.warn(`[${context.component || 'unknown'}] ${context.action || 'warning'}: ${errorMessage}`, logContext);
  } else {
    logger.info(`[${context.component || 'unknown'}] ${context.action || 'info'}: ${errorMessage}`, logContext);
  }
}

/**
 * Tracks a user interaction event
 */
export function trackInteraction(
  action: string,
  component: string,
  properties: Record<string, unknown> = {}
): void {
  const eventName = `interaction.${component}.${action}`;
  
  const fullProperties: Record<string, unknown> = {
    component,
    action,
    ...properties,
  };
  
  // Add browser context
  if (typeof window !== 'undefined') {
    fullProperties.timestamp = new Date().toISOString();
  }
  
  capture(eventName, fullProperties);
  logger.debug(`Interaction: ${component}.${action}`, fullProperties);
}

/**
 * Tracks a performance metric
 */
export function trackPerformance(
  metric: string,
  value: number,
  unit: string = 'ms',
  context: Record<string, unknown> = {}
): void {
  const eventName = `performance.${metric}`;
  
  capture(eventName, {
    metric,
    value,
    unit,
    ...context,
  });
  
  logger.debug(`Performance: ${metric} = ${value}${unit}`, context);
}

