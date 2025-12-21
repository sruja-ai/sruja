// Main converter module - orchestrates model conversion to LikeC4 format
import type { SrujaModelDump } from "@sruja/shared";
import type { LikeC4ModelDump } from "@likec4/core/types";
import { DEFAULT_PROJECT_ID, DEFAULT_PROJECT_NAME } from "@sruja/shared/utils/constants";
import { generateLevelViewElements } from "./likeC4ModelConverter/viewElements";
import { normalizeViewRule } from "./likeC4ModelConverter/viewRules";
import { validateRelations } from "./likeC4ModelConverter/relations";

/**
 * Converts a SrujaModelDump to a LikeC4ModelDump format with runtime validation.
 * 
 * @param model - Sruja model dump to convert
 * @param focusedSystemId - Optional system ID for L2 view generation
 * @param focusedContainerId - Optional container ID for L3 view generation
 * @returns LikeC4ModelDump or null if conversion fails
 * 
 * @remarks
 * The WASM backend exports models in LikeC4ModelDump format, but we need to
 * ensure all required fields are present with appropriate defaults.
 * 
 * Runtime validation ensures:
 * - Required fields are present and of correct type
 * - Optional fields have safe defaults
 * - Type compatibility with LikeC4Model.fromDump expectations
 * - Relationships reference existing elements
 * - Views are properly normalized
 */
