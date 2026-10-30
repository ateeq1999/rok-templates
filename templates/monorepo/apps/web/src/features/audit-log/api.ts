import { http } from "@/lib/http"
import { auditPageSchema } from "./schemas"
import type { AuditListParams } from "./schemas"

export const auditLogApi = {
  /** Admin only (403 otherwise). Newest first. */
  list: async (params: AuditListParams, signal?: AbortSignal) =>
    auditPageSchema.parse(
      await http("/api/v1/audit-log", { query: params, signal })
    ),
}
