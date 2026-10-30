import { http } from "@/lib/http"
import { apiKeysSchema, createdApiKeySchema, expiryDays } from "./schemas"
import type { ExpiryOption, Scope } from "./schemas"

export const apiKeysApi = {
  list: async () => apiKeysSchema.parse(await http("/api/v1/api-keys")),

  create: async (body: { name: string; scope: Scope; expiry: ExpiryOption }) =>
    createdApiKeySchema.parse(
      await http("/api/v1/api-keys", {
        method: "POST",
        body: {
          name: body.name,
          scope: body.scope,
          expires_in_days: expiryDays(body.expiry),
        },
      })
    ),

  revoke: (id: string) =>
    http<void>(`/api/v1/api-keys/${id}`, { method: "DELETE" }),
}
