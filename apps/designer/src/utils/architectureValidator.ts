/**
 * Architecture Validator
 * Client-side validation rules for real-time feedback in the Builder wizard
 */

import type { SrujaModelDump } from "@sruja/shared";

// ============================================================================
// Validation Types
// ============================================================================

export type ValidationSeverity = "error" | "warning" | "info";

export interface ValidationIssue {
  id: string;
  severity: ValidationSeverity;
  category: "orphan" | "duplicate" | "reference" | "missing" | "structure" | "c4" | "best-practice";
  elementId?: string;
  elementType?: string;
  message: string;
  suggestion?: string;
}

export interface ValidationResult {
  isValid: boolean;
  score: number; // 0-100
  issues: ValidationIssue[];
  summary: {
    errors: number;
    warnings: number;
    infos: number;
    bestPractices: number;
  };
}

// ============================================================================
// Validation Rules
// ============================================================================

/**
 * Detect best practice violations
 */
function checkBestPractices(model: SrujaModelDump): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const elements = Object.values(model.elements || {}) as any[];
  const systems = elements.filter((el) => el.kind === "system");
  const containers = elements.filter((el) => el.kind === "container");

  // BP001: Each System should have an overview/description
  systems.forEach((sys: any) => {
    // Description can be a string or an object with 'txt' property
    const description =
      typeof sys.description === "string" ? sys.description : sys.description?.txt || "";
    if (!description || description.trim().length < 10) {
      issues.push({
        id: `bp-overview-${sys.id}`,
        severity: "warning",
        category: "best-practice",
        elementId: sys.id,
        message: `System "${sys.title}" is missing a detailed description or overview`,
        suggestion: "Add an 'overview' block or a 'description' to provide context for reviewers.",
      });
    }

    // BP002: Metadata completeness (owner, status)
    // Sruja elements have metadata in system/container items in DSL,
    // but in Dump they might be in 'metadata' or 'properties'.
    const hasOwner = sys.metadata?.owner || sys.properties?.owner;
    const hasStatus = sys.metadata?.status || sys.properties?.status;

    if (!hasOwner) {
      issues.push({
        id: `bp-owner-${sys.id}`,
        severity: "info",
        category: "best-practice",
        elementId: sys.id,
        message: `System "${sys.title}" missing owner metadata`,
        suggestion: "Add 'owner' to the metadata block.",
      });
    }

    if (!hasStatus) {
      issues.push({
        id: `bp-status-${sys.id}`,
        severity: "info",
        category: "best-practice",
        elementId: sys.id,
        message: `System "${sys.title}" missing status (e.g., "production", "proposed")`,
        suggestion: "Add 'status' to the metadata block.",
      });
    }
  });

  // BP003: SLO for critical containers
  containers.forEach((container) => {
    // Check if container has SLO block
    // In Dump, SLO might be a field or in sruja extension
    const hasSlo = (container as any).slo || (model.sruja as any)?.slos?.[container.id];
    if (!hasSlo) {
      issues.push({
        id: `bp-slo-${container.id}`,
        severity: "info",
        category: "best-practice",
        elementId: container.id,
        message: `Container "${container.title}" has no SLO defined`,
        suggestion: "Define an 'slo' block (availability, latency) to specify reliability targets.",
      });
    }
  });

  // BP004: Process coverage (flows/scenarios)
  const flowCount = (model.sruja?.flows?.length || 0) + (model.sruja?.scenarios?.length || 0);
  if (flowCount === 0 && elements.length > 3) {
    issues.push({
      id: "bp-no-flows",
      severity: "warning",
      category: "best-practice",
      message: "Architecture lacks behavioral documentation (flows or scenarios)",
      suggestion:
        "Add a 'flow' or 'scenario' to document how components interact in key use cases.",
    });
  }

  // BP005: ADR coverage
  if ((model.sruja?.adrs?.length || 0) === 0 && elements.length > 5) {
    issues.push({
      id: "bp-no-adrs",
      severity: "info",
      category: "best-practice",
      message: "No Architectural Decision Records (ADR) found",
      suggestion: "Use 'adr' blocks to document the 'why' behind major technical choices.",
    });
  }

  return issues;
}

/**
 * Detect orphan elements (elements with no relations)
 */
function detectOrphanElements(model: SrujaModelDump): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const elements = Object.values(model.elements || {}) as any[];
  const connectedPaths = new Set<string>();

  // Collect all paths mentioned in relations
  (model.relations || []).forEach((rel: any) => {
    // Handle FqnRef structure: { model: string } or fallback to string
    const sourceRaw = rel.source || rel.from;
    const targetRaw = rel.target || rel.to;

    const source =
      sourceRaw && typeof sourceRaw === "object" && "model" in sourceRaw
        ? sourceRaw.model
        : typeof sourceRaw === "string"
          ? sourceRaw
          : null;

    const target =
      targetRaw && typeof targetRaw === "object" && "model" in targetRaw
        ? targetRaw.model
        : typeof targetRaw === "string"
          ? targetRaw
          : null;

    if (source && typeof source === "string") {
      connectedPaths.add(source);
    }
    if (target && typeof target === "string") {
      connectedPaths.add(target);
    }
  });

  // Check each element
  elements.forEach((el) => {
    // Skip if element is connected
    if (connectedPaths.has(el.id)) return;

    // Skip groups/systems that contain connected elements
    // In flat map, we check if any other element has this as parent
    const hasConnectedChildren = elements.some(
      (other) => other.id.startsWith(el.id + ".") && connectedPaths.has(other.id)
    );

    if (hasConnectedChildren) return;

    issues.push({
      id: `orphan-${el.id}`,
      severity: "warning",
      category: "orphan",
      elementId: el.id,
      message: `Element "${el.title}" (${el.kind}) has no relations`,
      suggestion:
        "Connect this element to others using '->' to show how it fits into the architecture.",
    });
  });

  return issues;
}

