/**
 * ArchitectureValidationService
 *
 * Domain service for validating architecture models, elements, relationships,
 * and business rules. This service encapsulates all validation logic that
 * doesn't naturally belong to a specific aggregate.
 *
 * @module domain/services
 */

import type {
  SrujaModelDump,
  Element,
  Relationship,
  ParsedView,
  Requirement,
  ADR,
  Policy,
  Scenario,
  Flow,
} from '@sruja/shared';
import {
  ValidationError,
  ConfigurationError,
  ok,
  err,
  type Result,
} from '@sruja/shared/utils';
import { ElementId } from '../value-objects/ElementId';
import { RelationshipKind } from '../value-objects/ElementRelationship';

/**
 * Validation severity levels
 */
export enum ValidationSeverity {
  /** Critical error that prevents model from being used */
  ERROR = 'error',
  /** Warning that should be addressed but doesn't block usage */
  WARNING = 'warning',
  /** Informational suggestion for improvement */
  INFO = 'info',
}

/**
 * Validation result with severity and location
 */
export interface ValidationIssue {
  /** The severity of this issue */
  severity: ValidationSeverity;
  /** Human-readable error message */
  message: string;
  /** The type of validation that failed */
  code: string;
  /** ID of the element/relationship this issue relates to (if applicable) */
  elementId?: string;
  /** Path to the specific field that has the issue */
  path?: string;
  /** Suggested fix (optional) */
  suggestion?: string;
}

/**
 * Validation report containing all issues found
 */
export interface ValidationReport {
  /** All validation issues found */
  issues: ValidationIssue[];
  /** Whether validation passed (no errors) */
  isValid: boolean;
  /** Whether there are any warnings */
  hasWarnings: boolean;
  /** Quality score (0-100, based on issues) */
  qualityScore: number;
}

/**
 * Validation rules configuration
 */
export interface ValidationRules {
  /** Whether to detect cycles in relationships */
  detectCycles: boolean;
  /** Whether to detect orphan elements */
  detectOrphans: boolean;
  /** Whether to validate required metadata fields */
  requireMetadata: boolean;
  /** Whether to validate all relationships have descriptions */
  requireDescriptions: boolean;
  /** Maximum allowed nesting depth for container relationships */
  maxNestingDepth: number;
  /** Whether to validate element names */
  validateElementNames: boolean;
  /** Minimum length for element names */
  minNameLength: number;
  /** Maximum length for element names */
  maxNameLength: number;
}

/**
 * Default validation rules
 */
export const DEFAULT_VALIDATION_RULES: ValidationRules = {
  detectCycles: true,
  detectOrphans: true,
  requireMetadata: false,
  requireDescriptions: true,
  maxNestingDepth: 10,
  validateElementNames: true,
  minNameLength: 2,
  maxNameLength: 100,
};

/**
 * ArchitectureValidationService
 *
 * Provides comprehensive validation for architecture models. This is a stateless
 * domain service that can be used to validate models at any point in their lifecycle.
 *
 * @example
 * ```typescript
 * const service = new ArchitectureValidationService();
 * const result = await service.validateModel(dump);
 * if (!result.isValid) {
 *   console.error('Validation failed:', result.issues);
 * }
 * ```
 */
export class ArchitectureValidationService {
  private rules: ValidationRules;

  /**
   * Creates a new validation service
   *
   * @param rules - Validation rules to use (defaults to DEFAULT_VALIDATION_RULES)
   */
  constructor(rules: Partial<ValidationRules> = {}) {
    this.rules = { ...DEFAULT_VALIDATION_RULES, ...rules };
  }

  /**
   * Validates an entire architecture model
   *
   * @param model - The model dump to validate
   * @returns A validation report with all issues found
   */
  validateModel(model: SrujaModelDump): ValidationReport {
    const issues: ValidationIssue[] = [];

    // Validate model structure
    issues.push(...this.validateModelStructure(model));

    // Validate elements
    if (model.elements) {
      for (const [id, element] of Object.entries(model.elements)) {
        issues.push(...this.validateElement(id, element));
      }
    }

    // Validate relationships
    if (model.relations) {
      for (let i = 0; i < model.relations.length; i++) {
        issues.push(...this.validateRelationship(i, model.relations[i], model.elements));
      }
    }

    // Validate views
    if (model.views) {
      for (const [viewName, view] of Object.entries(model.views)) {
        issues.push(...this.validateView(viewName, view, model.elements));
      }
    }

    // Validate specification
    if (model.specification) {
      issues.push(...this.validateSpecification(model.specification));
    }

    // Cross-element validation
    issues.push(...this.validateCrossElementRules(model));

    // Quality checks
    issues.push(...this.validateQuality(model));

    return this.createReport(issues);
  }

