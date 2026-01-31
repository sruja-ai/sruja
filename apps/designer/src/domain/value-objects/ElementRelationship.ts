/**
 * ElementRelationship Value Object
 *
 * A type-safe, validated relationship between two architecture elements.
 * Encapsulates all the properties and business rules for element relationships.
 *
 * @module domain/value-objects
 */

import { ElementId, isElementId, type ElementId as ElementIdType } from './ElementId';
import { ValidationError, ok, err, type Result } from '@sruja/shared/utils';

/**
 * Valid relationship kinds for architecture elements
 *
 * These are the standard relationship types used in C4-style architecture diagrams.
 */
export enum RelationshipKind {
  /** Uses or depends on */
  USES = 'uses',
  /** Delivers to */
  DELIVERS = 'delivers',
  /** Reads data from */
  READS = 'reads',
  /** Writes data to */
  WRITES = 'writes',
  /** Triggers or invokes */
  TRIGGERS = 'triggers',
  /** Contains (for parent-child relationships) */
  CONTAINS = 'contains',
  /** Inherits from */
  INHERITS = 'inherits',
  /** Implements */
  IMPLEMENTS = 'implements',
  /** Communicates with */
  COMMUNICATES = 'communicates',
  /** General relationship */
  RELATES = 'relates',
}

/**
 * Valid relationship direction for visualization
 */
export enum RelationshipDirection {
  /** Relationship flows from source to target */
  FORWARD = 'forward',
  /** Relationship flows from target to source */
  BACKWARD = 'backward',
  /** Relationship is bidirectional */
  BIDIRECTIONAL = 'bidirectional',
  /** No direction shown */
  NONE = 'none',
}

/**
 * Type for relationship identifier (combination of source and target)
 */
export type RelationshipId = string & { readonly __brand: unique symbol };

/**
 * ElementRelationship value object
 *
 * Represents a validated, type-safe relationship between two architecture elements.
 * Relationships are the edges in the architecture graph and must adhere to specific
 * business rules to maintain model integrity.
 *
 * @remarks
 * Relationships are immutable - any modification creates a new instance.
 * All relationships must have valid source and target ElementId values.
 *
 * @example
 * ```typescript
 * // Create a new relationship
 * const sourceId = ElementId.create('web-server')!;
 * const targetId = ElementId.create('database')!;
 *
 * const relationship = ElementRelationship.create(
 *   sourceId.value,
 *   targetId.value,
 *   'reads data from',
 *   { kind: RelationshipKind.READS }
 * );
 *
 * // With error handling
 * if (relationship.ok) {
 *   console.log(relationship.value.id); // 'web-server->database'
 *   console.log(relationship.value.hasKind(RelationshipKind.READS)); // true
 * }
 *
 * // Update description
 * const updated = relationship.value.updateDescription('queries');
 * ```
 */