/**
 * Detect duplicate IDs at the same level
 * (elements map guarantees unique IDs)
 */
function detectDuplicateIds(_model: SrujaModelDump): ValidationIssue[] {
  return [];
}

/**
 * Validate relation references point to existing elements
 */
function validateRelationReferences(model: SrujaModelDump): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const validIds = new Set(Object.keys(model.elements || {}));

  (model.relations || []).forEach((rel: any, index) => {
    // Handle FqnRef structure: { model: string } or fallback to string
    // Also support legacy 'from'/'to' properties
    const sourceRaw = rel.source || rel.from;
    const targetRaw = rel.target || rel.to;

    const source =
      sourceRaw && typeof sourceRaw === "object" && "model" in sourceRaw
        ? sourceRaw.model
        : typeof sourceRaw === "string"
          ? sourceRaw
          : String(sourceRaw); // Fallback for edge cases

    const target =
      targetRaw && typeof targetRaw === "object" && "model" in targetRaw
        ? targetRaw.model
        : typeof targetRaw === "string"
          ? targetRaw
          : String(targetRaw); // Fallback for edge cases

    // Only validate if we have valid string IDs
    if (typeof source === "string" && !validIds.has(source)) {
      issues.push({
        id: `invalid-ref-source-${index}`,
        severity: "error",
        category: "reference",
        elementId: source,
        message: `Relation references unknown element "${source}"`,
        suggestion: `Check if "${source}" exists or if there is a typo in the ID.`,
      });
    }

    if (typeof target === "string" && !validIds.has(target)) {
      issues.push({
        id: `invalid-ref-target-${index}`,
        severity: "error",
        category: "reference",
        elementId: target,
        message: `Relation references unknown element "${target}"`,
        suggestion: `Check if "${target}" exists or if there is a typo in the ID.`,
      });
    }
  });

  return issues;
}

/**
 * Detect issues in governance elements (requirements, adrs, etc.)
 */
function validateGovernanceReferences(model: SrujaModelDump): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const tags = new Set(Object.keys(model.specification?.tags || {}));

  // Validate requirement tags
  (model.sruja?.requirements || []).forEach((req: any) => {
    (req.tags || []).forEach((tag: string) => {
      if (!tags.has(tag)) {
        issues.push({
          id: `req-tag-ref-${req.id}-${tag}`,
          severity: "warning",
          category: "reference",
          elementType: "requirement",
          elementId: req.id,
          message: `Requirement "${req.id}" references unknown tag "${tag}"`,
          suggestion: `Define tag "${tag}" in the specification block.`,
        });
      }
    });
  });

  // Validate ADR tags
  (model.sruja?.adrs || []).forEach((adr: any) => {
    (adr.tags || []).forEach((tag: string) => {
      if (!tags.has(tag)) {
        issues.push({
          id: `adr-tag-ref-${adr.id}-${tag}`,
          severity: "warning",
          category: "reference",
          elementType: "adr",
          elementId: adr.id,
          message: `ADR "${adr.id}" references unknown tag "${tag}"`,
          suggestion: `Define tag "${tag}" in the specification block.`,
        });
      }
    });
  });

  return issues;
}

/**
 * Main Validator
 */
export function validateArchitecture(model: SrujaModelDump | null): ValidationResult {
  if (!model || !model.elements) {
    return {
      isValid: false,
      score: 0,
      issues: [
        {
          id: "no-architecture",
          severity: "error",
          category: "structure",
          message: "No architecture loaded",
        },
      ],
      summary: { errors: 1, warnings: 0, infos: 0, bestPractices: 0 },
    };
  }

  const allIssues: ValidationIssue[] = [
    ...detectDuplicateIds(model),
    ...validateRelationReferences(model),
    ...detectOrphanElements(model),
    ...checkBestPractices(model),
    ...validateGovernanceReferences(model),
  ];

  // Add empty architecture warning if no elements
  if (Object.keys(model.elements).length === 0) {
    allIssues.push({
      id: "empty-architecture",
      severity: "warning",
      category: "structure",
      message: "Architecture has no elements",
      suggestion: "Start by adding a system or a persona.",
    });
  }

  const summary = {
    errors: allIssues.filter((i) => i.severity === "error").length,
    warnings: allIssues.filter((i) => i.severity === "warning").length,
    infos: allIssues.filter((i) => i.severity === "info").length,
    bestPractices: allIssues.filter((i) => i.category === "best-practice").length,
  };

  // Score calculation: Base 100, deduct based on severity
  // Errors: -15, Warnings: -10, Info/BestPractices: -2
  const deductions = summary.errors * 15 + summary.warnings * 10 + summary.infos * 2;
  const score = Math.max(0, Math.min(100, 100 - deductions));

  return {
    isValid: summary.errors === 0,
    score,
    issues: allIssues,
    summary,
  };
}

export function getIssuesForElement(
  result: ValidationResult,
  elementPath: string
): ValidationIssue[] {
  return result.issues.filter((i) => i.elementId === elementPath);
}

export function getIssuesByCategory(
  result: ValidationResult,
  category: ValidationIssue["category"]
): ValidationIssue[] {
  return result.issues.filter((i) => i.category === category);
}

export function getSeverityColor(severity: ValidationSeverity): string {
  switch (severity) {
    case "error":
      return "text-red-500";
    case "warning":
      return "text-yellow-500";
    case "info":
      return "text-blue-500";
  }
}

export function getSeverityIcon(severity: ValidationSeverity): string {
  switch (severity) {
    case "error":
      return "AlertCircle";
    case "warning":
      return "AlertTriangle";
    case "info":
      return "Info";
  }
}
