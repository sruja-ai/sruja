/**
 * Architecture Use Cases
 *
 * Application-level use cases for architecture operations. These use cases
 * orchestrate domain operations and provide a clean API for the UI layer.
 *
 * @module application/use-cases
 */

import type {
  SrujaModelDump,
  Element,
  Relationship,
  ParsedView,
} from '@sruja/shared';
import { ok, err, type Result } from '@sruja/shared/utils/result';
import { ValidationError, ConfigurationError } from '@sruja/shared/utils/errors';
import { ArchitectureAggregate } from '../../domain/aggregates/ArchitectureAggregate';
import { ArchitectureValidationService, ValidationReport } from '../../domain/services/ArchitectureValidationService';
import type { ElementId } from '../../domain/value-objects/ElementId';
import { ElementRelationship, RelationshipKind } from '../../domain/value-objects/ElementRelationship';

/**
 * Result of element creation/update operation
 */
export interface ElementOperationResult {
  /** The updated architecture aggregate */
  aggregate: ArchitectureAggregate;
  /** The element that was operated on */
  element: Element;
}

/**
 * Result of relationship creation/update operation
 */
export interface RelationshipOperationResult {
  /** The updated architecture aggregate */
  aggregate: ArchitectureAggregate;
  /** The relationship that was operated on */
  relationship: Relationship;
}

/**
 * Options for searching elements
 */
export interface SearchOptions {
  /** Filter by element kind */
  kind?: string;
  /** Filter by tag */
  tag?: string;
  /** Search in name and description */
  searchTerm?: string;
  /** Maximum number of results */
  limit?: number;
}

/**
 * Statistics about the architecture
 */
export interface ArchitectureStats {
  /** Total number of elements */
  totalElements: number;
  /** Total number of relationships */
  totalRelationships: number;
  /** Number of elements by kind */
  elementsByKind: Record<string, number>;
  /** Number of relationships by kind */
  relationshipsByKind: Record<string, number>;
  /** Number of orphan elements */
  orphanCount: number;
  /** Maximum nesting depth */
  maxNestingDepth: number;
}

/**
 * Architecture use cases
 *
 * Orchestrates domain operations for architecture management. This class
 * provides a high-level API for the application to interact with the domain.
 *
 * @example
 * ```typescript
 * const useCases = new ArchitectureUseCases();
 *
 * // Load a model
 * const result = await useCases.loadModel(dump);
 * if (!result.ok) {
 *   console.error(result.error);
 *   return;
 * }
 *
 * // Create an element
 * const createResult = useCases.createElement(result.value, {
 *   id: 'web-server',
 *   name: 'Web Server',
 *   kind: 'container',
 *   description: 'Main web server',
 * });
 *
 * // Validate
 * const report = useCases.validate(createResult.value.aggregate);
 * ```
 */
export class ArchitectureUseCases {
  private validationService: ArchitectureValidationService;

  /**
   * Creates a new ArchitectureUseCases instance
   *
   * @param validationService - Optional validation service (defaults to new instance)
   */
  constructor(validationService?: ArchitectureValidationService) {
    this.validationService = validationService ?? new ArchitectureValidationService();
  }

  // =========================================================================
  // Model Operations
  // =========================================================================

  /**
   * Loads a model from a dump
   *
   * @param dump - The model dump to load
   * @returns Result containing the architecture aggregate or error
   */
  loadModel(dump: SrujaModelDump): Result<ArchitectureAggregate, ValidationError> {
    return ArchitectureAggregate.fromDump(dump);
  }

  /**
   * Creates a new empty architecture model
   *
   * @param name - The name of the architecture
   * @returns Result containing the architecture aggregate or error
   */
  createEmptyModel(name: string = 'Untitled'): Result<ArchitectureAggregate, ValidationError> {
    return ArchitectureAggregate.createEmpty(name);
  }

  /**
   * Saves the current architecture model to a dump format
   *
   * @param aggregate - The architecture aggregate to save
   * @returns The model dump
   */
  saveModel(aggregate: ArchitectureAggregate): SrujaModelDump {
    return aggregate.toDump();
  }

