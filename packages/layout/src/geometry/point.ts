export type Point = { x: number; y: number }

export function add(a: Point, b: Point): Point { return { x: a.x + b.x, y: a.y + b.y } }
export function sub(a: Point, b: Point): Point { return { x: a.x - b.x, y: a.y - b.y } }
export function mid(a: Point, b: Point): Point { return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 } }
