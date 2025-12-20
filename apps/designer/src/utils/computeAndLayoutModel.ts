/**
 * Computes and layouts views from a parsed model dump.
 * 
 * This function replicates what `layoutedModel()` does but works with model dumps
 * instead of requiring DSL source files and file system access.
 * 
 * @param parsedModelDump - A parsed LikeC4ModelDump (can be from JSON)
 * @returns A layouted LikeC4Model ready for rendering
 */

import { LikeC4Model } from "@likec4/core/model";
import { computeLikeC4Model } from "@likec4/core/compute-view";
import type { LikeC4ModelDump, ParsedLikeC4ModelData } from "@likec4/core/types";
import { GraphvizLayouter } from "@likec4/layouts";
import { GraphvizWasmAdapter } from "@likec4/layouts";

export async function computeAndLayoutModel(
  parsedModelDump: LikeC4ModelDump | ParsedLikeC4ModelData
): Promise<LikeC4Model<any>> {
  // Step 1: Ensure we have a parsed model
  // If _stage is not 'parsed', we need to convert it
  // Ensure all required fields are present (required by ParsedLikeC4ModelData and LikeC4Model)
  const parsedData: ParsedLikeC4ModelData = {
    ...parsedModelDump,
    _stage: 'parsed' as const,
    views: parsedModelDump.views || {},
    // Ensure deployments is always an object with elements (required by LikeC4DeploymentModel)
    deployments: {
      elements: parsedModelDump.deployments?.elements || {},
      relations: parsedModelDump.deployments?.relations || {},
    },
    // Ensure imports is always an object (required by LikeC4Model constructor)
    imports: parsedModelDump.imports || {},
    // Ensure relations is always an array (required by LikeC4Model constructor)
    relations: parsedModelDump.relations || [],
    // Ensure elements is always an object (required by LikeC4Model constructor)
    elements: parsedModelDump.elements || {},
  } as ParsedLikeC4ModelData;

  // Step 2: Compute views from rules
  // This converts parsed views (with rules) to computed views (with nodes/edges)
  const computedModel = computeLikeC4Model(parsedData);

  // Step 3: Layout views with Graphviz
  // GraphvizLayouter wraps GraphvizWasmAdapter and provides the layout() method
  const layouter = new GraphvizLayouter(new GraphvizWasmAdapter());
  const layoutedViews = await Promise.all(
    [...computedModel.views()].map(async (view) => {
      try {
        const layouted = await layouter.layout({
          view: view.$view,
          styles: computedModel.$styles,
        });
        return {
          id: view.id,
          diagram: layouted.diagram,
        };
      } catch (error) {
        console.error(`[computeAndLayoutModel] Failed to layout view ${view.id}:`, error);
        // Return the computed view without layout if layouting fails
        return {
          id: view.id,
          diagram: view.$view,
        };
      }
    })
  );

  // Step 4: Create layouted model with layouted views
  // Map layouted views to the correct format: { viewId: layoutedDiagram }
  const layoutedViewsMap = layoutedViews.reduce((acc, { id, diagram }) => {
    acc[id] = diagram;
    return acc;
  }, {} as Record<string, any>);

  const layoutedModel = LikeC4Model.create({
    ...computedModel.$data,
    _stage: 'layouted' as const,
    views: layoutedViewsMap,
  });

  return layoutedModel;
}

