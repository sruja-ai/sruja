// packages/shared/src/types/architecture.ts
// TypeScript type definitions for Sruja architecture models
// This file re-exports from split modules for backward compatibility

/**
 * @packageDocumentation
 *
 * # Architecture Types
 *
 * Core TypeScript type definitions for Sruja architecture models.
 *
 * ## Module Structure
 *
 * This module has been split into focused sub-modules:
 * - **core.ts**: Core model types (ModelMetadata, LayoutData, SrujaModelDump)
 * - **governance.ts**: Governance layer types (Requirement, ADR, Policy, SLO)
 * - **legacy.ts**: Legacy format types (ArchitectureJSON, etc.)
 * - **guards.ts**: Type guards and validation functions
 *
 * ## Usage
 *
 * ```typescript
 * import type { SrujaModelDump, Requirement, ADR } from '@sruja/shared/types';
 * import { isValidRequirement } from '@sruja/shared/types';
 * ```
 *
 * @module types/architecture
 */

// ===========================================================================
// Core Types
// ===========================================================================
// Core Types
export type {
  // Aliases for compatibility
  SpecificationDump as Specification,
  ElementKindDump as ElementSpecification,
  TagDump as TagSpecification,
  RelationshipKindDump as RelationshipSpecification,
  // Core types
  Element,
  Relationship,
  ParsedView,
  // Dump types
  SpecificationDump,
  ElementKindDump,
  TagDump,
  RelationshipKindDump,
  // Core Sruja types
  ModelMetadata,
  LayoutData,
  SrujaModelExtensions,
  FqnRef,
  SrujaRelationDump,
  SrujaModelDump,
  ElementDump,
  RelationDump,
  GlobalsDump,
  DeploymentsDump,
} from "./core";

// ===========================================================================
// Governance Types
// ===========================================================================
export type {
  SrujaExtensions,
  Requirement,
  ADR,
  Policy,
  Constraint,
  Convention,
  Contract,
  Scenario,
  Step,
  Flow,
  Deployment,
  SLO,
  // Deprecated aliases
  RequirementDump,
  ADRDump,
  PolicyDump,
  ConstraintDump,
  ConventionDump,
  ContractDump,
  ScenarioDump,
  StepDump,
  FlowDump,
  DeploymentDump,
  SLODump,
} from "./governance";

// ===========================================================================
// Legacy Types
// ===========================================================================
export type {
  ArchitectureJSON,
  MetadataJSON,
  MetadataEntryJSON,
  ArchitectureBody,
  NavigationJSON,
  ViewsJSON,
  ViewData,
  ViewNode,
  ViewEdge,
  SystemJSON,
  ContainerJSON,
  ComponentJSON,
  DataStoreJSON,
  QueueJSON,
  PersonJSON,
  RelationJSON,
  ScenarioJSON,
  ScenarioStepJSON,
  FlowJSON,
  RequirementJSON,
  ADRJSON,
  DeploymentNodeJSON,
  ContractJSON,
  ContractBodyJSON,
  SchemaBlockJSON,
  SchemaEntryJSON,
  TypeSpecJSON,
  PolicyJSON,
  OverviewJSON,
  ConstraintJSON,
  ConventionJSON,
  ScenarioNav,
  FlowNav,
  SLOJSON,
  SLOAvailabilityJSON,
  SLOLatencyJSON,
  SLOCurrentJSON,
  SLOErrorRateJSON,
  SLOThroughputJSON,
  ScaleJSON,
} from "./legacy";

// ===========================================================================
// Type Guards and Validation
// ===========================================================================
export {
  isSrujaModelFormat,
  isLegacyFormat,
  isValidRequirement,
  isValidADR,
  isValidPolicy,
  isValidSLO,
} from "./guards";
