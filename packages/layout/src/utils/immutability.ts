export function freezeDeep<T>(obj: T): T {
  if (obj && typeof obj === 'object') {
    Object.freeze(obj)
    for (const key of Object.keys(obj as any)) {
      const val = (obj as any)[key]
      if (val && typeof val === 'object') freezeDeep(val)
    }
  }
  return obj
}