  /**
   * Validates the architecture model
   *
   * @param aggregate - The architecture aggregate to validate
   * @returns A validation report
   */
  validate(aggregate: ArchitectureAggregate): ValidationReport {
    return this.validationService.validateModel(aggregate.toDump());
  }

  /**
   * Gets statistics about the architecture
   *
   * @param aggregate - The architecture aggregate to analyze
   * @returns Architecture statistics
   */
  getStats(aggregate: ArchitectureAggregate): ArchitectureStats {
    const elements = aggregate.getAllElements();
    const relationships = aggregate.getRelationships();

    // Count elements by kind
    const elementsByKind: Record<string, number> = {};
    for (const el of elements) {
      elementsByKind[el.kind] = (elementsByKind[el.kind] || 0) + 1;
    }

    // Count relationships by kind
    const relationshipsByKind: Record<string, number> = {};
    for (const rel of relationships) {
      const kind = rel.kind || 'unspecified';
      relationshipsByKind[kind] = (relationshipsByKind[kind] || 0) + 1;
    }

    // Count orphans
    const orphans = aggregate.findOrphanElements();

    return {
      totalElements: elements.length,
      totalRelationships: relationships.length,
      elementsByKind,
      relationshipsByKind,
      orphanCount: orphans.length,
      maxNestingDepth: this.calculateMaxDepth(aggregate),
    };
  }

  // =========================================================================
  // Element Operations
  // =========================================================================

  /**
   * Creates a new element in the architecture
   *
   * @param aggregate - The current architecture aggregate
   * @param element - The element to create
   * @returns Result containing updated aggregate and element or error
   */
  createElement(
    aggregate: ArchitectureAggregate,
    element: Element
  ): Result<ElementOperationResult, ValidationError> {
    const result = aggregate.addElement(element);
    if (!result.ok) {
      return err(result.error);
    }

    return ok({
      aggregate: result.value,
      element,
    });
  }

  /**
   * Updates an existing element
   *
   * @param aggregate - The current architecture aggregate
   * @param elementId - The ID of the element to update
   * @param updates - Partial updates to apply
   * @returns Result containing updated aggregate and element or error
   */
  updateElement(
    aggregate: ArchitectureAggregate,
    elementId: string,
    updates: Partial<Element>
  ): Result<ElementOperationResult, ValidationError> {
    const result = aggregate.updateElement(elementId, updates);
    if (!result.ok) {
      return err(result.error);
    }

    const updatedElement = result.value.getElement(elementId);
    if (!updatedElement) {
      return err(new ValidationError(`Element '${elementId}' not found after update`));
    }

    return ok({
      aggregate: result.value,
      element: updatedElement,
    });
  }

  /**
   * Removes an element from the architecture
   *
   * @param aggregate - The current architecture aggregate
   * @param elementId - The ID of the element to remove
   * @returns Result containing updated aggregate or error
   */
  removeElement(
    aggregate: ArchitectureAggregate,
    elementId: string
  ): Result<ArchitectureAggregate, ValidationError> {
    return aggregate.removeElement(elementId);
  }

  /**
   * Gets an element by ID
   *
   * @param aggregate - The architecture aggregate
   * @param elementId - The ID of the element to get
   * @returns The element or undefined
   */
  getElement(aggregate: ArchitectureAggregate, elementId: string): Element | undefined {
    return aggregate.getElement(elementId);
  }

  /**
   * Gets all elements
   *
   * @param aggregate - The architecture aggregate
   * @returns Array of all elements
   */
  getAllElements(aggregate: ArchitectureAggregate): Element[] {
    return aggregate.getAllElements();
  }

  /**
   * Searches for elements based on criteria
   *
   * @param aggregate - The architecture aggregate
   * @param options - Search options
   * @returns Array of matching elements
   */
  searchElements(aggregate: ArchitectureAggregate, options: SearchOptions): Element[] {
    let results = aggregate.getAllElements();

    // Filter by kind
    if (options.kind) {
      results = results.filter(el => el.kind === options.kind);
    }

    // Filter by tag
    if (options.tag) {
      results = results.filter(el => el.tags?.includes(options.tag!));
    }

    // Search by term
    if (options.searchTerm) {
      const term = options.searchTerm.toLowerCase();
      results = results.filter(
        el =>
          el.name.toLowerCase().includes(term) ||
          el.description?.toLowerCase().includes(term)
      );
    }

    // Apply limit
    if (options.limit && options.limit > 0) {
      results = results.slice(0, options.limit);
    }

    return results;
  }

