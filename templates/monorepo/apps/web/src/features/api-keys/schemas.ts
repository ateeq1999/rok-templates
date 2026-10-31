import { z } from "zod"
import type { ApiKeyResponse, CreatedApiKeyResponse } from "@/lib/client"

export const scopeSchema = z.enum(["read", "write"])
export type Scope = z.infer<typeof scopeSchema>

/** A key as the API lists it (the generated client's `ApiKeyResponse`). */
export type ApiKey = ApiKeyResponse

/** A key as it is created: with its secret, shown once. */
export type CreatedApiKey = CreatedApiKeyResponse

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
  return key.expires_at != null && new Date(key.expires_at) <= now
}
