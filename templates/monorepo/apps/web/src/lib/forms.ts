import { ApiError } from "./http"

export type ServerErrors = {
  /** Field name -> first message. */
  fields: Record<string, string>
  /** Anything not tied to a field. */
  form?: string
  /** HTTP status of the ApiError behind `form`, when there was one. */
  status?: number
}

export type ServerErrorOptions = {
  conflictField?: string
  fieldMap?: Record<string, string>
  /**
   * Where a 400 `bad_request` belongs. A field name, or a function that picks
   * one from the server's message (return undefined to keep it form-level).
   * Password-policy rejections ("This password is too easy to guess.") arrive
   * as 400, not as 422 field errors.
   */
  badRequestField?: string | ((message: string) => string | undefined)
  /** Where a 401 belongs, for endpoints where it means "wrong credential". */
  unauthorizedField?: string
}

/**
 * Turns an ApiError into per-field messages plus an optional form-level one.
 * 422 details map onto fields; 409 lands on `conflictField` when given; 400
 * lands on `badRequestField` when given.
 */
export function toServerErrors(
  error: unknown,
  opts: ServerErrorOptions = {}
): ServerErrors {
  if (!(error instanceof ApiError)) {
    return { fields: {}, form: "Something went wrong. Please try again." }
  }
  if (error.status === 422 && error.details) {
    const fields: Record<string, string> = {}
    for (const [name, issues] of Object.entries(error.details)) {
      const target = opts.fieldMap?.[name] ?? name
      if (issues[0]) fields[target] = issues[0].message
    }
    return { fields, status: error.status }
  }
  if (error.status === 409 && opts.conflictField) {
    return {
      fields: { [opts.conflictField]: error.message },
      status: error.status,
    }
  }
  if (error.status === 400 && opts.badRequestField) {
    const field =
      typeof opts.badRequestField === "function"
        ? opts.badRequestField(error.message)
        : opts.badRequestField
    if (field) return { fields: { [field]: error.message }, status: 400 }
  }
  if (error.status === 401 && opts.unauthorizedField) {
    return { fields: { [opts.unauthorizedField]: error.message }, status: 401 }
  }
  if (error.status === 429) {
    return {
      fields: {},
      form: "Too many attempts. Try again in a minute.",
      status: error.status,
    }
  }
  return { fields: {}, form: error.message, status: error.status }
}
