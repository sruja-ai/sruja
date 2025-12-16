/**
 * Architecture Validator
 * Client-side validation rules for real-time feedback in the Builder wizard
 */

import type { ArchitectureJSON, RelationJSON } from "../types";

// ============================================================================
// Validation Types
// ============================================================================

export type ValidationSeverity = "error" | "warning" | "info";

export interface ValidationIssue {
  id: string;
  severity: ValidationSeverity;
  category: "orphan" | "duplicate" | "reference" | "missing" | "structure";
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
  };
}

// ============================================================================
// Element Collection Helpers
// ============================================================================

interface ElementInfo {
  id: string;
  fullPath: string;
  type: "person" | "system" | "container" | "datastore" | "queue" | "component";
  label?: string;
  parentPath?: string;
}

/**
 * Collect all elements from the architecture with their full paths
 */
function collectAllElements(arch: ArchitectureJSON): ElementInfo[] {
  const elements: ElementInfo[] = [];

  // Persons
  (arch.architecture?.persons ?? []).forEach((p) => {
    elements.push({
      id: p.id,
      fullPath: p.id,
      type: "person",
      label: p.label,
    });
  });

  // Systems and their children
  (arch.architecture?.systems ?? []).forEach((system) => {
    elements.push({
      id: system.id,
      fullPath: system.id,
      type: "system",
      label: system.label,
    });

    // Containers
    (system.containers ?? []).forEach((container) => {
      const containerPath = `${system.id}.${container.id}`;
      elements.push({
        id: container.id,
        fullPath: containerPath,
        type: "container",
        label: container.label,
        parentPath: system.id,
      });

      // Components
      (container.components ?? []).forEach((component) => {
        elements.push({
          id: component.id,
          fullPath: `${containerPath}.${component.id}`,
          type: "component",
          label: component.label,
          parentPath: containerPath,
        });
      });
    });

    // Datastores
    (system.datastores ?? []).forEach((ds) => {
      elements.push({
        id: ds.id,
        fullPath: `${system.id}.${ds.id}`,
        type: "datastore",
        label: ds.label,
        parentPath: system.id,
      });
    });

    // Queues
    (system.queues ?? []).forEach((q) => {
      elements.push({
        id: q.id,
        fullPath: `${system.id}.${q.id}`,
        type: "queue",
        label: q.label,
        parentPath: system.id,
      });
    });
  });

  return elements;
}

/**
 * Get all valid element paths for reference checking
 */
function getValidPaths(elements: ElementInfo[]): Set<string> {
  return new Set(elements.map((e) => e.fullPath));
}

// ============================================================================
// Validation Rules
// ============================================================================

/**
 * Detect orphan elements (elements with no relations)
 */
function detectOrphanElements(
  elements: ElementInfo[],
  relations: RelationJSON[]
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const connectedPaths = new Set<string>();

  // Collect all paths mentioned in relations
  relations.forEach((rel) => {
    connectedPaths.add(rel.from);
    connectedPaths.add(rel.to);

    // Also mark parent paths as connected (implied relationships)
    const fromParts = rel.from.split(".");
    const toParts = rel.to.split(".");

    for (let i = 1; i < fromParts.length; i++) {
      connectedPaths.add(fromParts.slice(0, i).join("."));
    }
    for (let i = 1; i < toParts.length; i++) {
      connectedPaths.add(toParts.slice(0, i).join("."));
    }
  });

  // Check each element
  elements.forEach((el) => {
    // Skip if element is connected
    if (connectedPaths.has(el.fullPath)) return;

    // Skip if any child is connected (parent is implicitly connected)
    const hasConnectedChild = elements.some(
      (child) => child.parentPath === el.fullPath && connectedPaths.has(child.fullPath)
    );
    if (hasConnectedChild) return;

    // Skip persons if there are no systems (early stage)
    if (el.type === "person" && elements.filter((e) => e.type === "system").length === 0) {
      return;
    }

    issues.push({
      id: `orphan-${el.fullPath}`,
      severity: "warning",
      category: "orphan",
      elementId: el.fullPath,
      elementType: el.type,
      message: `${el.type} "${el.label || el.id}" has no relations`,
      suggestion: `Add a relation to or from ${el.fullPath}`,
    });
  });

  return issues;
}

/**
 * Detect duplicate IDs at the same level
 */
function detectDuplicateIds(elements: ElementInfo[]): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const seenPaths = new Map<string, ElementInfo>();

  elements.forEach((el) => {
    if (seenPaths.has(el.fullPath)) {
      const existing = seenPaths.get(el.fullPath)!;
      issues.push({
        id: `duplicate-${el.fullPath}`,
        severity: "error",
        category: "duplicate",
        elementId: el.fullPath,
        elementType: el.type,
        message: `Duplicate ID "${el.id}" - conflicts with ${existing.type}`,
        suggestion: `Rename one of the "${el.id}" elements`,
      });
    } else {
      seenPaths.set(el.fullPath, el);
    }
  });

  return issues;
}

/**
 * Validate relation references point to existing elements
 */