export class ElementRelationship {
  /**
   * Creates a new validated ElementRelationship
   *
   * @param source - The source element ID
   * @param target - The target element ID
   * @param description - A description of the relationship
   * @param options - Additional options
   * @returns Result containing the ElementRelationship or a ValidationError
   */
  static create(
    source: string,
    target: string,
    description: string,
    options: {
      kind?: RelationshipKind | string;
      technology?: string;
      direction?: RelationshipDirection;
      metadata?: Record<string, unknown>;
    } = {}
  ): Result<ElementRelationship, ValidationError> {
    // Validate source ID
    const sourceResult = ElementId.create(source);
    if (!sourceResult.ok) {
      return err(new ValidationError(`Invalid source element ID: ${sourceResult.error.message}`));
    }

    // Validate target ID
    const targetResult = ElementId.create(target);
    if (!targetResult.ok) {
      return err(new ValidationError(`Invalid target element ID: ${targetResult.error.message}`));
    }

    // Validate description
    const trimmedDescription = description.trim();
    if (trimmedDescription.length === 0) {
      return err(new ValidationError('Relationship description cannot be empty'));
    }
    if (trimmedDescription.length > 500) {
      return err(new ValidationError('Relationship description cannot exceed 500 characters'));
    }

    // Prevent self-relationships (unless explicitly allowed for certain kinds)
    if (source === target && options.kind !== RelationshipKind.CONTAINS) {
      return err(new ValidationError('Cannot create relationship between element and itself'));
    }

    // Validate technology if provided
    if (options.technology !== undefined) {
      const trimmedTech = options.technology.trim();
      if (trimmedTech.length > 100) {
        return err(new ValidationError('Relationship technology cannot exceed 100 characters'));
      }
    }

    // Validate kind if provided
    let kind: RelationshipKind | string | undefined = options.kind;
    if (kind !== undefined) {
      // Normalize string values to known kinds if they match
      const kindLower = kind.toLowerCase();
      const knownKind = Object.values(RelationshipKind).find(k => k === kindLower);
      if (knownKind) {
        kind = knownKind;
      }
    }

    // Generate relationship ID
    const id = `${source}->${target}` as RelationshipId;

    const relationship = new ElementRelationship(
      id,
      sourceResult.value,
      targetResult.value,
      trimmedDescription,
      kind,
      options.technology?.trim(),
      options.direction ?? RelationshipDirection.FORWARD,
      options.metadata ?? {}
    );

    return ok(relationship);
  }

  /**
   * Creates an ElementRelationship from existing data without validation
   *
   * @remarks
   * Use only when data is already validated (e.g., from trusted storage).
   *
   * @param data - The relationship data
   * @returns An ElementRelationship instance
   */
  static unsafeCreate(data: {
    source: string;
    target: string;
    description: string;
    kind?: RelationshipKind | string;
    technology?: string;
    direction?: RelationshipDirection;
    metadata?: Record<string, unknown>;
  }): ElementRelationship {
    const id = `${data.source}->${data.target}` as RelationshipId;
    return new ElementRelationship(
      id,
      ElementId.unsafe(data.source),
      ElementId.unsafe(data.target),
      data.description,
      data.kind,
      data.technology,
      data.direction ?? RelationshipDirection.FORWARD,
      data.metadata ?? {}
    );
  }

  /**
   * Checks if a relationship ID is valid
   *
   * @param value - The value to check
   * @returns True if valid
   */
  static isValidId(value: string): boolean {
    const parts = value.split('->');
    return (
      parts.length === 2 &&
      ElementId.isValid(parts[0]) &&
      ElementId.isValid(parts[1])
    );
  }

  /**
   * The unique identifier for this relationship
   */
  readonly id: RelationshipId;

  /**
   * The source element ID
   */
  readonly source: ElementIdType;

  /**
   * The target element ID
   */
  readonly target: ElementIdType;

  /**
   * Human-readable description of the relationship
   */
  readonly description: string;

  /**
   * The kind/type of relationship
   */
  readonly kind?: RelationshipKind | string;

  /**
   * The technology used in this relationship (optional)
   */
  readonly technology?: string;

  /**
   * The direction of the relationship for visualization
   */
  readonly direction: RelationshipDirection;

  /**
   * Additional metadata (key-value pairs)
   */
  readonly metadata: Record<string, unknown>;

  /**
   * Private constructor - use static factory methods
   *
   * @private
   */
  private constructor(
    id: RelationshipId,
    source: ElementIdType,
    target: ElementIdType,
    description: string,
    kind?: RelationshipKind | string,
    technology?: string,
    direction = RelationshipDirection.FORWARD,
    metadata: Record<string, unknown> = {}
  ) {
    this.id = id;
    this.source = source;
    this.target = target;
    this.description = description;
    this.kind = kind;
    this.technology = technology;
    this.direction = direction;
    this.metadata = Object.freeze({ ...metadata });
    Object.freeze(this); // Make immutable
  }