  // =========================================================================
  // Relationship Operations
  // =========================================================================

  /**
   * Creates a new relationship between elements
   *
   * @param aggregate - The current architecture aggregate
   * @param sourceId - Source element ID
   * @param targetId - Target element ID
   * @param description - Relationship description
   * @param options - Additional relationship options
   * @returns Result containing updated aggregate and relationship or error
   */
  createRelationship(
    aggregate: ArchitectureAggregate,
    sourceId: string,
    targetId: string,
    description: string,
    options?: {
      kind?: RelationshipKind | string;
      technology?: string;
      direction?: 'forward' | 'backward' | 'bidirectional' | 'none';
      metadata?: Record<string, unknown>;
    }
  ): Result<RelationshipOperationResult, ValidationError> {
    const result = aggregate.addRelationship(sourceId, targetId, {
      description,
      kind: options?.kind,
      technology: options?.technology,
      ...(options?.metadata && { metadata: options.metadata }),
    });

    if (!result.ok) {
      return err(result.error);
    }

    // Get the created relationship
    const relationships = result.value.getRelationships();
    const relationship = relationships.find(
      rel => rel.source === sourceId && rel.target === targetId
    );

    if (!relationship) {
      return err(new ValidationError('Failed to retrieve created relationship'));
    }

    return ok({
      aggregate: result.value,
      relationship,
    });
  }

  /**
   * Updates an existing relationship
   *
   * @param aggregate - The current architecture aggregate
   * @param sourceId - Source element ID
   * @param targetId - Target element ID
   * @param updates - Partial updates to apply
   * @returns Result containing updated aggregate and relationship or error
   */
  updateRelationship(
    aggregate: ArchitectureAggregate,
    sourceId: string,
    targetId: string,
    updates: Partial<Relationship>
  ): Result<RelationshipOperationResult, ValidationError> {
    const result = aggregate.updateRelationship(sourceId, targetId, updates);
    if (!result.ok) {
      return err(result.error);
    }

    // Get the updated relationship
    const relationships = result.value.getRelationships();
    const relationship = relationships.find(
      rel => rel.source === sourceId && rel.target === targetId
    );

    if (!relationship) {
      return err(new ValidationError('Failed to retrieve updated relationship'));
    }

    return ok({
      aggregate: result.value,
      relationship,
    });
  }

  /**
   * Removes a relationship between elements
   *
   * @param aggregate - The current architecture aggregate
   * @param sourceId - Source element ID
   * @param targetId - Target element ID
   * @returns Result containing updated aggregate or error
   */
  removeRelationship(
    aggregate: ArchitectureAggregate,
    sourceId: string,
    targetId: string
  ): Result<ArchitectureAggregate, ValidationError> {
    return aggregate.removeRelationship(sourceId, targetId);
  }

  /**
   * Gets all relationships
   *
   * @param aggregate - The architecture aggregate
   * @returns Array of all relationships
   */
  getAllRelationships(aggregate: ArchitectureAggregate): Relationship[] {
    return aggregate.getRelationships();
  }

  /**
   * Gets relationships for a specific element
   *
   * @param aggregate - The architecture aggregate
   * @param elementId - The element ID
   * @returns Array of relationships for the element
   */
  getRelationshipsForElement(aggregate: ArchitectureAggregate, elementId: string): Relationship[] {
    return aggregate.getRelationshipsForElement(elementId);
  }

  /**
   * Gets upstream relationships (where element is target)
   *
   * @param aggregate - The architecture aggregate
   * @param elementId - The element ID
   * @returns Array of upstream relationships
   */
  getUpstreamRelationships(aggregate: ArchitectureAggregate, elementId: string): Relationship[] {
    return aggregate.getUpstreamRelationships(elementId);
  }

