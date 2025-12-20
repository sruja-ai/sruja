/**
 * Quadtree Spatial Index Implementation
 * Provides efficient spatial queries for layout operations
 */

import type { SpatialIndex, LayoutNode, Bounds, Point, Rect } from "../core/types";

export class QuadTree implements SpatialIndex {
  public readonly type: "quadtree" = "quadtree";
  private root: QuadTreeNode;
  private _size: number = 0;
  public readonly bounds: Bounds;
  private maxDepth: number;
  private maxItems: number;

  constructor(
    bounds: Bounds,
    maxDepth: number = 10,
    maxItems: number = 10
  ) {
    this.bounds = bounds;
    this.maxDepth = maxDepth;
    this.maxItems = maxItems;
    this.root = new QuadTreeNode(bounds, 0, maxDepth, maxItems);
  }

  insert(node: LayoutNode): void {
    this.root.insert(node);
    this._size++;
  }

  remove(node: LayoutNode): void {
    if (this.root.remove(node)) {
      this._size--;
    }
  }

  query(bounds: Bounds): readonly LayoutNode[] {
    return this.root.query(bounds);
  }

  queryPoint(point: Point): readonly LayoutNode[] {
    return this.root.queryPoint(point);
  }

  nearest(point: Point, maxDistance?: number): readonly LayoutNode[] {
    const bounds: Bounds = {
      minX: point.x - (maxDistance || Infinity),
      minY: point.y - (maxDistance || Infinity),
      maxX: point.x + (maxDistance || Infinity),
      maxY: point.y + (maxDistance || Infinity),
    };

    const candidates = this.query(bounds);
    if (!maxDistance) return candidates;

    // Filter by actual distance
    return candidates.filter((node) => {
      const center = {
        x: node.bbox.x + node.bbox.width / 2,
        y: node.bbox.y + node.bbox.height / 2,
      };
      const dist = Math.hypot(center.x - point.x, center.y - point.y);
      return dist <= maxDistance;
    });
  }

  clear(): void {
    this.root = new QuadTreeNode(this.bounds, 0, this.maxDepth, this.maxItems);
    this._size = 0;
  }

  get size(): number {
    return this._size;
  }
}

class QuadTreeNode {
  private nodes: LayoutNode[] = [];
  private children: (QuadTreeNode | null)[] = [null, null, null, null];
  private divided: boolean = false;

  private bounds: Bounds;
  private depth: number;
  private maxDepth: number;
  private maxItems: number;

  constructor(
    bounds: Bounds,
    depth: number,
    maxDepth: number,
    maxItems: number
  ) {
    this.bounds = bounds;
    this.depth = depth;
    this.maxDepth = maxDepth;
    this.maxItems = maxItems;
  }

  insert(node: LayoutNode): boolean {
    if (!this.intersects(node.bbox)) {
      return false;
    }

    if (this.nodes.length < this.maxItems || this.depth >= this.maxDepth) {
      this.nodes.push(node);
      return true;
    }

    if (!this.divided) {
      this.subdivide();
    }

    return this.insertIntoChildren(node);
  }

  remove(node: LayoutNode): boolean {
    const index = this.nodes.findIndex((n) => n.id === node.id);
    if (index !== -1) {
      this.nodes.splice(index, 1);
      return true;
    }

    if (this.divided) {
      for (const child of this.children) {
        if (child && child.remove(node)) {
          return true;
        }
      }
    }

    return false;
  }

  query(bounds: Bounds): LayoutNode[] {
    const result: LayoutNode[] = [];

    if (!this.boundsIntersect(bounds)) {
      return result;
    }

    for (const node of this.nodes) {
      const boundsRect: Rect = {
        x: bounds.minX,
        y: bounds.minY,
        width: bounds.maxX - bounds.minX,
        height: bounds.maxY - bounds.minY,
      };
      if (this.rectIntersects(node.bbox, boundsRect)) {
        result.push(node);
      }
    }

    if (this.divided) {
      for (const child of this.children) {
        if (child) {
          result.push(...child.query(bounds));
        }
      }
    }

    return result;
  }

  queryPoint(point: Point): LayoutNode[] {
    const result: LayoutNode[] = [];

    if (!this.pointInBounds(point)) {
      return result;
    }

    for (const node of this.nodes) {
      if (this.pointInRect(point, node.bbox)) {
        result.push(node);
      }
    }

    if (this.divided) {
      for (const child of this.children) {
        if (child) {
          result.push(...child.queryPoint(point));
        }
      }
    }

    return result;
  }

