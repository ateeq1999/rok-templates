import { z } from "zod"

export const scopeSchema = z.enum(["read", "write"])
export type Scope = z.infer<typeof scopeSchema>

export const apiKeySchema = z.object({
  id: z.string(),
  name: z.string(),
  key_prefix: z.string(),
  scope: scopeSchema,
  expires_at: z.string().nullable(),
  last_used_at: z.string().nullable(),
  created_at: z.string(),
})
export type ApiKey = z.infer<typeof apiKeySchema>

export const apiKeysSchema = z.array(apiKeySchema)

export const createdApiKeySchema = apiKeySchema.extend({ key: z.string() })
export type CreatedApiKey = z.infer<typeof createdApiKeySchema>

export const expiryOptions = ["never", "30", "90", "365"] as const
export type ExpiryOption = (typeof expiryOptions)[number]

export const createApiKeyFormSchema = z.object({
  name: z
    .string()
    .trim()
    .min(1, "Required")
    .max(100, "Must be at most 100 characters"),
  scope: scopeSchema,
  expiry: z.enum(expiryOptions),
})

export function expiryDays(expiry: ExpiryOption): number | undefined {
  return expiry === "never" ? undefined : Number(expiry)
}

export function isExpired(
  key: Pick<ApiKey, "expires_at">,
  now = new Date()
): boolean {
  return key.expires_at !== null && new Date(key.expires_at) <= now
}
