export type BadgeVariant = "default" | "secondary" | "destructive" | "outline"

const ACTIONS: Record<string, { label: string; variant: BadgeVariant }> = {
  "user.created_by_admin": { label: "User created", variant: "default" },
  "user.role_changed": { label: "Role changed", variant: "secondary" },
  "user.active_status_changed": {
    label: "Status changed",
    variant: "outline",
  },
  "user.deleted": { label: "User deleted", variant: "destructive" },
}

/** Readable label and badge for an action; an unknown action shows its raw string. */
export function actionMeta(action: string): {
  label: string
  variant: BadgeVariant
} {
  return ACTIONS[action] ?? { label: action, variant: "outline" }
}

/** First 8 characters of an id, for when the user it names can no longer be looked up. */
export function shortId(id: string): string {
  return id.length > 8 ? id.slice(0, 8) : id
}

const MAX_VALUE_LENGTH = 80

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "—"
  if (typeof value === "string") return value
  if (typeof value === "number" || typeof value === "boolean")
    return String(value)
  const json = JSON.stringify(value)
  return json.length > MAX_VALUE_LENGTH
    ? `${json.slice(0, MAX_VALUE_LENGTH - 1)}…`
    : json
}

/**
 * `details` as a flat list of key/value strings. It assumes no particular keys
 * (they vary by action); anything that is not a plain object yields no rows.
 */
export function detailEntries(
  details: unknown
): { key: string; value: string }[] {
  if (!details || typeof details !== "object" || Array.isArray(details))
    return []
  return Object.entries(details as Record<string, unknown>).map(
    ([key, value]) => ({
      key: key.replace(/_/g, " "),
      value: formatValue(value),
    })
  )
}