  private subdivide(): void {
    const { minX, minY, maxX, maxY } = this.bounds;
    const midX = (minX + maxX) / 2;
    const midY = (minY + maxY) / 2;

    this.children[0] = new QuadTreeNode(
      { minX, minY, maxX: midX, maxY: midY },
      this.depth + 1,
      this.maxDepth,
      this.maxItems
    ); // NW

    this.children[1] = new QuadTreeNode(
      { minX: midX, minY, maxX, maxY: midY },
      this.depth + 1,
      this.maxDepth,
      this.maxItems
    ); // NE

    this.children[2] = new QuadTreeNode(
      { minX, minY: midY, maxX: midX, maxY },
      this.depth + 1,
      this.maxDepth,
      this.maxItems
    ); // SW

    this.children[3] = new QuadTreeNode(
      { minX: midX, minY: midY, maxX, maxY },
      this.depth + 1,
      this.maxDepth,
      this.maxItems
    ); // SE

    // Redistribute existing nodes
    const existingNodes = [...this.nodes];
    this.nodes = [];
    this.divided = true;

    for (const node of existingNodes) {
      this.insertIntoChildren(node);
    }
  }

  private insertIntoChildren(node: LayoutNode): boolean {
    for (const child of this.children) {
      if (child && child.insert(node)) {
        return true;
      }
    }
    return false;
  }

  private intersects(rect: Rect): boolean {
    const boundsRect: Rect = {
      x: this.bounds.minX,
      y: this.bounds.minY,
      width: this.bounds.maxX - this.bounds.minX,
      height: this.bounds.maxY - this.bounds.minY,
    };
    return this.rectIntersects(rect, boundsRect);
  }

  private boundsIntersect(bounds: Bounds): boolean {
    return !(
      bounds.maxX < this.bounds.minX ||
      bounds.minX > this.bounds.maxX ||
      bounds.maxY < this.bounds.minY ||
      bounds.minY > this.bounds.maxY
    );
  }

  private rectIntersects(rect1: Rect, rect2: Rect): boolean {
    return !(
      rect1.x + rect1.width < rect2.x ||
      rect2.x + rect2.width < rect1.x ||
      rect1.y + rect1.height < rect2.y ||
      rect2.y + rect2.height < rect1.y
    );
  }

  private pointInBounds(point: Point): boolean {
    return (
      point.x >= this.bounds.minX &&
      point.x <= this.bounds.maxX &&
      point.y >= this.bounds.minY &&
      point.y <= this.bounds.maxY
    );
  }

  private pointInRect(point: Point, rect: Rect): boolean {
    return (
      point.x >= rect.x &&
      point.x <= rect.x + rect.width &&
      point.y >= rect.y &&
      point.y <= rect.y + rect.height
    );
  }
}

/**
 * Factory function to create spatial index based on configuration
 */
export function createSpatialIndex(enable: boolean = true): SpatialIndex {
  if (!enable) {
    return new DummySpatialIndex();
  }

  // Create with reasonable default bounds
  const defaultBounds: Bounds = {
    minX: -10000,
    minY: -10000,
    maxX: 10000,
    maxY: 10000,
  };

  return new QuadTree(defaultBounds);
}

/**
 * Fallback spatial index implementation for when spatial indexing is disabled
 */
class DummySpatialIndex implements SpatialIndex {
  public readonly type: "quadtree" = "quadtree";
  private nodes: LayoutNode[] = [];
  public readonly bounds: Bounds;

  constructor(bounds: Bounds = { minX: 0, minY: 0, maxX: 1000, maxY: 1000 }) {
    this.bounds = bounds;
  }

  insert(node: LayoutNode): void {
    this.nodes.push(node);
  }

  remove(node: LayoutNode): void {
    const index = this.nodes.findIndex((n) => n.id === node.id);
    if (index !== -1) {
      this.nodes.splice(index, 1);
    }
  }

  query(bounds: Bounds): readonly LayoutNode[] {
    const boundsRect: Rect = {
      x: bounds.minX,
      y: bounds.minY,
      width: bounds.maxX - bounds.minX,
      height: bounds.maxY - bounds.minY,
    };
    return this.nodes.filter((node) => this.rectIntersects(node.bbox, boundsRect));
  }

  queryPoint(point: Point): readonly LayoutNode[] {
    return this.nodes.filter((node) => this.pointInRect(point, node.bbox));
  }

  nearest(point: Point, maxDistance?: number): readonly LayoutNode[] {
    const candidates = this.queryPoint(point);
    if (!maxDistance) return candidates;

    return candidates.filter((node) => {
      const center = {
        x: node.bbox.x + node.bbox.width / 2,
        y: node.bbox.y + node.bbox.height / 2,
      };
      const dist = Math.hypot(center.x - point.x, center.y - point.y);
      return dist <= maxDistance;
    });
  }

  clear(): void {
    this.nodes = [];
  }

  get size(): number {
    return this.nodes.length;
  }

  private rectIntersects(rect1: Rect, rect2: Rect): boolean {
    return !(
      rect1.x + rect1.width < rect2.x ||
      rect2.x + rect2.width < rect1.x ||
      rect1.y + rect1.height < rect2.y ||
      rect2.y + rect2.height < rect1.y
    );
  }

  private pointInRect(point: Point, rect: Rect): boolean {
    return (
      point.x >= rect.x &&
      point.x <= rect.x + rect.width &&
      point.y >= rect.y &&
      point.y <= rect.y + rect.height
    );
  }
}
