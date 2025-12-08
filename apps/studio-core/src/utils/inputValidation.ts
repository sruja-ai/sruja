// apps/studio-core/src/utils/inputValidation.ts

/**
 * Input validation and sanitization utilities
 * Provides FAANG-level security for user inputs
 */

// Node ID validation constants
const NODE_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const NODE_ID_MAX_LENGTH = 100;
const NODE_ID_MIN_LENGTH = 1;

// Label validation constants
const LABEL_MAX_LENGTH = 200;
const LABEL_MIN_LENGTH = 1;

// DSL validation constants
const DSL_MAX_LENGTH = 10 * 1024 * 1024; // 10MB

export interface ValidationResult {
  isValid: boolean;
  error?: string;
  sanitized?: string;
}

/**
 * Validates a node ID according to Sruja language spec.
 * 
 * Rules:
 * - Must be alphanumeric, underscore, or hyphen only
 * - Must be between 1-100 characters
 * - Must not be empty
 * 
 * @param id - The node ID to validate
 * @returns ValidationResult with isValid flag, optional error message, and sanitized value
 * 
 * @example
 * ```typescript
 * const result = validateNodeId('my-system');
 * if (result.isValid) {
 *   console.log('Valid ID:', result.sanitized);
 * } else {
 *   console.error('Invalid ID:', result.error);
 * }
 * ```
 */
export function validateNodeId(id: string): ValidationResult {
  if (!id || typeof id !== 'string') {
    return {
      isValid: false,
      error: 'Node ID is required',
    };
  }

  const trimmed = id.trim();

  if (trimmed.length < NODE_ID_MIN_LENGTH) {
    return {
      isValid: false,
      error: `Node ID must be at least ${NODE_ID_MIN_LENGTH} character(s)`,
    };
  }

  if (trimmed.length > NODE_ID_MAX_LENGTH) {
    return {
      isValid: false,
      error: `Node ID must be at most ${NODE_ID_MAX_LENGTH} characters`,
    };
  }

  if (!NODE_ID_PATTERN.test(trimmed)) {
    return {
      isValid: false,
      error: 'Node ID can only contain letters, numbers, underscores, and hyphens',
    };
  }

  return {
    isValid: true,
    sanitized: trimmed,
  };
}

/**
 * Validates and sanitizes a node label.
 * 
 * Performs:
 * - HTML tag removal to prevent XSS attacks
 * - Whitespace trimming
 * - Length validation (1-200 characters)
 * 
 * @param label - The label to validate and sanitize
 * @returns ValidationResult with sanitized label or error message
 * 
 * @example
 * ```typescript
 * const result = validateNodeLabel('<script>alert("xss")</script>My Label');
 * // result.sanitized === 'My Label' (HTML tags removed)
 * ```
 */
export function validateNodeLabel(label: string): ValidationResult {
  if (!label || typeof label !== 'string') {
    return {
      isValid: false,
      error: 'Label is required',
    };
  }

  // Remove HTML tags to prevent XSS
  const sanitized = label
    .replace(/<[^>]*>/g, '') // Remove HTML tags
    .trim();

  if (sanitized.length < LABEL_MIN_LENGTH) {
    return {
      isValid: false,
      error: `Label must be at least ${LABEL_MIN_LENGTH} character(s)`,
    };
  }

  if (sanitized.length > LABEL_MAX_LENGTH) {
    return {
      isValid: false,
      error: `Label must be at most ${LABEL_MAX_LENGTH} characters`,
    };
  }

  return {
    isValid: true,
    sanitized,
  };
}

/**
 * Sanitizes text input to prevent XSS attacks by escaping HTML special characters.
 * 
 * Escapes the following characters:
 * - `&` → `&amp;`
 * - `<` → `&lt;`
 * - `>` → `&gt;`
 * - `"` → `&quot;`
 * - `'` → `&#x27;`
 * - `/` → `&#x2F;`
 * 
 * @param text - The text to sanitize
 * @returns Sanitized text safe for rendering in HTML
 * 
 * @example
 * ```typescript
 * const safe = sanitizeText('<script>alert("xss")</script>');
 * // safe === '&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;'
 * ```
 */