  /**
   * Validates a single element
   *
   * @param id - The element ID
   * @param element - The element to validate
   * @returns Array of validation issues
   */
  validateElement(id: string, element: Element): ValidationIssue[] {
    const issues: ValidationIssue[] = [];

    // Validate ID
    const idValidation = ElementId.create(id);
    if (!idValidation.ok) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: `Invalid element ID: ${idValidation.error.message}`,
        code: 'INVALID_ELEMENT_ID',
        elementId: id,
        path: `elements.${id}.id`,
      });
    }

    // Validate name
    if (this.rules.validateElementNames) {
      if (!element.name || element.name.trim().length === 0) {
        issues.push({
          severity: ValidationSeverity.ERROR,
          message: 'Element name cannot be empty',
          code: 'EMPTY_ELEMENT_NAME',
          elementId: id,
          path: `elements.${id}.name`,
        });
      } else {
        const nameLength = element.name.trim().length;
        if (nameLength < this.rules.minNameLength) {
          issues.push({
            severity: ValidationSeverity.WARNING,
            message: `Element name should be at least ${this.rules.minNameLength} characters`,
            code: 'SHORT_ELEMENT_NAME',
            elementId: id,
            path: `elements.${id}.name`,
            suggestion: `Consider a more descriptive name (current: ${nameLength} characters)`,
          });
        }
        if (nameLength > this.rules.maxNameLength) {
          issues.push({
            severity: ValidationSeverity.ERROR,
            message: `Element name cannot exceed ${this.rules.maxNameLength} characters`,
            code: 'LONG_ELEMENT_NAME',
            elementId: id,
            path: `elements.${id}.name`,
          });
        }
      }
    }

    // Validate kind
    if (!element.kind || element.kind.trim().length === 0) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: 'Element kind cannot be empty',
        code: 'EMPTY_ELEMENT_KIND',
        elementId: id,
        path: `elements.${id}.kind`,
      });
    }

    // Validate metadata
    if (this.rules.requireMetadata) {
      const requiredFields = ['description', 'technology'];
      for (const field of requiredFields) {
        if (!element[field as keyof Element]) {
          issues.push({
            severity: ValidationSeverity.WARNING,
            message: `Element is missing required metadata field: ${field}`,
            code: 'MISSING_METADATA',
            elementId: id,
            path: `elements.${id}.${field}`,
            suggestion: `Add a ${field} to improve documentation`,
          });
        }
      }
    }

    return issues;
  }

  /**
   * Validates a relationship between two elements
   *
   * @param index - The index of the relationship
   * @param relationship - The relationship to validate
   * @param elements - Map of all elements for reference validation
   * @returns Array of validation issues
   */
  validateRelationship(
    index: number,
    relationship: Relationship,
    elements?: Record<string, Element>
  ): ValidationIssue[] {
    const issues: ValidationIssue[] = [];
    const path = `relations[${index}]`;

    // Validate source element
    const sourceIdValidation = ElementId.create(relationship.source);
    if (!sourceIdValidation.ok) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: `Invalid source element ID: ${sourceIdValidation.error.message}`,
        code: 'INVALID_SOURCE_ID',
        path: `${path}.source`,
      });
    } else if (elements && !elements[relationship.source]) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: `Source element '${relationship.source}' does not exist`,
        code: 'SOURCE_NOT_FOUND',
        path: `${path}.source`,
      });
    }

    // Validate target element
    const targetIdValidation = ElementId.create(relationship.target);
    if (!targetIdValidation.ok) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: `Invalid target element ID: ${targetIdValidation.error.message}`,
        code: 'INVALID_TARGET_ID',
        path: `${path}.target`,
      });
    } else if (elements && !elements[relationship.target]) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: `Target element '${relationship.target}' does not exist`,
        code: 'TARGET_NOT_FOUND',
        path: `${path}.target`,
      });
    }

    // Validate description
    if (this.rules.requireDescriptions) {
      if (!relationship.description || relationship.description.trim().length === 0) {
        issues.push({
          severity: ValidationSeverity.WARNING,
          message: 'Relationship description is empty',
          code: 'EMPTY_RELATIONSHIP_DESCRIPTION',
          path: `${path}.description`,
          suggestion: 'Add a description to clarify the relationship',
        });
      }
    }

    // Check for self-relationships
    if (relationship.source === relationship.target) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: 'Relationship cannot connect an element to itself',
        code: 'SELF_RELATIONSHIP',
        path: `${path}`,
      });
    }

    // Validate relationship kind if specified
    if (relationship.kind) {
      const validKinds = Object.values(RelationshipKind);
      if (!validKinds.includes(relationship.kind as RelationshipKind)) {
        issues.push({
          severity: ValidationSeverity.WARNING,
          message: `Unknown relationship kind: '${relationship.kind}'`,
          code: 'UNKNOWN_RELATIONSHIP_KIND',
          path: `${path}.kind`,
          suggestion: `Use one of: ${validKinds.join(', ')}`,
        });
      }
    }

    return issues;
  }

  /**
   * Validates a view definition
   *
   * @param viewName - The name of the view
   * @param view - The view to validate
   * @param elements - Map of all elements for reference validation
   * @returns Array of validation issues
   */
  validateView(
    viewName: string,
    view: ParsedView,
    elements?: Record<string, Element>
  ): ValidationIssue[] {
    const issues: ValidationIssue[] = [];
    const path = `views.${viewName}`;

    // Validate view name
    if (!viewName || viewName.trim().length === 0) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: 'View name cannot be empty',
        code: 'EMPTY_VIEW_NAME',
        path: `views`,
      });
    }

    // Validate title
    if (!view.title || view.title.trim().length === 0) {
      issues.push({
        severity: ValidationSeverity.WARNING,
        message: 'View title is empty',
        code: 'EMPTY_VIEW_TITLE',
        path: `${path}.title`,
      });
    }

    // Validate element references
    if (elements && view.elements) {
      for (const elementRef of view.elements) {
        const id = typeof elementRef === 'string' ? elementRef : elementRef.id;
        if (!elements[id]) {
          issues.push({
            severity: ValidationSeverity.ERROR,
            message: `View references non-existent element: '${id}'`,
            code: 'VIEW_ELEMENT_NOT_FOUND',
            elementId: id,
            path: `${path}.elements`,
          });
        }
      }
    }

    // Validate relationship references
    if (elements && view.relationships) {
      for (const rel of view.relationships) {
        const sourceId = typeof rel.source === 'string' ? rel.source : rel.source.id;
        const targetId = typeof rel.target === 'string' ? rel.target : rel.target.id;

        if (!elements[sourceId]) {
          issues.push({
            severity: ValidationSeverity.ERROR,
            message: `View relationship references non-existent source element: '${sourceId}'`,
            code: 'VIEW_RELATIONSHIP_SOURCE_NOT_FOUND',
            elementId: sourceId,
            path: `${path}.relationships`,
          });
        }

        if (!elements[targetId]) {
          issues.push({
            severity: ValidationSeverity.ERROR,
            message: `View relationship references non-existent target element: '${targetId}'`,
            code: 'VIEW_RELATIONSHIP_TARGET_NOT_FOUND',
            elementId: targetId,
            path: `${path}.relationships`,
          });
        }
      }
    }

    return issues;
  }

  /**
   * Validates a specification
   *
   * @param specification - The specification to validate
   * @returns Array of validation issues
   */
  validateSpecification(specification: any): ValidationIssue[] {
    const issues: ValidationIssue[] = [];
    const path = 'specification';

    // Validate requirements
    if (specification.requirements) {
      for (let i = 0; i < specification.requirements.length; i++) {
        const req = specification.requirements[i] as Requirement;
        if (!req.id || req.id.trim().length === 0) {
          issues.push({
            severity: ValidationSeverity.ERROR,
            message: 'Requirement ID cannot be empty',
            code: 'EMPTY_REQUIREMENT_ID',
            path: `${path}.requirements[${i}].id`,
          });
        }
        if (!req.title || req.title.trim().length === 0) {
          issues.push({
            severity: ValidationSeverity.WARNING,
            message: 'Requirement title is empty',
            code: 'EMPTY_REQUIREMENT_TITLE',
            path: `${path}.requirements[${i}].title`,
          });
        }
      }
    }

    return issues;
  }

  /**
   * Validates cross-element business rules
   *
   * @param model - The model to validate
   * @returns Array of validation issues
   */
  private validateCrossElementRules(model: SrujaModelDump): ValidationIssue[] {
    const issues: ValidationIssue[] = [];

    if (!model.elements || !model.relations) {
      return issues;
    }

    // Detect cycles
    if (this.rules.detectCycles) {
      const cycles = this.detectCycles(model.elements, model.relations);
      cycles.forEach(cycle => {
        issues.push({
          severity: ValidationSeverity.ERROR,
          message: `Cyclic dependency detected: ${cycle.join(' → ')}`,
          code: 'CYCLIC_DEPENDENCY',
          suggestion: 'Break the cycle by removing or redirecting one relationship',
        });
      });
    }

    // Detect orphan elements
    if (this.rules.detectOrphans) {
      const orphans = this.detectOrphanElements(model.elements, model.relations);
      orphans.forEach(id => {
        issues.push({
          severity: ValidationSeverity.WARNING,
          message: `Element '${id}' has no relationships and may be isolated`,
          code: 'ORPHAN_ELEMENT',
          elementId: id,
          suggestion: 'Connect this element to the rest of the architecture',
        });
      });
    }

    // Validate nesting depth
    const maxDepth = this.validateNestingDepth(model.elements, model.relations);
    if (maxDepth > this.rules.maxNestingDepth) {
      issues.push({
        severity: ValidationSeverity.WARNING,
        message: `Maximum nesting depth (${maxDepth}) exceeds recommended limit (${this.rules.maxNestingDepth})`,
        code: 'EXCESSIVE_NESTING_DEPTH',
        suggestion: 'Consider flattening the architecture by reducing nested containers',
      });
    }

    // Detect duplicate relationships
    const duplicates = this.detectDuplicateRelationships(model.relations);
    duplicates.forEach(dup => {
      issues.push({
        severity: ValidationSeverity.WARNING,
        message: `Duplicate relationship detected between '${dup.source}' and '${dup.target}'`,
        code: 'DUPLICATE_RELATIONSHIP',
        suggestion: 'Remove duplicate relationships to avoid ambiguity',
      });
    });

    return issues;
  }

  /**
   * Validates model quality
   *
   * @param model - The model to validate
   * @returns Array of validation issues
   */
  private validateQuality(model: SrujaModelDump): ValidationIssue[] {
    const issues: ValidationIssue[] = [];

    if (!model.elements || !model.relations) {
      return issues;
    }

    const elementCount = Object.keys(model.elements).length;
    const relationshipCount = model.relations.length;

    // Check for reasonable relationship-to-element ratio
    if (elementCount > 0 && relationshipCount === 0) {
      issues.push({
        severity: ValidationSeverity.WARNING,
        message: 'Architecture has elements but no relationships',
        code: 'NO_RELATIONSHIPS',
        suggestion: 'Add relationships to show how elements interact',
      });
    }

    // Check for over-connected elements
    const connectionCounts = new Map<string, number>();
    for (const rel of model.relations) {
      connectionCounts.set(rel.source, (connectionCounts.get(rel.source) || 0) + 1);
      connectionCounts.set(rel.target, (connectionCounts.get(rel.target) || 0) + 1);
    }

    connectionCounts.forEach((count, id) => {
      if (count > 15) {
        issues.push({
          severity: ValidationSeverity.WARNING,
          message: `Element '${id}' has ${count} connections, which may indicate high coupling`,
          code: 'HIGH_COUPLING',
          elementId: id,
          suggestion: 'Consider refactoring to reduce coupling',
        });
      }
    });

    return issues;
  }

  /**
   * Validates basic model structure
   *
   * @param model - The model to validate
   * @returns Array of validation issues
   */
  private validateModelStructure(model: SrujaModelDump): ValidationIssue[] {
    const issues: ValidationIssue[] = [];

    if (!model.elements) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: 'Model is missing elements',
        code: 'MISSING_ELEMENTS',
        path: 'model.elements',
      });
    }

    if (!model.relations) {
      issues.push({
        severity: ValidationSeverity.ERROR,
        message: 'Model is missing relations',
        code: 'MISSING_RELATIONS',
        path: 'model.relations',
      });
    }

    if (!model.metadata) {
      issues.push({
        severity: ValidationSeverity.WARNING,
        message: 'Model is missing metadata',
        code: 'MISSING_METADATA',
        path: 'model.metadata',
      });
    } else if (!model.metadata.name || model.metadata.name.trim().length === 0) {
      issues.push({
        severity: ValidationSeverity.WARNING,
        message: 'Model metadata is missing name',
        code: 'MISSING_MODEL_NAME',
        path: 'model.metadata.name',
      });
    }

    return issues;
  }

  /**
   * Detects cycles in the relationship graph
   *
   * @private
   * @param elements - Map of elements
   * @param relations - Array of relationships
   * @returns Array of cycles found (each cycle is an array of element IDs)
   */
  private detectCycles(
    elements: Record<string, Element>,
    relations: Relationship[]
  ): string[][] {
    const cycles: string[][] = [];
    const visited = new Set<string>();
    const recursionStack = new Set<string>();

    // Build adjacency list
    const adj = new Map<string, string[]>();
    for (const id of Object.keys(elements)) {
      adj.set(id, []);
    }
    for (const rel of relations) {
      adj.get(rel.source)?.push(rel.target);
    }

    const dfs = (node: string, path: string[]): boolean => {
      visited.add(node);
      recursionStack.add(node);

      const neighbors = adj.get(node) || [];
      for (const neighbor of neighbors) {
        if (!visited.has(neighbor)) {
          if (dfs(neighbor, [...path, neighbor])) {
            return true;
          }
        } else if (recursionStack.has(neighbor)) {
          // Found a cycle
          const cycleStart = path.indexOf(neighbor);
          if (cycleStart !== -1) {
            cycles.push([...path.slice(cycleStart), neighbor]);
          }
        }
      }

      recursionStack.delete(node);
      return false;
    };

    for (const id of Object.keys(elements)) {
      if (!visited.has(id)) {
        dfs(id, [id]);
      }
    }

    return cycles;
  }

  /**
   * Detects orphan elements (no relationships)
   *
   * @private
   * @param elements - Map of elements
   * @param relations - Array of relationships
   * @returns Array of orphan element IDs
   */
  private detectOrphanElements(
    elements: Record<string, Element>,
    relations: Relationship[]
  ): string[] {
    const connected = new Set<string>();

    for (const rel of relations) {
      connected.add(rel.source);
      connected.add(rel.target);
    }

    return Object.keys(elements).filter(id => !connected.has(id));
  }

  /**
   * Validates nesting depth of container relationships
   *
   * @private
   * @param elements - Map of elements
   * @param relations - Array of relationships
   * @returns Maximum nesting depth found
   */
  private validateNestingDepth(
    elements: Record<string, Element>,
    relations: Relationship[]
  ): number {
    // Build parent-child relationships from "contains" relationships
    const children = new Map<string, string[]>();
    for (const id of Object.keys(elements)) {
      children.set(id, []);
    }
    for (const rel of relations) {
      if (rel.kind === RelationshipKind.CONTAINS) {
        children.get(rel.source)?.push(rel.target);
      }
    }

    // Calculate max depth using DFS
    const memo = new Map<string, number>();

    const getDepth = (id: string): number => {
      if (memo.has(id)) {
        return memo.get(id)!;
      }

      const childIds = children.get(id) || [];
      if (childIds.length === 0) {
        memo.set(id, 1);
        return 1;
      }

      const maxChildDepth = Math.max(...childIds.map(child => getDepth(child)));
      const depth = maxChildDepth + 1;
      memo.set(id, depth);
      return depth;
    };

    let maxDepth = 0;
    for (const id of Object.keys(elements)) {
      maxDepth = Math.max(maxDepth, getDepth(id));
    }

    return maxDepth;
  }

  /**
   * Detects duplicate relationships
   *
   * @private
   * @param relations - Array of relationships
   * @returns Array of duplicate relationships found
   */
  private detectDuplicateRelationships(relations: Relationship[]): Array<{
    source: string;
    target: string;
    count: number;
  }> {
    const keyCounts = new Map<string, { source: string; target: string; count: number }>();

    for (const rel of relations) {
      const key = `${rel.source}->${rel.target}`;
      const existing = keyCounts.get(key);
      if (existing) {
        existing.count++;
      } else {
        keyCounts.set(key, { source: rel.source, target: rel.target, count: 1 });
      }
    }

    return Array.from(keyCounts.values())
      .filter(entry => entry.count > 1)
      .map(entry => ({
        source: entry.source,
        target: entry.target,
        count: entry.count,
      }));
  }

  /**
   * Creates a validation report from issues
   *
   * @private
   * @param issues - Array of validation issues
   * @returns A validation report
   */
  private createReport(issues: ValidationIssue[]): ValidationReport {
    const errors = issues.filter(issue => issue.severity === ValidationSeverity.ERROR);
    const warnings = issues.filter(issue => issue.severity === ValidationSeverity.WARNING);

    // Calculate quality score (0-100)
    // Start at 100, subtract for errors (-10) and warnings (-2)
    let score = 100;
    score -= errors.length * 10;
    score -= warnings.length * 2;
    score = Math.max(0, score);

    return {
      issues,
      isValid: errors.length === 0,
      hasWarnings: warnings.length > 0,
      qualityScore: score,
    };
  }

  /**
   * Gets validation rules currently being used
   *
   * @returns The validation rules
   */
  getRules(): ValidationRules {
    return { ...this.rules };
  }

  /**
   * Updates validation rules
   *
   * @param rules - Partial rules to update
   */
  updateRules(rules: Partial<ValidationRules>): void {
    this.rules = { ...this.rules, ...rules };
  }
}
