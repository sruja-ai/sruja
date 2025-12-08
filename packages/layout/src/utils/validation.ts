export class ValidationError extends Error {
  code: string
  constructor(code: string, message: string) {
    super(message)
    this.code = code
    this.name = 'ValidationError'
  }
}

export type ValidationResult = { ok: true } | { ok: false; errors: ValidationError[] }