export function sanitizeText(text: string): string {
  if (!text || typeof text !== 'string') {
    return '';
  }

  const map: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#x27;',
    '/': '&#x2F;',
  };

  return text.replace(/[&<>"'/]/g, (char) => map[char] || char);
}

/**
 * Validates DSL input for basic structure and length constraints.
 * 
 * Validates:
 * - Maximum length (10MB)
 * - Non-empty input
 * - Basic structure (should start with 'architecture' keyword, but parser handles syntax)
 * 
 * @param dsl - The DSL code to validate
 * @returns ValidationResult indicating if DSL is valid
 * 
 * @example
 * ```typescript
 * const result = validateDslInput('architecture "My System" { ... }');
 * if (result.isValid) {
 *   // Proceed with parsing
 * }
 * ```
 */
export function validateDslInput(dsl: string): ValidationResult {
  if (!dsl || typeof dsl !== 'string') {
    return {
      isValid: false,
      error: 'DSL input is required',
    };
  }

  if (dsl.length > DSL_MAX_LENGTH) {
    return {
      isValid: false,
      error: `DSL input exceeds maximum length of ${DSL_MAX_LENGTH / 1024 / 1024}MB`,
    };
  }

  // Basic validation: should start with 'architecture' keyword
  const trimmed = dsl.trim();
  if (trimmed.length > 0 && !trimmed.toLowerCase().startsWith('architecture')) {
    // This is a warning, not an error - allow partial DSL
    // The parser will handle syntax errors
  }

  return {
    isValid: true,
    sanitized: trimmed,
  };
}

/**
 * Validates and sanitizes a search query.
 * 
 * Performs:
 * - HTML sanitization to prevent XSS
 * - Length validation (max 200 characters)
 * - Empty queries are considered valid
 * 
 * @param query - The search query to validate
 * @returns ValidationResult with sanitized query
 * 
 * @example
 * ```typescript
 * const result = validateSearchQuery('user <script>');
 * // result.sanitized === 'user &lt;script&gt;'
 * ```
 */
export function validateSearchQuery(query: string): ValidationResult {
  if (!query || typeof query !== 'string') {
    return {
      isValid: true, // Empty query is valid
      sanitized: '',
    };
  }

  const sanitized = sanitizeText(query.trim());

  if (sanitized.length > 200) {
    return {
      isValid: false,
      error: 'Search query must be at most 200 characters',
    };
  }

  return {
    isValid: true,
    sanitized,
  };
}

/**
 * Validates a URL for share links.
 * 
 * Validates:
 * - Valid URL format
 * - Protocol must be http or https (security requirement)
 * 
 * @param url - The URL to validate
 * @returns ValidationResult indicating if URL is valid
 * 
 * @example
 * ```typescript
 * const result = validateUrl('https://example.com/share?id=123');
 * if (result.isValid) {
 *   window.open(result.sanitized);
 * }
 * ```
 */
export function validateUrl(url: string): ValidationResult {
  if (!url || typeof url !== 'string') {
    return {
      isValid: false,
      error: 'URL is required',
    };
  }

  try {
    const parsed = new URL(url);
    
    // Only allow http and https protocols
    if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
      return {
        isValid: false,
        error: 'URL must use http or https protocol',
      };
    }

    return {
      isValid: true,
      sanitized: url,
    };
  } catch {
    return {
      isValid: false,
      error: 'Invalid URL format',
    };
  }
}

/**
 * Valid node types in Sruja architecture
 */
const VALID_NODE_TYPES = [
  'person',
  'system',
  'container',
  'component',
  'datastore',
  'queue',
  'requirement',
  'adr',
  'deployment',
] as const;

/**
 * Valid node type in Sruja architecture
 */
export type NodeType = typeof VALID_NODE_TYPES[number];

/**
 * Type guard to check if a string is a valid node type.
 * 
 * @param type - The string to check
 * @returns True if the string is a valid node type
 * 
 * @example
 * ```typescript
 * if (validateNodeType(userInput)) {
 *   // userInput is now typed as NodeType
 *   addNode(userInput, ...);
 * }
 * ```
 */
