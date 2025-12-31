// packages/shared/src/types/governance.ts
// Governance layer types for Sruja architecture models

/**
 * Sruja governance layer extensions.
 *
 * @public
 * @remarks
 * Contains all governance-related artifacts that extend the core architecture model.
 * These extensions support enterprise-level architecture governance, compliance,
 * and operational excellence requirements.
 *
 * @example
 * const extensions: SrujaExtensions = { requirements: [], adrs: [], policies: [], slos: [] };
 */
export interface SrujaExtensions {
  /** Functional and non-functional requirements */
  readonly requirements?: ReadonlyArray<Requirement>;
  /** Architecture Decision Records */
  readonly adrs?: ReadonlyArray<ADR>;
  /** Security and compliance policies */
  readonly policies?: ReadonlyArray<Policy>;
  /** Architectural constraints */
  readonly constraints?: ReadonlyArray<Constraint>;
  /** Architectural conventions */
  readonly conventions?: ReadonlyArray<Convention>;
  /** API and service contracts */
  readonly contracts?: ReadonlyArray<Contract>;
  /** User scenario flows */
  readonly scenarios?: ReadonlyArray<Scenario>;
  /** Data flow diagrams */
  readonly flows?: ReadonlyArray<Flow>;
  /** Deployment configurations */
  readonly deployments?: ReadonlyArray<Deployment>;
  /** Service Level Objectives */
  readonly slos?: ReadonlyArray<SLO>;
}

/**
 * Requirement definition for functional and non-functional requirements.
 *
 * @public
 * @remarks
 * Requirements can be linked to specific architectural elements for traceability.
 * Supports priority and status tracking for requirement management workflows.
 *
 * @example
 * const requirement: Requirement = { id: 'req-001', title: 'System must handle 10k concurrent users', type: 'non-functional' };
 */
export interface Requirement {
  /** Unique requirement identifier */
  readonly id: string;
  /** Requirement title (required) */
  readonly title: string;
  /** Requirement type (e.g., 'functional', 'non-functional', 'security') */
  readonly type?: string;
  /** Detailed requirement description */
  readonly description?: string;
  /** Priority level (e.g., 'low', 'medium', 'high', 'critical') */
  readonly priority?: string;
  /** Current status (e.g., 'draft', 'approved', 'implemented', 'deprecated') */
  readonly status?: string;
  /** Array of element FQNs this requirement applies to */
  readonly elements?: ReadonlyArray<string>;
  /** Optional tags for grouping or filtering */
  readonly tags?: ReadonlyArray<string>;
}

/**
 * Architecture Decision Record (ADR).
 *
 * @public
 * @remarks
 * Documents important architectural decisions, their context, and consequences.
 * Follows the standard ADR format for decision tracking and knowledge management.
 *
 * @example
 * const adr: ADR = { id: 'adr-001', title: 'Use microservices architecture', status: 'accepted' };
 */
export interface ADR {
  /** Unique ADR identifier */
  readonly id: string;
  /** ADR title (required) */
  readonly title: string;
  /** Decision status (e.g., 'proposed', 'accepted', 'deprecated', 'superseded') */
  readonly status?: string;
  /** Context that led to this decision */
  readonly context?: string;
  /** The decision that was made */
  readonly decision?: string;
  /** Consequences and trade-offs of this decision */
  readonly consequences?: string;
  /** ISO 8601 date when decision was made */
  readonly date?: string;
  /** Author or decision maker */
  readonly author?: string;
  /** Optional tags for grouping or filtering */
  readonly tags?: ReadonlyArray<string>;
}

/**
 * Policy definition for security, compliance, or architectural policies.
 *
 * @public
 * @remarks
 * Policies define rules and guidelines that must be followed.
 * Can be linked to specific elements and have different enforcement levels.
 *
 * @example
 * const policy: Policy = { id: 'policy-001', title: 'All APIs must use HTTPS', category: 'security', enforcement: 'mandatory' };
 */
export interface Policy {
  /** Unique policy identifier */
  readonly id: string;
  /** Policy title (required) */
  readonly title: string;
  /** Policy category (e.g., 'security', 'compliance', 'performance') */
  readonly category?: string;
  /** Enforcement level (e.g., 'mandatory', 'recommended', 'advisory') */
  readonly enforcement?: string;
  /** Detailed policy description */
  readonly description?: string;
  /** Array of element FQNs this policy applies to */
  readonly elements?: ReadonlyArray<string>;
}

/**
 * Architectural constraint definition.
 *
 * @public
 * @remarks
 * Constraints represent limitations or restrictions that must be considered
 * in architectural decisions (e.g., technology restrictions, regulatory requirements).
 *
 * @example
 * const constraint: Constraint = { id: 'constraint-001', description: 'Must use only open-source technologies', type: 'technology' };
 */
export interface Constraint {
  /** Unique constraint identifier */
  readonly id: string;
  /** Constraint description (required) */
  readonly description: string;
  /** Constraint type (e.g., 'technology', 'regulatory', 'budget', 'time') */
  readonly type?: string;
}

/**
 * Architectural convention definition.
 *
 * @public
 * @remarks
 * Conventions represent agreed-upon patterns, naming standards, or practices
 * that teams should follow for consistency.
 *
 * @example
 * const convention: Convention = { id: 'conv-001', description: 'All service names must follow pattern' };
 */
export interface Convention {
  /** Unique convention identifier */
  readonly id: string;
  /** Convention description (required) */
  readonly description: string;
}

