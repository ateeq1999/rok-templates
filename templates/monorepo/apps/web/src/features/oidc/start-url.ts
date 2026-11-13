import { apiUrl } from "@/lib/client"

/**
 * Where the browser goes to sign in with `provider`:
 * `GET /api/v1/auth/oidc/{provider}/start`, which redirects to the provider
 * and, after its callback, back to this App's `/auth/oidc`.
 *
 * `start` answers only a redirect, so the released `rok` leaves it out of the
 * contract and the generated client. After the next rok release, the client
 * generates a URL builder for it (`oidcStartUrl`): use that and delete this
 * file.
 */
export function signInStartUrl(provider: string): string {
  return apiUrl(`/api/v1/auth/oidc/${encodeURIComponent(provider)}/start`)
}
