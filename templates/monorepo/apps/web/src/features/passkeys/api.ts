import { http } from "@/lib/http"
import { tokenResponseSchema } from "@/features/auth/schemas"
import { beginResponseSchema, passkeySchema, passkeysSchema } from "./schemas"

export const passkeysApi = {
  list: async () => passkeysSchema.parse(await http("/api/v1/auth/passkeys")),

  remove: (id: string) =>
    http<void>(`/api/v1/auth/passkeys/${id}`, { method: "DELETE" }),

  registerBegin: async () =>
    beginResponseSchema.parse(
      await http("/api/v1/auth/passkeys/register/begin", { method: "POST" })
    ),

  registerFinish: async (body: {
    challenge_id: string
    name?: string
    credential: unknown
  }) =>
    passkeySchema.parse(
      await http("/api/v1/auth/passkeys/register/finish", {
        method: "POST",
        body,
      })
    ),

  loginBegin: async () =>
    beginResponseSchema.parse(
      await http("/api/v1/auth/passkeys/login/begin", {
        method: "POST",
        skipExpire: true,
      })
    ),

  loginFinish: async (body: { challenge_id: string; credential: unknown }) =>
    tokenResponseSchema.parse(
      await http("/api/v1/auth/passkeys/login/finish", {
        method: "POST",
        body,
        skipExpire: true,
      })
    ),
}
