import { http } from "@/lib/http"
import { tokenResponseSchema } from "@/features/auth/schemas"
import {
  identitiesSchema,
  linkResponseSchema,
  providersSchema,
} from "./schemas"
import type { OAuthProvider } from "./schemas"

export const oauthApi = {
  providers: async () =>
    providersSchema.parse(
      await http("/api/v1/auth/oauth/providers", { skipExpire: true })
    ),

  identities: async () =>
    identitiesSchema.parse(await http("/api/v1/auth/oauth/identities")),

  link: async (provider: OAuthProvider) =>
    linkResponseSchema.parse(
      await http(`/api/v1/auth/oauth/${provider}/link`, { method: "POST" })
    ),

  unlink: (provider: OAuthProvider) =>
    http<void>(`/api/v1/auth/oauth/${provider}`, { method: "DELETE" }),

  exchange: async (code: string) =>
    tokenResponseSchema.parse(
      await http("/api/v1/auth/oauth/exchange", {
        method: "POST",
        body: { code },
        skipExpire: true,
      })
    ),
}
