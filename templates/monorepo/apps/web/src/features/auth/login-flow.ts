import { ApiError } from "@/lib/http"
import type { LoginResponse } from "./schemas"

export type LoginStep =
  | { kind: "signed_in"; accessToken: string }
  | { kind: "totp"; pendingToken: string }

/** Tells the two `POST /auth/login` answers apart by the `requires_totp` marker. */
export function loginStepFor(response: LoginResponse): LoginStep {
  if ("requires_totp" in response)
    return { kind: "totp", pendingToken: response.pending_token }
  return { kind: "signed_in", accessToken: response.access_token }
}

/**
 * What to do when `POST /auth/2fa/verify` fails:
 * - retry: 401, a wrong code. The pending token is not spent, stay on the code step.
 * - restart: 400, the pending token is invalid or expired (5 minutes). Back to the password step.
 * - error: anything else (network, 5xx, rate limit): show the message and stay.
 */
export type TotpVerifyOutcome = "retry" | "restart" | "error"

export function totpVerifyOutcome(error: unknown): TotpVerifyOutcome {
  if (error instanceof ApiError) {
    if (error.status === 401) return "retry"
    if (error.status === 400) return "restart"
  }
  return "error"
}
