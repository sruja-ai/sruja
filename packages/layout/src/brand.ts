import { ValidationError } from './utils/validation'

export type Brand<T, B extends string> = T & { __brand: B }

export type C4Id = Brand<string, 'C4Id'>

export function isC4Id(value: unknown): value is C4Id {
  return typeof value === 'string' && /^[a-zA-Z][a-zA-Z0-9_-]*$/.test(value as string)
}

export function createC4Id(id: string): C4Id {
  if (!id || id.trim().length === 0) {
    throw new ValidationError('E003', 'C4Id cannot be empty')
  }
  if (!/^[a-zA-Z][a-zA-Z0-9_-]*$/.test(id)) {
    throw new ValidationError('E004', `C4Id must start with letter and contain only alphanumeric, hyphens, underscores: "${id}"`)
  }
  return id as C4Id
}