  /**
   * Gets downstream relationships (where element is source)
   *
   * @param aggregate - The architecture aggregate
   * @param elementId - The element ID
   * @returns Array of downstream relationships
   */
  getDownstreamRelationships(aggregate: ArchitectureAggregate, elementId: string): Relationship[] {
    return aggregate.getDownstreamRelationships(elementId);
  }

  // =========================================================================
  // View Operations
  // =========================================================================

  /**
   * Gets a view by name
   *
   * @param aggregate - The architecture aggregate
   * @param viewName - The name of the view
   * @returns The view or undefined
   */
  getView(aggregate: ArchitectureAggregate, viewName: string): ParsedView | undefined {
    return aggregate.getView(viewName);
  }

  /**
   * Gets all views
   *
   * @param aggregate - The architecture aggregate
   * @returns Object mapping view names to views
   */
  getAllViews(aggregate: ArchitectureAggregate): Record<string, ParsedView> {
    return aggregate.getAllViews();
  }

  /**
   * Sets or updates a view
   *
   * @param aggregate - The current architecture aggregate
   * @param viewName - The name of the view
   * @param view - The view definition
   * @returns Result containing updated aggregate or error
   */
  setView(
    aggregate: ArchitectureAggregate,
    viewName: string,
    view: ParsedView
  ): Result<ArchitectureAggregate, ValidationError> {
    return aggregate.setView(viewName, view);
  }

  /**
   * Removes a view
   *
   * @param aggregate - The current architecture aggregate
   * @param viewName - The name of the view to remove
   * @returns Result containing updated aggregate or error
   */
  removeView(
    aggregate: ArchitectureAggregate,
    viewName: string
  ): Result<ArchitectureAggregate, ValidationError> {
    return aggregate.removeView(viewName);
  }

  // =========================================================================
  // Batch Operations
  // =========================================================================

  /**
   * Duplicates a set of elements and their relationships
   *
   * @param aggregate - The current architecture aggregate
   * @param elementIds - Array of element IDs to duplicate
   * @param idPrefix - Prefix to add to new element IDs
   * @returns Result containing updated aggregate with duplicated elements
   */
  duplicateElements(
    aggregate: ArchitectureAggregate,
    elementIds: string[],
    idPrefix: string = 'copy-'
  ): Result<ArchitectureAggregate, ValidationError> {
    let currentAggregate = aggregate;
    const idMap = new Map<string, string>();

    // First pass: duplicate elements
    for (const elementId of elementIds) {
      const element = currentAggregate.getElement(elementId);
      if (!element) {
        continue;
      }

      const newId = `${idPrefix}${elementId}`;
      idMap.set(elementId, newId);

      const newElement: Element = {
        ...element,
        id: newId,
        name: `${element.name} (Copy)`,
      };

      const result = currentAggregate.addElement(newElement);
      if (!result.ok) {
        return err(result.error);
      }
      currentAggregate = result.value;
    }

    // Second pass: duplicate relationships between selected elements
    const allRelationships = currentAggregate.getRelationships();
    for (const rel of allRelationships) {
      const isSourceSelected = idMap.has(rel.source);
      const isTargetSelected = idMap.has(rel.target);

      // Only duplicate relationships where both ends are selected
      if (isSourceSelected && isTargetSelected) {
        const newSourceId = idMap.get(rel.source)!;
        const newTargetId = idMap.get(rel.target)!;

        const result = currentAggregate.addRelationship(newSourceId, newTargetId, {
          description: rel.description,
          kind: rel.kind,
          technology: rel.technology,
        });

        if (!result.ok) {
          return err(result.error);
        }
        currentAggregate = result.value;
      }
    }

    return ok(currentAggregate);
  }

  /**
   * Removes multiple elements and their relationships
   *
   * @param aggregate - The current architecture aggregate
   * @param elementIds - Array of element IDs to remove
   * @returns Result containing updated aggregate or error
   */
  removeMultipleElements(
    aggregate: ArchitectureAggregate,
    elementIds: string[]
  ): Result<ArchitectureAggregate, ValidationError> {
    let currentAggregate = aggregate;

    for (const elementId of elementIds) {
      const result = currentAggregate.removeElement(elementId);
      if (!result.ok) {
        // Log error but continue with other elements
        console.warn(`Failed to remove element '${elementId}': ${result.error.message}`);
      } else {
        currentAggregate = result.value;
      }
    }

    return ok(currentAggregate);
  }

