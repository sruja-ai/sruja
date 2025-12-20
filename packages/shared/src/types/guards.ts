// packages/shared/src/types/guards.ts
// Type guards and validation utilities for architecture types

import { isNonEmptyString, isValidElementId, isValidPercentage } from "../utils/validation";
import type { SrujaModelDump } from "./core";
import type { ArchitectureJSON } from "./legacy";
import type { Requirement, ADR, Policy, SLO } from "./governance";

/**
 * Type guard to check if JSON is in new LikeC4 format (SrujaModelDump).
 * 
 * @public
 * @param json - The value to check
 * @returns `true` if the value is a valid SrujaModelDump structure
 * 
 * @remarks
 * Validates the structure more thoroughly by checking types of nested properties.
 * 
 * @example
 * if (isLikeC4Format(data)) { const elements = data.elements; }
 */
export function isLikeC4Format(json: unknown): json is SrujaModelDump {
  if (typeof json !== 'object' || json === null) {
    return false;
  }
  
  const obj = json as Record<string, unknown>;
  
  // Check required top-level properties
  if (!('specification' in obj) || !('elements' in obj)) {
    return false;
  }
  
  // Validate specification is an object
  if (typeof obj.specification !== 'object' || obj.specification === null) {
    return false;
  }
  
  // Validate elements is an object (Record<string, Element>)
  if (typeof obj.elements !== 'object' || obj.elements === null) {
    return false;
  }
  
  // _metadata is optional but if present should be an object
  if ('_metadata' in obj && (typeof obj._metadata !== 'object' || obj._metadata === null)) {
    return false;
  }
  
  return true;
}

/**
 * Type guard to check if JSON is in legacy format (ArchitectureJSON).
 * 
 * @public
 * @param json - The value to check
 * @returns `true` if the value is a valid ArchitectureJSON structure
 * 
 * @remarks
 * Validates the structure more thoroughly by checking types of nested properties.
 * 
 * @example
 * if (isLegacyFormat(data)) { const architecture = data.architecture; }
 */
export function isLegacyFormat(json: unknown): json is ArchitectureJSON {
  if (typeof json !== 'object' || json === null) {
    return false;
  }
  
  const obj = json as Record<string, unknown>;
  
  // Check required top-level properties
  if (!('metadata' in obj) || !('architecture' in obj)) {
    return false;
  }
  
  // Validate metadata is an object with required fields
  if (typeof obj.metadata !== 'object' || obj.metadata === null) {
    return false;
  }
  
  const metadata = obj.metadata as Record<string, unknown>;
  if (typeof metadata.name !== 'string' || typeof metadata.version !== 'string') {
    return false;
  }
  
  // Validate architecture is an object
  if (typeof obj.architecture !== 'object' || obj.architecture === null) {
    return false;
  }
  
  return true;
}

/**
 * Validates a Requirement object.
 * 
 * @public
 * @param requirement - The requirement to validate
 * @returns true if the requirement is valid
 * 
 * @example
 * if (isValidRequirement(req)) { processRequirement(req); }
 */
export function isValidRequirement(requirement: unknown): requirement is Requirement {
  if (typeof requirement !== 'object' || requirement === null) {
    return false;
  }
  const req = requirement as Partial<Requirement>;
  return (
    isValidElementId(req.id) &&
    isNonEmptyString(req.title)
  );
}

/**
 * Validates an ADR object.
 * 
 * @public
 * @param adr - The ADR to validate
 * @returns `true` if the ADR is valid
 */
export function isValidADR(adr: unknown): adr is ADR {
  if (typeof adr !== 'object' || adr === null) {
    return false;
  }
  const a = adr as Partial<ADR>;
  return (
    isValidElementId(a.id) &&
    isNonEmptyString(a.title)
  );
}

/**
 * Validates a Policy object.
 * 
 * @public
 * @param policy - The policy to validate
 * @returns `true` if the policy is valid
 */
export function isValidPolicy(policy: unknown): policy is Policy {
  if (typeof policy !== 'object' || policy === null) {
    return false;
  }
  const p = policy as Partial<Policy>;
  return (
    isValidElementId(p.id) &&
    isNonEmptyString(p.title)
  );
}

/**
 * Validates a SLO object.
 * 
 * @public
 * @param slo - The SLO to validate
 * @returns `true` if the SLO is valid
 */
export function isValidSLO(slo: unknown): slo is SLO {
  if (typeof slo !== 'object' || slo === null) {
    return false;
  }
  const s = slo as Partial<SLO>;
  return (
    isValidElementId(s.id) &&
    (s.target === undefined || isValidPercentage(s.target)) &&
    (s.availability === undefined || isValidPercentage(s.availability)) &&
    (s.errorBudget === undefined || isValidPercentage(s.errorBudget))
  );
}

