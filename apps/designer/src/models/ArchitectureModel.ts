// apps/designer/src/models/ArchitectureModel.ts
import type {
  SrujaModelDump,
  ElementDump,
  RelationDump,
  Requirement,
  ADR,
  Policy,
  SLO,
  ParsedView,
  ModelMetadata,
} from "@sruja/shared";
import type { Role } from "../components/RoleSwitcher";

/**
 * Core architecture model shared across all role views.
 *
 * This class provides a unified interface to the architecture data,
 * allowing different role views to access the same underlying model
 * while presenting it through different lenses.
 */
export class ArchitectureModel {
  private model: SrujaModelDump | null = null;
  private listeners: Map<Role, Set<(model: SrujaModelDump | null) => void>> = new Map();

  /**
   * Update the underlying architecture model.
   * Notifies all registered role view listeners.
   */
  updateModel(model: SrujaModelDump | null): void {
    this.model = model;
    this.notifyListeners();
  }

  /**
   * Get the current architecture model.
   */
  getModel(): SrujaModelDump | null {
    return this.model;
  }

  /**
   * Register a listener for a specific role view.
   * Returns an unsubscribe function.
   */
  subscribe(role: Role, callback: (model: SrujaModelDump | null) => void): () => void {
    if (!this.listeners.has(role)) {
      this.listeners.set(role, new Set());
    }
    this.listeners.get(role)!.add(callback);

    // Immediately call with current model
    callback(this.model);

    // Return unsubscribe function
    return () => {
      const roleListeners = this.listeners.get(role);
      if (roleListeners) {
        roleListeners.delete(callback);
        if (roleListeners.size === 0) {
          this.listeners.delete(role);
        }
      }
    };
  }

  /**
   * Notify all listeners of model changes.
   */
  private notifyListeners(): void {
    for (const [role, callbacks] of this.listeners.entries()) {
      for (const callback of callbacks) {
        try {
          callback(this.model);
        } catch (error) {
          console.error(`Error in role view listener (${role}):`, error);
        }
      }
    }
  }

  /**
   * Get nodes from the model.
   */
  getNodes(): Map<string, ElementDump> {
    if (!this.model?.elements) {
      return new Map();
    }
    return new Map(Object.entries(this.model.elements));
  }

  /**
   * Get relations from the model.
   */
  getRelations(): readonly RelationDump[] {
    return this.model?.relations || [];
  }

  /**
   * Get requirements from the model.
   */
  getRequirements(): Requirement[] {
    // Requirements are now in sruja extensions, not in specification
    return [...(this.model?.sruja?.requirements || [])];
  }

  /**
   * Get ADRs from the model.
   */
  getADRs(): ADR[] {
    // ADRs are in sruja extensions
    return [...(this.model?.sruja?.adrs || [])];
  }

  /**
   * Get policies from the model.
   */
  getPolicies(): Policy[] {
    // Policies are in sruja extensions
    return [...(this.model?.sruja?.policies || [])];
  }

  /**
   * Get SLOs from the model.
   */
  getSLOs(): SLO[] {
    // SLOs are in sruja extensions
    return [...(this.model?.sruja?.slos || [])];
  }

  /**
   * Get deployments from the model.
   */
  getDeployments(): readonly import("@sruja/shared").Deployment[] {
    // Deployments are in sruja extensions
    return this.model?.sruja?.deployments || [];
  }

  /**
   * Get metadata from the model.
   */
  getMetadata(): ModelMetadata | undefined {
    return this.model?._metadata;
  }

  /**
   * Calculate difference between current model and a baseline.
   */
  getBaselineDiff(baseline: SrujaModelDump | null): {
    components: number;
    relations: number;
    adrs: number;
    policies: number;
    complexity: number;
  } {
    if (!this.model || !baseline) {
      return { components: 0, relations: 0, adrs: 0, policies: 0, complexity: 0 };
    }

    // 1. Component Count Diff
    const currentComponents = Object.keys(this.model.elements || {}).length;
    const baselineComponents = Object.keys(baseline.elements || {}).length;

    // 2. Relations Count Diff
    const currentRelations = (this.model.relations || []).length;
    const baselineRelations = (baseline.relations || []).length;

    // 3. ADR Count Diff
    const getADRCount = (m: SrujaModelDump) => {
      let count = 0;
      // From metadata
      if ((m._metadata as unknown as Record<string, unknown>)?.adrs)
        count += ((m._metadata as unknown as Record<string, unknown>).adrs as unknown[]).length;
      // From specification elements
      if (m.specification?.elements) {
        for (const spec of Object.values(m.specification.elements)) {
          if ((spec as unknown as Record<string, unknown>).adrs)
            count += ((spec as unknown as Record<string, unknown>).adrs as unknown[]).length;
        }
      }
      return count;
    };
    const currentADRs = getADRCount(this.model);
    const baselineADRs = getADRCount(baseline);

    // 4. Policy Count Diff
    const getPolicyCount = (m: SrujaModelDump) => {
      let count = 0;
      if (m.specification?.elements) {
        for (const spec of Object.values(m.specification.elements)) {
          if ((spec as unknown as Record<string, unknown>).policies)
            count += ((spec as unknown as Record<string, unknown>).policies as unknown[]).length;
        }
      }
      return count;
    };
    const currentPolicies = getPolicyCount(this.model);
    const baselinePolicies = getPolicyCount(baseline);

    // 5. Complexity Diff
    const getComplexity = (comps: number, rels: number) => (comps > 0 ? rels / comps : 0);
    const currentComplexity = getComplexity(currentComponents, currentRelations);
    const baselineComplexity = getComplexity(baselineComponents, baselineRelations);

    return {
      components: currentComponents - baselineComponents,
      relations: currentRelations - baselineRelations,
      adrs: currentADRs - baselineADRs,
      policies: currentPolicies - baselinePolicies,
      complexity: currentComplexity - baselineComplexity,
    };
  }

