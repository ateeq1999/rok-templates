import { keepPreviousData, useQueries } from "@tanstack/react-query"
import { userDetailQueryOptions } from "@/features/users/queries"
import { keys, queries } from "@/lib/client"
import { shortId } from "./labels"
import type { AuditListParams } from "./schemas"

export const auditKeys = {
  all: keys.auditLog,
  list: (p: AuditListParams) => queries.auditLog.list(p).queryKey,
}

export const auditLogQueryOptions = (params: AuditListParams) => ({
  ...queries.auditLog.list(params),
  placeholderData: keepPreviousData,
})

/**
 * Resolves user ids to their email through GET /users/{id},
 * deduplicated. The queries share the users-detail cache with the rest of the
 * admin area. A user who is gone (404, e.g. soft-deleted) or unreadable falls
 * back to a shortened id.
 */
export function useUserLabels(ids: (string | null | undefined)[]) {
  const unique = [...new Set(ids.filter((id): id is string => Boolean(id)))]
  const results = useQueries({
    queries: unique.map((id) => ({
      ...userDetailQueryOptions(id),
      retry: false,
      staleTime: 5 * 60_000,
    })),
  })

  const labels = new Map<string, { label: string; resolved: boolean }>()
  unique.forEach((id, i) => {
    const user = results[i]?.data
    labels.set(
      id,
      user
        ? { label: user.email, resolved: true }
        : { label: shortId(id), resolved: false }
    )
  })
  return (id: string | null | undefined) =>
    id ? (labels.get(id) ?? { label: shortId(id), resolved: false }) : undefined
}
