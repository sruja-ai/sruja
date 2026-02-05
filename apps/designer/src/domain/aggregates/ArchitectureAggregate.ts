/**
 * ArchitectureAggregate
 *
 * The root aggregate for the architecture domain. Encapsulates all business logic
 * and invariants related to architecture models.
 *
 * @module domain/aggregates
 */

import type {
  SrujaModelDump,
  Element,
  Relationship,
  ParsedView,
  Specification,
} from "@sruja/shared";

import { ElementId } from "../value-objects/ElementId";
import { ValidationError } from "@sruja/shared/utils";
import { ok, err, type Result } from "@sruja/shared/utils/result";

/**
 * Architecture aggregate root
 *
 * Acts as the consistency boundary for the architecture domain. All mutations
 * to the architecture model should go through this aggregate to ensure
 * business rules are enforced.
 *
 * @remarks
 * This aggregate is immutable. All operations return a new instance.
 */
export class ArchitectureAggregate {
  /**
   * Creates a new ArchitectureAggregate from a SrujaModelDump
   *
   * @param dump - The model dump to create the aggregate from
   * @returns Result containing the aggregate or validation error
   */
  static fromDump(dump: SrujaModelDump): Result<ArchitectureAggregate, ValidationError> {
    try {
      // Validate the dump structure
      if (!dump.elements || !dump.relations) {
        return err(new ValidationError("Invalid model dump: missing elements or relations"));
      }

      // Create value objects for elements
      const elements = new Map<string, Element>();
      for (const [id, element] of Object.entries(dump.elements || {})) {
        elements.set(id, element);
      }

      // Map relations (FqnRef source/target) to Relationship[] (string source/target)
      const relations = dump.relations ?? [];
      const relationships: Relationship[] = relations.map((rel) => ({
        ...rel,
        id: rel.id,
        source: typeof rel.source === "string" ? rel.source : rel.source.model,
        target: typeof rel.target === "string" ? rel.target : rel.target.model,
      }));

      const defaultMetadata = {
        name: "Untitled",
        version: "1.0.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0.0",
      } as const;
      const aggregate = new ArchitectureAggregate(
        dump._metadata ?? defaultMetadata,
        elements,
        relationships,
        dump.views ?? {},
        dump.specification,
        dump.sruja
      );

      // Validate business rules
      const validationResult = aggregate.validate();
      if (!validationResult.ok) {
        return validationResult;
      }

      return ok(aggregate);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to create architecture aggregate: ${error instanceof Error ? error.message : "Unknown error"}`
        )
      );
    }
  }

  /**
   * Creates a new empty ArchitectureAggregate
   */
  static createEmpty(name: string = "Untitled"): Result<ArchitectureAggregate, ValidationError> {
    const metadata = {
      name,
      version: "1.0.0",
      generated: new Date().toISOString(),
      srujaVersion: "1.0.0",
    } as const;

    const aggregate = new ArchitectureAggregate(metadata, new Map(), [], {}, undefined, undefined);
    return ok(aggregate);
  }

  public readonly metadata: SrujaModelDump["_metadata"];
  private readonly elements: Map<string, Element>;
  private readonly relationships: Relationship[];
  private readonly views: Record<string, ParsedView>;
  public readonly specification?: Specification;
  public readonly sruja?: SrujaModelDump["sruja"];

  private constructor(
    metadata: SrujaModelDump["_metadata"],
    elements: Map<string, Element>,
    relationships: Relationship[],
    views: Record<string, ParsedView>,
    specification?: Specification,
    sruja?: SrujaModelDump["sruja"]
  ) {
    this.metadata = metadata;
    this.elements = elements;
    this.relationships = relationships;
    this.views = views;
    this.specification = specification;
    this.sruja = sruja;
  }

  // =========================================================================
  // Element Operations
  // =========================================================================

  /**
   * Adds a new element to the architecture
   *
   * @param element - The element to add
   * @returns Result containing new aggregate or validation error
   */
  addElement(element: Element): Result<ArchitectureAggregate, ValidationError> {
    // Validate element
    const idResult = ElementId.create(element.id);
    if (!idResult.ok) {
      return err(idResult.error);
    }

    // Check for duplicate ID
    if (this.elements.has(element.id)) {
      return err(new ValidationError(`Element with ID '${element.id}' already exists`));
    }

    // Validate element structure
    if (!element.title || element.title.trim() === "") {
      return err(new ValidationError("Element title cannot be empty"));
    }

    // Create new elements map (immutable)
    const newElements = new Map(this.elements);
    newElements.set(element.id, element);

    return ok(this.clone({ elements: newElements }));
  }

  /**
   * Updates an existing element
   *
   * @param id - The ID of the element to update
   * @param updates - Partial updates to apply
   * @returns Result containing new aggregate or validation error
   */
  updateElement(
    id: string,
    updates: Partial<Element>
  ): Result<ArchitectureAggregate, ValidationError> {
    const element = this.elements.get(id);
    if (!element) {
      return err(new ValidationError(`Element with ID '${id}' not found`));
    }

    // Create updated element
    const updatedElement: Element = {
      ...element,
      ...updates,
      id, // ID cannot be changed
    };

    // Validate title if updated
    if (updates.title !== undefined && updates.title.trim() === "") {
      return err(new ValidationError("Element title cannot be empty"));
    }

    const newElements = new Map(this.elements);
    newElements.set(id, updatedElement);

    return ok(this.clone({ elements: newElements }));
  }

  /**
   * Removes an element from the architecture
   *
   * @param id - The ID of the element to remove
   * @returns Result containing new aggregate or validation error
   */
  removeElement(id: string): Result<ArchitectureAggregate, ValidationError> {
    const element = this.elements.get(id);
    if (!element) {
      return err(new ValidationError(`Element with ID '${id}' not found`));
    }

    // Remove the element
    const newElements = new Map(this.elements);
    newElements.delete(id);

    // Remove all relationships connected to this element
    const newRelationships = this.relationships.filter(
      (rel) => rel.source !== id && rel.target !== id
    );

    return ok(
      this.clone({
        elements: newElements,
        relationships: newRelationships,
      })
    );
  }

  /**
   * Gets an element by ID
   *
   * @param id - The element ID
   * @returns The element or undefined if not found
   */
  getElement(id: string): Element | undefined {
    return this.elements.get(id);
  }

  /**
   * Gets all elements
   *
   * @returns Array of all elements
   */
  getAllElements(): Element[] {
    return Array.from(this.elements.values());
  }

  /**
   * Finds elements by kind
   *
   * @param kind - The kind of element to find
   * @returns Array of matching elements
   */
  findElementsByKind(kind: string): Element[] {
    return this.getAllElements().filter((el) => el.kind === kind);
  }

  /**
   * Finds elements by tag
   *
   * @param tag - The tag to search for
   * @returns Array of matching elements
   */
  findElementsByTag(tag: string): Element[] {
    return this.getAllElements().filter((el) => el.tags && el.tags.includes(tag));
  }

  // =========================================================================
  // Relationship Operations
  // =========================================================================

  /**
   * Adds a relationship between two elements
   *
   * @param sourceId - Source element ID
   * @param targetId - Target element ID
   * @param relationship - Relationship details
   * @returns Result containing new aggregate or validation error
   */
  addRelationship(
    sourceId: string,
    targetId: string,
    relationship: Omit<Relationship, "source" | "target">
  ): Result<ArchitectureAggregate, ValidationError> {
    // Validate both elements exist
    if (!this.elements.has(sourceId)) {
      return err(new ValidationError(`Source element '${sourceId}' not found`));
    }
    if (!this.elements.has(targetId)) {
      return err(new ValidationError(`Target element '${targetId}' not found`));
    }

    // Prevent self-relationships
    if (sourceId === targetId) {
      return err(new ValidationError("Cannot create relationship between element and itself"));
    }

    // Check for duplicate relationship
    const duplicateExists = this.relationships.some(
      (rel) => rel.source === sourceId && rel.target === targetId
    );
    if (duplicateExists) {
      return err(
        new ValidationError(`Relationship already exists between '${sourceId}' and '${targetId}'`)
      );
    }

    const newRelationship: Relationship = {
      ...relationship,
      source: sourceId,
      target: targetId,
    };

    const newRelationships = [...this.relationships, newRelationship];
    return ok(this.clone({ relationships: newRelationships }));
  }

  /**
   * Updates a relationship
   *
   * @param sourceId - Source element ID
   * @param targetId - Target element ID
   * @param updates - Partial updates to apply
   * @returns Result containing new aggregate or validation error
   */
  updateRelationship(
    sourceId: string,
    targetId: string,
    updates: Partial<Relationship>
  ): Result<ArchitectureAggregate, ValidationError> {
    const index = this.relationships.findIndex(
      (rel) => rel.source === sourceId && rel.target === targetId
    );

    if (index === -1) {
      return err(
        new ValidationError(`Relationship not found between '${sourceId}' and '${targetId}'`)
      );
    }

    const existing = this.relationships[index];
    const updated: Relationship = {
      ...existing,
      ...updates,
      source: sourceId, // Cannot change source/target
      target: targetId,
    };

    const newRelationships = [...this.relationships];
    newRelationships[index] = updated;

    return ok(this.clone({ relationships: newRelationships }));
  }

  /**
   * Removes a relationship
   *
   * @param sourceId - Source element ID
   * @param targetId - Target element ID
   * @returns Result containing new aggregate or validation error
   */
  removeRelationship(
    sourceId: string,
    targetId: string
  ): Result<ArchitectureAggregate, ValidationError> {
    const index = this.relationships.findIndex(
      (rel) => rel.source === sourceId && rel.target === targetId
    );

    if (index === -1) {
      return err(
        new ValidationError(`Relationship not found between '${sourceId}' and '${targetId}'`)
      );
    }

    const newRelationships = this.relationships.filter(
      (rel) => !(rel.source === sourceId && rel.target === targetId)
    );

    return ok(this.clone({ relationships: newRelationships }));
  }

  /**
   * Gets all relationships
   *
   * @returns Array of all relationships
   */
  getRelationships(): Relationship[] {
    return [...this.relationships];
  }

  /**
   * Gets relationships for a specific element
   *
   * @param elementId - The element ID
   * @returns Array of relationships where the element is source or target
   */
  getRelationshipsForElement(elementId: string): Relationship[] {
    return this.relationships.filter((rel) => rel.source === elementId || rel.target === elementId);
  }

  /**
   * Gets upstream relationships (where element is target)
   *
   * @param elementId - The element ID
   * @returns Array of upstream relationships
   */
  getUpstreamRelationships(elementId: string): Relationship[] {
    return this.relationships.filter((rel) => rel.target === elementId);
  }

  /**
   * Gets downstream relationships (where element is source)
   *
   * @param elementId - The element ID
   * @returns Array of downstream relationships
   */
  getDownstreamRelationships(elementId: string): Relationship[] {
    return this.relationships.filter((rel) => rel.source === elementId);
  }

  // =========================================================================
  // Business Rules Validation
  // =========================================================================

  /**
   * Validates the aggregate against business rules
   *
   * @returns Result containing void or validation error
   */
  validate(): Result<void, ValidationError> {
    // Validate all relationships reference existing elements
    for (const rel of this.relationships) {
      if (!this.elements.has(rel.source)) {
        return err(
          new ValidationError(
            `Relationship references non-existent source element: '${rel.source}'`
          )
        );
      }
      if (!this.elements.has(rel.target)) {
        return err(
          new ValidationError(
            `Relationship references non-existent target element: '${rel.target}'`
          )
        );
      }
    }

    // Check for cycles in relationships
    const cycles = this.detectCycles();
    if (cycles.length > 0) {
      return err(
        new ValidationError(
          `Cyclic dependencies detected: ${cycles.map((c) => c.join(" -> ")).join(", ")}`
        )
      );
    }

    return ok(undefined);
  }

  /**
   * Detects cycles in the relationship graph
   *
   * @private
   * @returns Array of cycles found
   */
  private detectCycles(): string[][] {
    const visited = new Set<string>();
    const recursionStack = new Set<string>();
    const cycles: string[][] = [];

    const dfs = (node: string, path: string[]) => {
      visited.add(node);
      recursionStack.add(node);

      const outgoing = this.getDownstreamRelationships(node);
      for (const rel of outgoing) {
        const neighbor = rel.target;

        if (!visited.has(neighbor)) {
          const result = dfs(neighbor, [...path, neighbor]);
          if (result.length > 0) {
            cycles.push(result);
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
      return [];
    };

    for (const element of this.getAllElements()) {
      if (!visited.has(element.id)) {
        dfs(element.id, [element.id]);
      }
    }

    return cycles;
  }

  /**
   * Checks if the architecture has orphan elements (no relationships)
   *
   * @returns Array of orphan element IDs
   */
  findOrphanElements(): string[] {
    return this.getAllElements()
      .filter((el) => this.getRelationshipsForElement(el.id).length === 0)
      .map((el) => el.id);
  }

  // =========================================================================
  // View Operations
  // =========================================================================

  /**
   * Adds or updates a view
   *
   * @param name - View name
   * @param view - View definition
   * @returns Result containing new aggregate or validation error
   */
  setView(name: string, view: ParsedView): Result<ArchitectureAggregate, ValidationError> {
    const newViews = { ...this.views, [name]: view };
    return ok(this.clone({ views: newViews }));
  }

  /**
   * Removes a view
   *
   * @param name - View name
   * @returns Result containing new aggregate or validation error
   */
  removeView(name: string): Result<ArchitectureAggregate, ValidationError> {
    if (!this.views[name]) {
      return err(new ValidationError(`View '${name}' not found`));
    }

    const newViews = { ...this.views };
    delete newViews[name];
    return ok(this.clone({ views: newViews }));
  }

  /**
   * Gets a view by name
   *
   * @param name - View name
   * @returns The view or undefined if not found
   */
  getView(name: string): ParsedView | undefined {
    return this.views[name];
  }

  /**
   * Gets all views
   *
   * @returns Object mapping view names to views
   */
  getAllViews(): Record<string, ParsedView> {
    return { ...this.views };
  }

  // =========================================================================
  // Serialization
  // =========================================================================

  /**
   * Converts the aggregate to a SrujaModelDump
   *
   * @returns The model dump
   */
  toDump(): SrujaModelDump {
    const elements: Record<string, Element> = {};
    for (const [id, element] of this.elements.entries()) {
      elements[id] = element;
    }

    const relations = this.relationships.map((rel) => ({
      ...rel,
      source: { model: rel.source },
      target: { model: rel.target },
    }));

    return {
      elements,
      relations,
      views: this.views,
      _metadata: this.metadata,
      specification: this.specification,
      sruja: this.sruja,
    };
  }

  // =========================================================================
  // Private Helpers
  // =========================================================================

  /**
   * Creates a clone of the aggregate with updated fields
   *
   * @private
   * @param updates - Fields to update
   * @returns New aggregate instance
   */
  private clone(
    updates: Partial<{
      elements: Map<string, Element>;
      relationships: Relationship[];
      views: Record<string, ParsedView>;
    }>
  ): ArchitectureAggregate {
    return new ArchitectureAggregate(
      this.metadata,
      updates.elements ?? this.elements,
      updates.relationships ?? this.relationships,
      updates.views ?? this.views,
      this.specification,
      this.sruja
    );
  }
}