  // =========================================================================
  // Query Methods
  // =========================================================================

  /**
   * Checks if this relationship has a specific kind
   *
   * @param kind - The kind to check
   * @returns True if the relationship has this kind
   */
  hasKind(kind: RelationshipKind | string): boolean {
    return this.kind === kind;
  }

  /**
   * Checks if this relationship involves a specific element
   *
   * @param elementId - The element ID to check
   * @returns True if the relationship involves this element
   */
  involves(elementId: string | ElementIdType): boolean {
    const id = typeof elementId === 'string' ? elementId : elementId.value;
    return this.source.value === id || this.target.value === id;
  }

  /**
   * Checks if this relationship is bidirectional
   *
   * @returns True if bidirectional
   */
  isBidirectional(): boolean {
    return this.direction === RelationshipDirection.BIDIRECTIONAL;
  }

  /**
   * Checks if this relationship goes from source to target
   *
   * @returns True if forward direction
   */
  isForward(): boolean {
    return this.direction === RelationshipDirection.FORWARD;
  }

  /**
   * Checks if this relationship goes from target to source
   *
   * @returns True if backward direction
   */
  isBackward(): boolean {
    return this.direction === RelationshipDirection.BACKWARD;
  }

  /**
   * Checks if this relationship has technology specified
   *
   * @returns True if technology is specified
   */
  hasTechnology(): boolean {
    return this.technology !== undefined && this.technology.length > 0;
  }

  /**
   * Gets a metadata value
   *
   * @param key - The metadata key
   * @returns The metadata value or undefined
   */
  getMetadata<T = unknown>(key: string): T | undefined {
    return this.metadata[key] as T;
  }

  /**
   * Checks if metadata has a specific key
   *
   * @param key - The key to check
   * @returns True if key exists
   */
  hasMetadata(key: string): boolean {
    return key in this.metadata;
  }

  // =========================================================================
  // Update Methods (return new instances)
  // =========================================================================

  /**
   * Updates the description
   *
   * @param newDescription - The new description
   * @returns Result containing new relationship or validation error
   */
  updateDescription(newDescription: string): Result<ElementRelationship, ValidationError> {
    const trimmed = newDescription.trim();
    if (trimmed.length === 0) {
      return err(new ValidationError('Relationship description cannot be empty'));
    }
    if (trimmed.length > 500) {
      return err(new ValidationError('Relationship description cannot exceed 500 characters'));
    }

    return ok(
      new ElementRelationship(
        this.id,
        this.source,
        this.target,
        trimmed,
        this.kind,
        this.technology,
        this.direction,
        this.metadata
      )
    );
  }

  /**
   * Updates the technology
   *
   * @param newTechnology - The new technology (or undefined to remove)
   * @returns Result containing new relationship or validation error
   */
  updateTechnology(newTechnology?: string): Result<ElementRelationship, ValidationError> {
    if (newTechnology !== undefined) {
      const trimmed = newTechnology.trim();
      if (trimmed.length > 100) {
        return err(new ValidationError('Relationship technology cannot exceed 100 characters'));
      }
      return ok(
        new ElementRelationship(
          this.id,
          this.source,
          this.target,
          this.description,
          this.kind,
          trimmed,
          this.direction,
          this.metadata
        )
      );
    }

    return ok(
      new ElementRelationship(
        this.id,
        this.source,
        this.target,
        this.description,
        this.kind,
        undefined,
        this.direction,
        this.metadata
      )
    );
  }

  /**
   * Updates the direction
   *
   * @param newDirection - The new direction
   * @returns A new relationship with updated direction
   */
  updateDirection(newDirection: RelationshipDirection): ElementRelationship {
    return new ElementRelationship(
      this.id,
      this.source,
      this.target,
      this.description,
      this.kind,
      this.technology,
      newDirection,
      this.metadata
    );
  }

