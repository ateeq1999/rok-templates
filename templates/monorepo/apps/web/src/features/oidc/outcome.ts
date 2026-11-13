/**
 * How a sign-in (or a link) with a provider ended, as the API's callback
 * carries it to `/auth/oidc` in the URL's fragment.
 */
export type OidcOutcome =
  | { kind: "session"; accessToken: string }
  | { kind: "totp"; pendingToken: string }
  | { kind: "linked"; provider: string }
  | { kind: "refused"; message: string }

/** Reads `#access_token=…`, `#requires_totp=true&pending_token=…`, `#linked=…` or `#error=…&message=…`. */
export function oidcOutcome(fragment: string): OidcOutcome {
  const params = new URLSearchParams(fragment.replace(/^#/, ""))
  const accessToken = params.get("access_token")
  if (accessToken) return { kind: "session", accessToken }
  const pendingToken = params.get("pending_token")
  if (params.get("requires_totp") === "true" && pendingToken) {
    return { kind: "totp", pendingToken }
  }
  const provider = params.get("linked")
  if (provider) return { kind: "linked", provider }
  return {
    kind: "refused",
    message:
      params.get("message") ?? "The sign-in didn't finish. Please try again.",
  }
}
