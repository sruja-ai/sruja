/**
 * Optimization module exports
 *
 * Provides the Agent-Driven Optimization Loop infrastructure:
 * - LayoutAuditor: Playwright-based visual validation
 * - MemoryBank: Storage for successful layouts (few-shot examples)
 */
export { LayoutAuditor, auditLayout } from "./LayoutAuditor";
export type { AuditResult, AuditOptions } from "./LayoutAuditor";

export { MemoryBank } from "./MemoryBank";
export type { SuccessfulLayout, MemoryBankOptions } from "./MemoryBank";