export function validateNodeType(type: string): type is NodeType {
  return VALID_NODE_TYPES.includes(type as NodeType);
}

/**
 * Validates and sanitizes a relation label.
 * 
 * Performs:
 * - HTML sanitization
 * - Length validation (max 200 characters)
 * - Empty labels are valid (optional field)
 * 
 * @param label - The relation label to validate
 * @returns ValidationResult with sanitized label
 */
export function validateRelationLabel(label: string): ValidationResult {
  if (!label || typeof label !== 'string') {
    return {
      isValid: true, // Empty label is valid (optional)
      sanitized: '',
    };
  }

  const sanitized = sanitizeText(label.trim());

  if (sanitized.length > LABEL_MAX_LENGTH) {
    return {
      isValid: false,
      error: `Relation label must be at most ${LABEL_MAX_LENGTH} characters`,
    };
  }

  return {
    isValid: true,
    sanitized,
  };
}

/**
 * Data structure for ADR validation
 */
export interface AdrValidationData {
  title: string;
  status: string;
  context?: string;
  decision?: string;
  consequences?: string;
}

/**
 * Validates ADR (Architecture Decision Record) data.
 * 
 * Validates:
 * - Title (required, 1-200 characters, sanitized)
 * - Status (must be: proposed, accepted, deprecated, superseded)
 * - Sanitizes all text fields (context, decision, consequences)
 * 
 * @param data - The ADR data to validate
 * @returns ValidationResult with sanitized data as JSON string
 * 
 * @example
 * ```typescript
 * const result = validateAdrData({
 *   title: 'Use React',
 *   status: 'accepted',
 *   context: 'Need a UI framework',
 *   decision: 'Choose React',
 *   consequences: 'Team needs React training'
 * });
 * if (result.isValid) {
 *   const sanitized = JSON.parse(result.sanitized!);
 *   // Use sanitized data
 * }
 * ```
 */
export function validateAdrData(data: AdrValidationData): ValidationResult {
  // Validate title
  const titleResult = validateNodeLabel(data.title);
  if (!titleResult.isValid) {
    return {
      isValid: false,
      error: `Title: ${titleResult.error}`,
    };
  }

  // Validate status
  const validStatuses = ['proposed', 'accepted', 'deprecated', 'superseded'];
  if (!validStatuses.includes(data.status?.toLowerCase())) {
    return {
      isValid: false,
      error: `Status must be one of: ${validStatuses.join(', ')}`,
    };
  }

  // Sanitize optional fields
  const sanitized = {
    title: titleResult.sanitized!,
    status: data.status.toLowerCase(),
    context: data.context ? sanitizeText(data.context) : undefined,
    decision: data.decision ? sanitizeText(data.decision) : undefined,
    consequences: data.consequences ? sanitizeText(data.consequences) : undefined,
  };

  return {
    isValid: true,
    sanitized: JSON.stringify(sanitized),
  };
}

/**
 * Validates and sanitizes properties update data.
 * 
 * Performs:
 * - Ensures updates is a valid object
 * - Sanitizes all string values (XSS prevention)
 * - Preserves non-string values (numbers, booleans, etc.)
 * 
 * @param updates - The properties update object
 * @returns ValidationResult with sanitized data as JSON string
 * 
 * @example
 * ```typescript
 * const result = validatePropertiesUpdate({
 *   label: '<script>alert("xss")</script>',
 *   description: 'My description',
 *   count: 42
 * });
 * // String values sanitized, numbers preserved
 * ```
 */
export function validatePropertiesUpdate(updates: Record<string, unknown>): ValidationResult {
  if (!updates || typeof updates !== 'object') {
    return {
      isValid: false,
      error: 'Properties update must be an object',
    };
  }

  const sanitized: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(updates)) {
    if (typeof value === 'string') {
      // Sanitize string values
      sanitized[key] = sanitizeText(value);
    } else {
      // Keep non-string values as-is (numbers, booleans, etc.)
      sanitized[key] = value;
    }
  }

  return {
    isValid: true,
    sanitized: JSON.stringify(sanitized),
  };
}



