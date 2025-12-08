// packages/viewer/src/serialize/nodes/top-level.ts
import type { NodeSingular } from 'cytoscape';
import type { ArchitectureBody } from '../../types';
import { getMeta } from './helpers';

/**
 * Serialize person node
 */
export function serializePerson(node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  if (!arch.persons) arch.persons = [];
  arch.persons.push({
    id: data.id,
    label: data.label,
    metadata: getMeta(node),
  });
}

/**
 * Serialize system node
 */
export function serializeSystem(node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  if (!arch.systems) arch.systems = [];
  arch.systems.push({
    id: data.id,
    label: data.label,
    metadata: getMeta(node),
  });
}

/**
 * Serialize requirement node
 */
export function serializeRequirement(node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  if (!arch.requirements) arch.requirements = [];
  arch.requirements.push({
    id: data.id,
    title: data.label,
    metadata: getMeta(node),
  });
}

/**
 * Serialize ADR node
 */
export function serializeADR(node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  if (!arch.adrs) arch.adrs = [];
  arch.adrs.push({
    id: data.id,
    title: data.label,
    metadata: getMeta(node),
  });
}

/**
 * Serialize deployment node
 */
export function serializeDeployment(node: NodeSingular, arch: ArchitectureBody): void {
  const data = node.data();
  if (!arch.deployment) arch.deployment = [];
  arch.deployment.push({
    id: data.id,
    label: data.label,
    metadata: getMeta(node),
  });
}
