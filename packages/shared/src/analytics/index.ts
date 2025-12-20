// packages/shared/src/analytics/index.ts
// Analytics and error tracking utilities

/**
 * @packageDocumentation
 * 
 * # Analytics Module
 * 
 * Analytics and error tracking utilities for Sruja applications.
 * 
 * ## Available Modules
 * 
 * - **posthog**: PostHog analytics integration
 * - **errorTracking**: Error tracking and reporting
 * - **auto**: Automatic event tracking
 * 
 * ## Usage
 * 
 * ```typescript
 * import { capture, identify } from '@sruja/shared/analytics';
 * import { trackError } from '@sruja/shared/analytics/errorTracking';
 * ```
 * 
 * @module analytics
 */

// ============================================================================
// Analytics
// ============================================================================
/**
 * PostHog analytics integration.
 */
export * from './posthog';

// ============================================================================
// Error Tracking
// ============================================================================
/**
 * Error tracking and reporting utilities.
 */
export * from './errorTracking';

// ============================================================================
// Auto Tracking
// ============================================================================
/**
 * Automatic event tracking utilities.
 */
export * from './auto';