/**
 * Contract definition for API or service contracts.
 *
 * @public
 * @remarks
 * Contracts define the interface between services, including request/response
 * schemas, error handling, and versioning information.
 *
 * @example
 * const contract: Contract = { id: 'contract-001', type: 'api', description: 'Payment service API contract' };
 */
export interface Contract {
  /** Unique contract identifier */
  readonly id: string;
  /** Contract type (e.g., 'api', 'event', 'data') */
  readonly type?: string;
  /** Contract description */
  readonly description?: string;
  /** Schema reference (URL or JSON schema) */
  readonly schema?: string;
}

/**
 * User scenario definition (BDD-style user story flows).
 *
 * @public
 * @remarks
 * Scenarios describe user interactions and system behaviors in a step-by-step format.
 * Used for documenting user journeys and acceptance criteria.
 *
 * @example
 * const scenario: Scenario = { id: 'scenario-001', title: 'User places an order', steps: [] };
 */
export interface Scenario {
  /** Unique scenario identifier */
  readonly id: string;
  /** Scenario title (required) */
  readonly title: string;
  /** Scenario description */
  readonly description?: string;
  /** Ordered sequence of steps in this scenario */
  readonly steps?: ReadonlyArray<Step>;
}

/**
 * Step definition used in Scenario or Flow.
 *
 * @public
 * @remarks
 * Represents a single step in a user scenario or data flow.
 * Steps can reference source and target elements to show interaction flow.
 *
 * @example
 * const step: Step = { description: 'User authenticates', from: 'user', to: 'system:auth' };
 */
export interface Step {
  /** Optional step identifier */
  readonly id?: string;
  /** Step description (required) */
  readonly description: string;
  /** Source element FQN (where the step originates) */
  readonly from?: string;
  /** Target element FQN (where the step goes) */
  readonly to?: string;
  /** Optional tags for grouping or filtering */
  readonly tags?: ReadonlyArray<string>;
}

/**
 * Data flow definition (DFD-style flow diagrams).
 *
 * @public
 * @remarks
 * Flows describe data movement and processing through the system.
 * Used for documenting data pipelines and transformation processes.
 *
 * @example
 * const flow: Flow = { id: 'flow-001', title: 'Order processing flow', steps: [] };
 */
export interface Flow {
  /** Unique flow identifier */
  readonly id: string;
  /** Flow title (required) */
  readonly title: string;
  /** Flow description */
  readonly description?: string;
  /** Ordered sequence of steps in this flow */
  readonly steps?: ReadonlyArray<Step>;
}

/**
 * Deployment configuration definition.
 *
 * @public
 * @remarks
 * Represents deployment infrastructure, including hierarchical deployment nodes,
 * technology stack, and instance configurations. Supports multi-region and
 * multi-environment deployments.
 *
 * @example
 * const deployment: Deployment = { id: 'deploy-001', kind: 'kubernetes', title: 'Production deployment' };
 */
export interface Deployment {
  /** Unique deployment identifier */
  readonly id: string;
  /** Deployment kind (e.g., 'kubernetes', 'docker', 'serverless') */
  readonly kind?: string;
  /** Deployment title */
  readonly title?: string;
  /** Deployment description */
  readonly description?: string;
  /** Technology stack used */
  readonly technology?: string;
  /** Nested deployment configurations (hierarchical structure) */
  readonly children?: ReadonlyArray<Deployment>;
  /** Instance identifiers deployed in this configuration */
  readonly instances?: ReadonlyArray<string>;
}

/**
 * Service Level Objective (SLO) definition.
 *
 * @public
 * @remarks
 * Defines measurable service quality targets including availability, latency,
 * throughput, and error budgets. Used for operational excellence and SLA management.
 *
 * @example
 * const slo: SLO = { id: 'slo-001', target: 99.9, availability: 99.95, latency: '200ms' };
 */
export interface SLO {
  /** Unique SLO identifier */
  readonly id: string;
  /** Overall target percentage (0-100) */
  readonly target?: number;
  /** Availability target percentage (0-100) */
  readonly availability?: number;
  /** Latency target (e.g., '200ms', 'p95: 500ms') */
  readonly latency?: string;
  /** Throughput target (e.g., '1000 req/s', '10k messages/min') */
  readonly throughput?: string;
  /** Error budget percentage (0-100) */
  readonly errorBudget?: number;
  /** Recovery Time Objective (RTO) */
  readonly recoveryObjective?: string;
  /** Recovery Point Objective (RPO) */
  readonly recoveryPoint?: string;
}

// ===========================================================================
// Backward Compatibility: Type aliases with "Dump" suffix
// ===========================================================================
// These aliases are kept for backward compatibility with existing code.
// New code should use the types without "Dump" suffix.

/** @deprecated Use `Requirement` instead */
export type RequirementDump = Requirement;
/** @deprecated Use `ADR` instead */
export type ADRDump = ADR;
/** @deprecated Use `Policy` instead */
export type PolicyDump = Policy;
/** @deprecated Use `Constraint` instead */
export type ConstraintDump = Constraint;
/** @deprecated Use `Convention` instead */
export type ConventionDump = Convention;
/** @deprecated Use `Contract` instead */
export type ContractDump = Contract;
/** @deprecated Use `Scenario` instead */
export type ScenarioDump = Scenario;
/** @deprecated Use `Step` instead */
export type StepDump = Step;
/** @deprecated Use `Flow` instead */
export type FlowDump = Flow;
/** @deprecated Use `Deployment` instead */
export type DeploymentDump = Deployment;
/** @deprecated Use `SLO` instead */
export type SLODump = SLO;
