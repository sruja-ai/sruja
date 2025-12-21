// apps/social-publish/src/utils/error-handler.ts
import { logger } from '@sruja/shared';

/**
 * Sanitizes error messages to remove sensitive information
 */
export function sanitizeError(error: unknown): string {
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
      /[\w-]{20,}/g, // Long strings that might be tokens
    ];
    
    sensitivePatterns.forEach((pattern) => {
      message = message.replace(pattern, '[REDACTED]');
    });
    
    return message;
  }
  
  return String(error);
}

/**
 * Safely logs an error without exposing sensitive information
 */
export function logError(platform: string, error: unknown): void {
  const sanitized = sanitizeError(error);
  const context: Record<string, unknown> = {
    platform,
    sanitizedMessage: sanitized,
  };
  
  if (error instanceof Error) {
    context.errorType = error.constructor.name;
    context.errorName = error.name;
    if (error.stack) {
      // Sanitize stack trace too
      context.stack = sanitizeError(error);
    }
  }
  
  logger.error(`Failed to publish to ${platform}`, context);
}

/**
 * Wraps an async function with error handling that logs but doesn't throw
 */
export async function safeExecute<T>(
  platform: string,
  fn: () => Promise<T>
): Promise<T | null> {
  try {
    return await fn();
  } catch (error) {
    logError(platform, error);
    return null;
  }
}

