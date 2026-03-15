/**
 * Safe JSON parsing with optional type guard. Avoids silent misuse of JSON.parse.
 */

export type ParseResult<T> = { ok: true; value: T } | { ok: false; error: string };

/**
 * Parse JSON and optionally validate shape. Returns result object instead of throwing.
 */
export function parseJsonSafe<T>(
  raw: string,
  guard?: (value: unknown) => value is T
): ParseResult<T> {
  try {
    const value: unknown = JSON.parse(raw);
    if (guard && !guard(value)) {
      return { ok: false, error: "JSON shape validation failed" };
    }
    return { ok: true, value: value as T };
  } catch (e) {
    const message = e instanceof SyntaxError ? e.message : String(e);
    return { ok: false, error: message };
  }
}

/** Type guard: value is a non-null object. */
export function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}