  /**
   * Calculate Blast Radius for a failed node.
   * Returns a set of node IDs that would be impacted (upstream dependents).
   */
  getBlastRadius(failedNodeId: string): Set<string> {
    const impacted = new Set<string>();
    if (!this.model?.relations) return impacted;

    // Build reverse adjacency list (target -> sources)
    // If A calls B (A -> B), then if B fails, A is impacted.
    const dependents = new Map<string, string[]>();
    for (const rel of this.model.relations) {
      if (!dependents.has(rel.target.model)) {
        dependents.set(rel.target.model, []);
      }
      dependents.get(rel.target.model)!.push(rel.source.model);
    }

    // BFS to find all upstream nodes
    const queue = [failedNodeId];
    while (queue.length > 0) {
      const current = queue.shift()!;
      if (dependents.has(current)) {
        for (const dependent of dependents.get(current)!) {
          if (!impacted.has(dependent) && dependent !== failedNodeId) {
            impacted.add(dependent);
            queue.push(dependent);
          }
        }
      }
    }

    return impacted;
  }

  /**
   * Get all views from the model.
   */
  getViews(): Record<string, ParsedView> {
    return this.model?.views || {};
  }

  /**
   * Get views filtered by a specific tag.
   * Tags are simple strings like "devops", "sre", "failure-scenario", etc.
   *
   * @param tag - The tag to filter by (e.g., "devops", "sre", "engineering-manager")
   * @returns Views that have this tag
   */
  getViewsByTag(tag: string): ParsedView[] {
    const views = this.getViews();
    return Object.values(views).filter((view: ParsedView) => view.tags?.includes(tag));
  }

  /**
   * Discover all role elements from the model.
   * Roles are elements with kind="role" defined in the DSL.
   *
   * @returns Array of role element IDs
   *
   * Example: If DSL has `role devops "DevOps Team"`, this returns ["devops"]
   */
  discoverRoles(): string[] {
    if (!this.model?.elements) {
      return [];
    }

    const roles: string[] = [];
    for (const [id, element] of Object.entries(this.model.elements)) {
      if (element.kind === "role") {
        roles.push(id);
      }
    }

    return roles.sort();
  }

  /**
   * Discover all unique tags from views.
   * Returns all tags found in views, which can represent roles, teams, or any categorization.
   *
   * @returns Array of unique tag values found in all views
   *
   * Example: If views have tags ["devops", "sre", "engineering-manager"],
   * this returns ["devops", "engineering-manager", "sre"] (sorted)
   */
  discoverTags(): string[] {
    const views = this.getViews();
    const tagSet = new Set<string>();

    for (const view of Object.values(views)) {
      if (view.tags) {
        for (const tag of view.tags) {
          if (tag) {
            tagSet.add(tag);
          }
        }
      }
    }

    return Array.from(tagSet).sort();
  }

  /**
   * Get views associated with a specific Role.
   * Filters views by checking if their tags reference a role element.
   *
   * @param role - Role element ID (e.g., 'devops', 'sre')
   * @returns Views that have this role ID in their tags
   *
   * Example: If DSL has `role devops "DevOps Team"` and a view has `tags ["devops"]`,
   * calling getViewsByRole("devops") will return that view.
   */
  getViewsByRole(role: Role | string): ParsedView[] {
    // Simple tag matching - just look for the role name as a tag
    return this.getViewsByTag(role.toLowerCase());
  }
}

// Singleton instance
let architectureModelInstance: ArchitectureModel | null = null;

/**
 * Get the singleton ArchitectureModel instance.
 */
export function getArchitectureModel(): ArchitectureModel {
  if (!architectureModelInstance) {
    architectureModelInstance = new ArchitectureModel();
  }
  return architectureModelInstance;
}
