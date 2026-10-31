import { z } from "zod"
import type { AuditLogEntryResponse } from "@/lib/client"

/**
 * An audit-log row (the generated client's `AuditLogEntryResponse`). Its
 * `details` is an object whose keys depend on the action; `null` if a
 * stored row was unreadable.
 */
export type AuditEntry = AuditLogEntryResponse

export const auditSearchSchema = z.object({
  page: z.number().int().min(1).catch(1).default(1),
  per_page: z
    .number()
    .int()
    .refine((n) => [20, 50, 100].includes(n))
    .catch(20)
    .default(20),
})
export type AuditListParams = z.infer<typeof auditSearchSchema>