export function convertToLikeC4ModelDump(
  model: SrujaModelDump,
  focusedSystemId: string | null = null,
  focusedContainerId: string | null = null
): LikeC4ModelDump | null {
  // Input validation
  if (!model || typeof model !== "object") {
    console.error("❌ Model is not a valid object");
    return null;
  }

  // Validate required fields
  if (!model.elements || typeof model.elements !== "object") {
    console.error("❌ Model missing required 'elements' field or elements is not an object");
    return null;
  }

  // Validate elements is a proper object (not array, not null)
  if (Array.isArray(model.elements) || model.elements === null) {
    console.error("❌ Model 'elements' must be an object, not array or null");
    return null;
  }

  try {
    const stage = 'parsed' as const;

    // Validate and construct project object
    const projectId = model.projectId || model.project?.id || DEFAULT_PROJECT_ID;
    const project = model.project || { id: projectId, name: DEFAULT_PROJECT_NAME };

    if (typeof project.id !== "string" || project.id.length === 0) {
      console.error("❌ Invalid project.id: must be non-empty string");
      return null;
    }

    // Validate globals structure
    const globals = model.globals || { predicates: {}, dynamicPredicates: {}, styles: {} };
    if (typeof globals !== "object" || globals === null || Array.isArray(globals)) {
      console.error("❌ Invalid globals: must be an object");
      return null;
    }

    // Add default node color styles based on element kind (if not already defined)
    const styles: Record<string, any> = globals.styles && typeof globals.styles === "object"
      ? { ...globals.styles }
      : {};

    const defaultStyles: Record<string, any> = {
      person: { fill: "#ffcccc", stroke: "#cc6666", strokeWidth: 2 },
      system: { fill: "#cce5ff", stroke: "#4a90e2", strokeWidth: 2 },
      container: { fill: "#ccffcc", stroke: "#66cc66", strokeWidth: 2 },
      component: { fill: "#ffffcc", stroke: "#cccc66", strokeWidth: 2 },
      database: { fill: "#e6ccff", stroke: "#9966cc", strokeWidth: 2 },
      datastore: { fill: "#e6ccff", stroke: "#9966cc", strokeWidth: 2 },
      queue: { fill: "#ffe6cc", stroke: "#cc9966", strokeWidth: 2 },
    };

    // Merge default styles with existing styles (don't override user-defined styles)
    Object.entries(defaultStyles).forEach(([kind, style]) => {
      if (!styles[kind]) {
        styles[kind] = style;
      }
    });

    const enhancedGlobals = {
      ...globals,
      styles
    };

    // Validate and filter relations
    const relations = Array.isArray(model.relations) ? model.relations : [];
    const validRelations = validateRelations(relations, model.elements);

    // Normalize views
    let views = model.views && typeof model.views === "object" && !Array.isArray(model.views)
      ? { ...model.views }
      : {};

    const defaultViewConfig = {
      rules: [{ include: [{ wildcard: true }] }],
      nodes: [],
      edges: []
    };

    const normalizedViews: Record<string, any> = {};
    Object.keys(views).forEach((viewId) => {
      const view = views[viewId] as any;
      const normalizedRules = view.rules && Array.isArray(view.rules) && view.rules.length > 0
        ? view.rules.map(normalizeViewRule)
        : defaultViewConfig.rules;

      normalizedViews[viewId] = {
        id: view.id || viewId,
        title: view.title || viewId,
        _stage: 'parsed' as const,
        _type: view._type || 'element' as const,
        rules: normalizedRules,
        ...(view.nodes && Array.isArray(view.nodes) ? { nodes: view.nodes } : {}),
        ...(view.edges && Array.isArray(view.edges) ? { edges: view.edges } : {}),
        ...Object.fromEntries(
          Object.entries(view).filter(([key]) => !['id', 'title', '_stage', '_type', 'rules'].includes(key))
        )
      };
    });

    // Generate L1, L2, L3 views with proper element filtering
    const l1Elements = generateLevelViewElements(model, "L1");
    normalizedViews["L1"] = {
      id: "L1",
      title: "Landscape View (L1)",
      _stage: 'parsed' as const,
      _type: 'element' as const,
      rules: l1Elements.length > 0
        ? [{ include: l1Elements.map(id => ({ ref: { model: id } })) }]
        : defaultViewConfig.rules,
      nodes: [],
      edges: []
    };

    if (focusedSystemId) {
      const l2Elements = generateLevelViewElements(model, "L2", focusedSystemId);
      normalizedViews["L2"] = {
        id: "L2",
        title: `Container View (L2) - ${focusedSystemId}`,
        _stage: 'parsed' as const,
        _type: 'element' as const,
        rules: l2Elements.length > 0
          ? [{ include: l2Elements.map(id => ({ ref: { model: id } })) }]
          : defaultViewConfig.rules,
        nodes: [],
        edges: []
      };
    }

    if (focusedSystemId && focusedContainerId) {
      const l3Elements = generateLevelViewElements(model, "L3", focusedSystemId, focusedContainerId);
      normalizedViews["L3"] = {
        id: "L3",
        title: `Component View (L3) - ${focusedSystemId}.${focusedContainerId}`,
        _stage: 'parsed' as const,
        _type: 'element' as const,
        rules: l3Elements.length > 0
          ? [{ include: l3Elements.map(id => ({ ref: { model: id } })) }]
          : defaultViewConfig.rules,
        nodes: [],
        edges: []
      };
    }

    if (Object.keys(normalizedViews).length === 0) {
      normalizedViews["index"] = {
        id: "index",
        title: "Index",
        _stage: 'parsed' as const,
        _type: 'element' as const,
        ...defaultViewConfig
      };
    }

    views = normalizedViews;

    // Validate other optional structures
    const specification = model.specification && typeof model.specification === "object" && !Array.isArray(model.specification)
      ? model.specification
      : { elements: {}, tags: {}, relationships: {} };

    const deployments = model.deployments && typeof model.deployments === "object" && !Array.isArray(model.deployments)
      ? model.deployments
      : { elements: {}, relations: {} };

    const imports = model.imports && typeof model.imports === "object" && !Array.isArray(model.imports)
      ? model.imports
      : {};

    const modelDump: LikeC4ModelDump = {
      _stage: stage as any,
      projectId,
      project,
      globals: enhancedGlobals as LikeC4ModelDump['globals'],
      imports: imports as LikeC4ModelDump['imports'],
      deployments: deployments as LikeC4ModelDump['deployments'],
      specification: specification as LikeC4ModelDump['specification'],
      elements: model.elements as LikeC4ModelDump['elements'],
      relations: validRelations as LikeC4ModelDump['relations'],
      views: views as LikeC4ModelDump['views'],
    };

    return modelDump;
  } catch (error) {
    console.error("❌ Error converting model to LikeC4ModelDump:", error);
    if (error instanceof Error) {
      console.error("❌ Error details:", error.message, error.stack);
    }
    return null;
  }
}

// Re-export for convenience
export { generateLevelViewElements } from "./likeC4ModelConverter/viewElements";
