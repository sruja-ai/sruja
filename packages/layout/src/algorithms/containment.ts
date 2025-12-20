import type { PositionedC4Node } from "../types";
import type { C4LayoutOptions } from "../c4-options";
import type { Rect } from "../geometry/rect";
import { SAFETY_MARGIN, DEFAULT_PADDING, MIN_PARENT_PADDING } from "../constants";

/**
 * Verify and enforce strict parent containment.
 * Ensures that parent nodes strictly enclose all their children with appropriate padding.
 */
export function enforceParentContainment(
    nodes: Map<string, PositionedC4Node>,
    _options: C4LayoutOptions
): Map<string, PositionedC4Node> {
    // PRE-PROCESSING: Build a reliable map of Parent -> Children
    // We cannot rely solely on `node.childrenIds` as it might be stale or incomplete
    const parentToChildren = new Map<string, string[]>();
    let maxDepth = 0;

    for (const node of nodes.values()) {
        maxDepth = Math.max(maxDepth, node.depth);
        if (node.parentId) {
            const pid = node.parentId;
            if (!parentToChildren.has(pid)) {
                parentToChildren.set(pid, []);
            }
            parentToChildren.get(pid)!.push(node.nodeId);
        }
    }

    // Helper to get bounds of a group of child IDs
    function getChildrenBounds(childIds: string[]): Rect | null {
        let minX = Infinity;
        let minY = Infinity;
        let maxX = -Infinity;
        let maxY = -Infinity;
        let found = false;

        for (const childId of childIds) {
            const childNode = nodes.get(childId);
            if (childNode && childNode.visible !== false) { // Skip invisible
                found = true;
                const bbox = childNode.bbox;
                minX = Math.min(minX, bbox.x);
                minY = Math.min(minY, bbox.y);
                maxX = Math.max(maxX, bbox.x + bbox.width);
                maxY = Math.max(maxY, bbox.y + bbox.height);
            }
        }

        if (!found) return null;
        return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
    }

    // Iterate bottom-up (deepest first)
    for (let d = maxDepth; d >= 0; d--) {
        const layerNodes = Array.from(nodes.values()).filter((n) => n.depth === d);

        for (const node of layerNodes) {
            // Use our reliable map first, fallback to childrenIds
            const derivedChildren = parentToChildren.get(node.nodeId) || [];
            const statedChildren = node.childrenIds || [];
            // Merge unique child IDs
            const allChildren = Array.from(new Set([...derivedChildren, ...statedChildren]));

            if (allChildren.length === 0) continue;

            const childrenBounds = getChildrenBounds(allChildren);

            if (childrenBounds) {
                // Determine padding based on level
                const lvl = (node.level || "").toLowerCase();
                const paddingMap = _options.spacing?.padding || {};
                let basePadding = DEFAULT_PADDING;

                if (lvl === "context" || lvl === "softwaresystem") {
                    basePadding = paddingMap.SoftwareSystem || MIN_PARENT_PADDING;
                } else if (lvl === "container") {
                    basePadding = paddingMap.Container || DEFAULT_PADDING;
                } else if (lvl === "component") {
                    basePadding = paddingMap.Component || DEFAULT_PADDING;
                }

                // SAFETY_MARGIN from constants is 24.
                // We want: ChildBounds + SafetyMargin + Padding inside
                // But optimization uses `padding` logic for separation.
                // Let's ensure we are generous enough to satisfy the limit.

                const requiredLeft = childrenBounds.x - SAFETY_MARGIN - basePadding;
                const requiredTop = childrenBounds.y - SAFETY_MARGIN - basePadding;
                const requiredRight = childrenBounds.x + childrenBounds.width + SAFETY_MARGIN + basePadding;
                const requiredBottom = childrenBounds.y + childrenBounds.height + SAFETY_MARGIN + basePadding;

                // Strict Fit: Parent follows children exactly
                // Do NOT anchor to current.x/y, as that causes "ghost trails" of empty space 
                // if children move away.

                const current = node.bbox;

                const newX = requiredLeft;
                const newY = requiredTop;
                const newWidth = requiredRight - requiredLeft;
                const newHeight = requiredBottom - requiredTop;

                // Only update if changed
                if (
                    Math.abs(newX - current.x) > 1 ||
                    Math.abs(newY - current.y) > 1 ||
                    Math.abs(newWidth - current.width) > 1 ||
                    Math.abs(newHeight - current.height) > 1
                ) {
                    nodes.set(node.nodeId, {
                        ...node,
                        bbox: { x: newX, y: newY, width: newWidth, height: newHeight },
                        // Update content box
                        contentBox: {
                            x: newX + basePadding,
                            y: newY + basePadding,
                            width: newWidth - basePadding * 2,
                            height: newHeight - basePadding * 2
                        }
                    });
                }
            }
        }
    }

    return nodes;
}