  /**
   * Updates the kind
   *
   * @param newKind - The new kind
   * @returns A new relationship with updated kind
   */
  updateKind(newKind: RelationshipKind | string | undefined): ElementRelationship {
    return new ElementRelationship(
      this.id,
      this.source,
      this.target,
      this.description,
      newKind,
      this.technology,
      this.direction,
      this.metadata
    );
  }

  /**
   * Updates metadata
   *
   * @param updates - Partial metadata updates
   * @returns A new relationship with updated metadata
   */
  updateMetadata(updates: Partial<Record<string, unknown>>): ElementRelationship {
    return new ElementRelationship(
      this.id,
      this.source,
      this.target,
      this.description,
      this.kind,
      this.technology,
      this.direction,
      { ...this.metadata, ...updates }
    );
  }

  /**
   * Reverses the relationship (swaps source and target)
   *
   * @returns A new reversed relationship
   */
  reverse(): ElementRelationship {
    const newId = `${this.target}->${this.source}` as RelationshipId;

    // Flip direction
    let newDirection = this.direction;
    if (this.direction === RelationshipDirection.FORWARD) {
      newDirection = RelationshipDirection.BACKWARD;
    } else if (this.direction === RelationshipDirection.BACKWARD) {
      newDirection = RelationshipDirection.FORWARD;
    }

    return new ElementRelationship(
      newId,
      this.target,
      this.source,
      this.description,
      this.kind,
      this.technology,
      newDirection,
      this.metadata
    );
  }

  // =========================================================================
  // Serialization
  // =========================================================================

  /**
   * Converts to plain object
   *
   * @returns Plain object representation
   */
  toJSON(): {
    source: string;
    target: string;
    description: string;
    kind?: RelationshipKind | string;
    technology?: string;
    direction: RelationshipDirection;
    metadata: Record<string, unknown>;
  } {
    return {
      source: this.source.value,
      target: this.target.value,
      description: this.description,
      kind: this.kind,
      technology: this.technology,
      direction: this.direction,
      metadata: this.metadata,
    };
  }

  /**
   * String representation
   *
   * @returns String representation
   */
  toString(): string {
    const techPart = this.technology ? ` [${this.technology}]` : '';
    const kindPart = this.kind ? ` (${this.kind})` : '';
    return `${this.source.value} --${this.description}${techPart}${kindPart}--> ${this.target.value}`;
  }

  /**
   * Symbol for string conversion
   */
  [Symbol.toStringTag](): string {
    return 'ElementRelationship';
  }
}

/**
 * Type guard to check if a value is an ElementRelationship
 *
 * @param value - The value to check
 * @returns True if the value is an ElementRelationship
 */
export function isElementRelationship(value: unknown): value is ElementRelationship {
  return value instanceof ElementRelationship;
}

/**
 * Utility function to assert a value is an ElementRelationship
 *
 * @throws ValidationError if the value is not an ElementRelationship
 * @param value - The value to assert
 * @returns The value as ElementRelationship
 */
export function assertElementRelationship(value: unknown): ElementRelationship {
  if (!isElementRelationship(value)) {
    throw new ValidationError(`Expected ElementRelationship, got ${typeof value}`);
  }
  return value;
}

/**
 * Creates a reverse relationship from an existing one
 *
 * @param relationship - The relationship to reverse
 * @returns A new reversed relationship
 */
export function createReverseRelationship(
  relationship: ElementRelationship
): ElementRelationship {
  return relationship.reverse();
}

/**
 * Compares two relationships for equality
 *
 * @param a - First relationship
 * @param b - Second relationship
 * @returns True if they represent the same relationship
 */
export function relationshipsEqual(
  a: ElementRelationship,
  b: ElementRelationship
): boolean {
  return (
    a.id === b.id &&
    a.description === b.description &&
    a.kind === b.kind &&
    a.technology === b.technology &&
    a.direction === b.direction
  );
}
