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
 * import { isLikeC4Format, isValidRequirement } from '@sruja/shared/types';
 * ```
 * 
 * @module types/architecture
 */

// ===========================================================================
// Core Types
// ===========================================================================
export type {
  // LikeC4 types
  Element,
  Relationship,
  ParsedView,
  Specification,
  ElementSpecification,
  TagSpecification,
  RelationshipSpecification,
  ElementStyle,
  Link,
  ParsedLikeC4ModelData,
  ComputedLikeC4ModelData,
  LayoutedLikeC4ModelData,
  LikeC4ModelDump,
  Builder,
  BuilderSpecification,
  // Core Sruja types
  ProjectDump,
  ModelMetadata,
  LayoutData,
  SrujaModelExtensions,
  FqnRef,
  SrujaRelationDump,
  SrujaModelDump,
  ElementDump,
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
  isLikeC4Format,
  isLegacyFormat,
  isValidRequirement,
  isValidADR,
  isValidPolicy,
  isValidSLO,
} from "./guards";
