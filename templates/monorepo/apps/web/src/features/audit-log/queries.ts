import {
  keepPreviousData,
  queryOptions,
  useQueries,
} from "@tanstack/react-query"
import { userDetailQueryOptions } from "@/features/users/queries"
import { auditLogApi } from "./api"
import { shortId } from "./labels"
import type { AuditListParams } from "./schemas"

export const auditKeys = {
  all: ["audit-log"] as const,
  list: (p: AuditListParams) => ["audit-log", "list", p] as const,
}

export const auditLogQueryOptions = (params: AuditListParams) =>
  queryOptions({
    queryKey: auditKeys.list(params),
    queryFn: ({ signal }) => auditLogApi.list(params, signal),
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