function validateRelationReferences(
  relations: RelationJSON[],
  validPaths: Set<string>
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  relations.forEach((rel, index) => {
    if (!validPaths.has(rel.from)) {
      issues.push({
        id: `invalid-ref-from-${index}`,
        severity: "error",
        category: "reference",
        elementId: rel.from,
        message: `Relation references unknown element "${rel.from}"`,
        suggestion: `Check if "${rel.from}" exists or fix the path`,
      });
    }

    if (!validPaths.has(rel.to)) {
      issues.push({
        id: `invalid-ref-to-${index}`,
        severity: "error",
        category: "reference",
        elementId: rel.to,
        message: `Relation references unknown element "${rel.to}"`,
        suggestion: `Check if "${rel.to}" exists or fix the path`,
      });
    }
  });

  return issues;
}

/**
 * Validate tag references in requirements and ADRs
 */
function validateTagReferences(arch: ArchitectureJSON, validPaths: Set<string>): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  // Check requirement tags
  (arch.architecture?.requirements ?? []).forEach((req) => {
    (req.tags ?? []).forEach((tag) => {
      if (!validPaths.has(tag)) {
        issues.push({
          id: `invalid-tag-req-${req.id}-${tag}`,
          severity: "error",
          category: "reference",
          elementId: req.id,
          elementType: "requirement",
          message: `Requirement "${req.id}" references unknown element "${tag}"`,
          suggestion: `Remove the tag or create element "${tag}"`,
        });
      }
    });
  });

  // Check ADR tags
  (arch.architecture?.adrs ?? []).forEach((adr) => {
    (adr.tags ?? []).forEach((tag) => {
      if (!validPaths.has(tag)) {
        issues.push({
          id: `invalid-tag-adr-${adr.id}-${tag}`,
          severity: "error",
          category: "reference",
          elementId: adr.id,
          elementType: "adr",
          message: `ADR "${adr.id}" references unknown element "${tag}"`,
          suggestion: `Remove the tag or create element "${tag}"`,
        });
      }
    });
  });

  return issues;
}

/**
 * Check for missing labels (info level)
 */
function checkMissingLabels(elements: ElementInfo[]): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  elements.forEach((el) => {
    // Only warn for systems and containers (important for clarity)
    if ((el.type === "system" || el.type === "container") && !el.label) {
      issues.push({
        id: `missing-label-${el.fullPath}`,
        severity: "info",
        category: "missing",
        elementId: el.fullPath,
        elementType: el.type,
        message: `${el.type} "${el.id}" has no label`,
        suggestion: `Add a descriptive label for better readability`,
      });
    }
  });

  return issues;
}

/**
 * Check structural issues
 */
function checkStructuralIssues(arch: ArchitectureJSON): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  // Check for empty architecture
  const systems = arch.architecture?.systems ?? [];
  const persons = arch.architecture?.persons ?? [];

  if (systems.length === 0 && persons.length === 0) {
    issues.push({
      id: "empty-architecture",
      severity: "warning",
      category: "structure",
      message: "Architecture has no systems or persons defined",
      suggestion: "Add at least one system to get started",
    });
  }

  // Check for systems without containers (L2 opportunity)
  systems.forEach((system) => {
    const hasChildren =
      (system.containers?.length ?? 0) > 0 ||
      (system.datastores?.length ?? 0) > 0 ||
      (system.queues?.length ?? 0) > 0;

    if (!hasChildren) {
      issues.push({
        id: `no-containers-${system.id}`,
        severity: "info",
        category: "structure",
        elementId: system.id,
        elementType: "system",
        message: `System "${system.label || system.id}" has no containers`,
        suggestion: "Consider adding containers for L2 detail",
      });
    }
  });

  return issues;
}

// ============================================================================
// Main Validator
// ============================================================================

/**
 * Validate an architecture and return all issues
 */
export function validateArchitecture(arch: ArchitectureJSON | null): ValidationResult {
  if (!arch || !arch.architecture) {
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
      summary: { errors: 1, warnings: 0, infos: 0 },
    };
  }

  const elements = collectAllElements(arch);
  const validPaths = getValidPaths(elements);
  const relations = arch.architecture.relations ?? [];

  // Collect all issues
  const allIssues: ValidationIssue[] = [
    ...detectDuplicateIds(elements),
    ...validateRelationReferences(relations, validPaths),
    ...validateTagReferences(arch, validPaths),
    ...detectOrphanElements(elements, relations),
    ...checkMissingLabels(elements),
    ...checkStructuralIssues(arch),
  ];

  // Calculate summary
  const summary = {
    errors: allIssues.filter((i) => i.severity === "error").length,
    warnings: allIssues.filter((i) => i.severity === "warning").length,
    infos: allIssues.filter((i) => i.severity === "info").length,
  };

  // Calculate score (100 - deductions)
  // Errors: -10 each, Warnings: -5 each, Infos: -1 each
  const deductions = summary.errors * 10 + summary.warnings * 5 + summary.infos * 1;
  const score = Math.max(0, Math.min(100, 100 - deductions));

  return {
    isValid: summary.errors === 0,
    score,
    issues: allIssues,
    summary,
  };
}

/**
 * Get issues for a specific element
 */
export function getIssuesForElement(
  result: ValidationResult,
  elementPath: string
): ValidationIssue[] {
  return result.issues.filter((i) => i.elementId === elementPath);
}

/**
 * Get issues by category
 */
export function getIssuesByCategory(
  result: ValidationResult,
  category: ValidationIssue["category"]
): ValidationIssue[] {
  return result.issues.filter((i) => i.category === category);
}

/**
 * Get color class for severity
 */
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

/**
 * Get icon name for severity
 */
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
