import { http } from "@/lib/http"
import { totpEnabledSchema, totpSetupSchema } from "./schemas"

// Every call is skipExpire: a wrong code or password is a normal 401 here, not
// an expired session. All of them need an interactive sign-in (API keys get 403).
export const twoFactorApi = {
  setup: async () =>
    totpSetupSchema.parse(
      await http("/api/v1/auth/2fa/setup", { method: "POST", skipExpire: true })
    ),

  /** Returns the recovery codes. This is the only time they are ever shown. */
  enable: async (body: { code: string }) =>
    totpEnabledSchema.parse(
      await http("/api/v1/auth/2fa/enable", {
        method: "POST",
        body,
        skipExpire: true,
      })
    ),

  disable: (body: { current_password: string }) =>
    http<{ message: string }>("/api/v1/auth/2fa/disable", {
      method: "POST",
      body,
      skipExpire: true,
    }),
}