  // =========================================================================
  // Analysis Operations
  // =========================================================================

  /**
   * Finds the shortest path between two elements
   *
   * @param aggregate - The architecture aggregate
   * @param fromId - Source element ID
   * @param toId - Target element ID
   * @returns Array of element IDs representing the path, or empty if no path exists
   */
  findShortestPath(
    aggregate: ArchitectureAggregate,
    fromId: string,
    toId: string
  ): string[] {
    const elements = aggregate.getAllElements();
    const relationships = aggregate.getRelationships();

    // Build adjacency list
    const adj = new Map<string, string[]>();
    for (const el of elements) {
      adj.set(el.id, []);
    }
    for (const rel of relationships) {
      adj.get(rel.source)?.push(rel.target);
    }

    // BFS to find shortest path
    const visited = new Set<string>();
    const parent = new Map<string, string | null>();
    const queue: string[] = [fromId];
    visited.add(fromId);
    parent.set(fromId, null);

    while (queue.length > 0) {
      const current = queue.shift()!;
      if (current === toId) {
        // Reconstruct path
        const path: string[] = [];
        let node: string | null = toId;
        while (node !== null) {
          path.unshift(node);
          node = parent.get(node)!;
        }
        return path;
      }

      for (const neighbor of adj.get(current) || []) {
        if (!visited.has(neighbor)) {
          visited.add(neighbor);
          parent.set(neighbor, current);
          queue.push(neighbor);
        }
      }
    }

    return []; // No path found
  }

  /**
   * Finds all elements that depend on a given element (transitive downstream)
   *
   * @param aggregate - The architecture aggregate
   * @param elementId - The element ID to find dependents for
   * @returns Array of dependent element IDs
   */
  findDependents(aggregate: ArchitectureAggregate, elementId: string): string[] {
    const dependents: string[] = [];
    const visited = new Set<string>();

    const dfs = (id: string) => {
      const downstream = aggregate.getDownstreamRelationships(id);
      for (const rel of downstream) {
        if (!visited.has(rel.target)) {
          visited.add(rel.target);
          dependents.push(rel.target);
          dfs(rel.target);
        }
      }
    };

    dfs(elementId);
    return dependents;
  }

  /**
   * Finds all elements that a given element depends on (transitive upstream)
   *
   * @param aggregate - The architecture aggregate
   * @param elementId - The element ID to find dependencies for
   * @returns Array of dependency element IDs
   */
  findDependencies(aggregate: ArchitectureAggregate, elementId: string): string[] {
    const dependencies: string[] = [];
    const visited = new Set<string>();

    const dfs = (id: string) => {
      const upstream = aggregate.getUpstreamRelationships(id);
      for (const rel of upstream) {
        if (!visited.has(rel.source)) {
          visited.add(rel.source);
          dependencies.push(rel.source);
          dfs(rel.source);
        }
      }
    };

    dfs(elementId);
    return dependencies;
  }

  /**
   * Calculates the maximum nesting depth of the architecture
   *
   * @private
   * @param aggregate - The architecture aggregate
   * @returns Maximum nesting depth
   */
  private calculateMaxDepth(aggregate: ArchitectureAggregate): number {
    const elements = aggregate.getAllElements();
    const relationships = aggregate.getRelationships();

    // Build parent-child map from "contains" relationships
    const children = new Map<string, string[]>();
    for (const el of elements) {
      children.set(el.id, []);
    }
    for (const rel of relationships) {
      if (rel.kind === RelationshipKind.CONTAINS) {
        children.get(rel.source)?.push(rel.target);
      }
    }

    // Calculate max depth using DFS with memoization
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
    for (const el of elements) {
      maxDepth = Math.max(maxDepth, getDepth(el.id));
    }

    return maxDepth;
  }
}
