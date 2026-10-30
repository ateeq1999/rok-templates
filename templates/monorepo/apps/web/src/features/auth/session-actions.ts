import { redirect, useNavigate, useRouter } from "@tanstack/react-router"
import type { AnyRouter } from "@tanstack/react-router"
import { useQueryClient } from "@tanstack/react-query"
import type { QueryClient } from "@tanstack/react-query"
import { session } from "@/lib/session"
import { meQueryOptions } from "@/features/users/queries"
import { ApiError } from "@/lib/http"
import type { User } from "@/features/users/schemas"
import type { RouterContext } from "@/router"
import { safeRedirect } from "./schemas"

/** Where a signed-in user lands: admins on the dashboard, others on their profile. */
export function homeFor(me: Pick<User, "role">) {
  return me.role === "admin" ? ("/admin" as const) : ("/profile" as const)
}

/**
 * Shared by every sign-in path (password, OAuth, passkey, QR): clear any
 * previous user's cache, store the token, load `me`, then go to the
 * requested redirect (same-origin only) or the role's home.
 */
export async function completeSignIn(
  qc: QueryClient,
  router: AnyRouter,
  token: string,
  redirectTo?: string
) {
  qc.clear()
  session.login(token)
  const me = await qc.fetchQuery(meQueryOptions)
  const target = safeRedirect(redirectTo)
  if (target) await router.navigate({ href: target })
  else await router.navigate({ to: homeFor(me) })
}

/**
 * Sign out: forget the token, drop every cached record, go to /login.
 * `reason` selects the notice shown there (e.g. "password-changed").
 */
export function useLogout() {
  const qc = useQueryClient()
  const navigate = useNavigate()
  const router = useRouter()
  return async (reason?: string) => {
    session.clear()
    qc.clear()
    await navigate({ to: "/login", search: reason ? { reason } : {} })
    void router.invalidate()
  }
}

/** For login/register/forgot: signed-in users belong in the app. */
export async function redirectIfSignedIn({
  context,
}: {
  context: RouterContext
}) {
  if (!session.hasValidToken()) return
  try {
    const me = await context.queryClient.ensureQueryData(meQueryOptions)
    throw redirect({ to: homeFor(me) })
  } catch (error) {
    // a rejected token just means the user stays on the public page
    if (error instanceof ApiError) return
    throw error
  }
}

/** Loads `me`, turning a rejected token into a redirect to /login. */
export async function requireMe(
  context: RouterContext,
  location?: { href: string }
) {
  if (!session.hasValidToken()) {
    throw redirect({
      to: "/login",
      search: location ? { redirect: location.href } : {},
    })
  }
  try {
    return await context.queryClient.ensureQueryData(meQueryOptions)
  } catch (error) {
    // A 401 clears the session, and the session-ended handler also clears the
    // query cache, which cancels this very request: the caller then sees a
    // CancelledError instead of the ApiError. "No token any more" covers both.
    if (
      (error instanceof ApiError && error.status === 401) ||
      !session.hasValidToken()
    ) {
      throw redirect({
        to: "/login",
        search: {
          ...(location ? { redirect: location.href } : {}),
          reason: "session-expired",
        },
      })
    }
    throw error
  }
}
