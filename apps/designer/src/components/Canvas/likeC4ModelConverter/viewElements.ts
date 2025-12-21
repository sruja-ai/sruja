// View element generation for C4 hierarchy levels
import type { SrujaModelDump } from "@sruja/shared";
import type { C4Level } from "../types";

/**
 * Generates proper view elements for C4 hierarchy levels based on model structure.
 * 
 * @param model - The model dump containing elements
 * @param level - The C4 level (L1, L2, L3)
 * @param focusedSystemId - For L2: the system to show containers for
 * @param focusedContainerId - For L3: the container to show components for
 * @returns Array of element IDs to include in the view
 * 
 * @remarks
 * This function implements the C4 model hierarchy:
 * - L1: Landscape view (persons and systems)
 * - L2: Container view (containers, datastores, queues within a system)
 * - L3: Component view (components within a container)
 */
export function generateLevelViewElements(
  model: SrujaModelDump,
  level: C4Level,
  focusedSystemId: string | null = null,
  focusedContainerId: string | null = null
): string[] {
  if (!model.elements || typeof model.elements !== "object") {
    return [];
  }

  const elementIds: string[] = [];
  const elements = model.elements;

  if (level === "L1") {
    // L1: Include all persons and systems (top-level, no dots in ID)
    Object.keys(elements).forEach((elementId) => {
      const element = elements[elementId] as any;
      // No dots in ID means it's a top-level element (person or system)
      if (!elementId.includes(".")) {
        const kind = element.kind || element.type || "";
        if (kind === "person" || kind === "system") {
          elementIds.push(elementId);
        }
      }
    });
  } else if (level === "L2" && focusedSystemId) {
    // L2: Include containers, datastores, queues for the focused system
    // Elements should have format: SystemName.ContainerName
    const systemPrefix = `${focusedSystemId}.`;
    Object.keys(elements).forEach((elementId) => {
      if (elementId.startsWith(systemPrefix) && !elementId.substring(systemPrefix.length).includes(".")) {
        const element = elements[elementId] as any;
        const kind = element.kind || element.type || "";
        // Include containers, datastores, queues (but not components which have two dots)
        if (kind === "container" || kind === "datastore" || kind === "queue") {
          elementIds.push(elementId);
        }
      }
    });
    // Also include the system itself
    if (elements[focusedSystemId]) {
      elementIds.push(focusedSystemId);
    }
  } else if (level === "L3" && focusedSystemId && focusedContainerId) {
    // L3: Include components for the focused container
    // Elements should have format: SystemName.ContainerName.ComponentName
    const containerPrefix = `${focusedSystemId}.${focusedContainerId}.`;
    Object.keys(elements).forEach((elementId) => {
      if (elementId.startsWith(containerPrefix)) {
        const element = elements[elementId] as any;
        const kind = element.kind || element.type || "";
        if (kind === "component") {
          elementIds.push(elementId);
        }
      }
    });
    // Also include the container itself
    const containerId = `${focusedSystemId}.${focusedContainerId}`;
    if (elements[containerId]) {
      elementIds.push(containerId);
    }
    // And the system
    if (elements[focusedSystemId]) {
      elementIds.push(focusedSystemId);
    }
  }

  return elementIds;
}

