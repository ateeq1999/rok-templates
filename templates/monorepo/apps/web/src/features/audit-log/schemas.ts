import { z } from "zod"

export const auditEntrySchema = z.object({
  id: z.string(),
  actor_user_id: z.string(),
  action: z.string(),
  target_user_id: z.string().nullable(),
  /** An object whose keys depend on the action; `null` if a stored row was unreadable. */
  details: z.unknown(),
  created_at: z.string(),
})
export type AuditEntry = z.infer<typeof auditEntrySchema>

export const auditPageSchema = z.object({
  items: z.array(auditEntrySchema),
  page: z.number(),
  per_page: z.number(),
  total: z.number(),
})

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
