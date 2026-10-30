import { useEffect, useRef } from "react"
import {
  Link,
  createFileRoute,
  redirect,
  useRouter,
} from "@tanstack/react-router"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { AuthCard } from "@/components/layout/auth-card"
import { buttonVariants } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { oauthApi } from "@/features/oauth/api"
import { oauthErrorMessage } from "@/features/oauth/messages"
import { oauthKeys, providersQueryOptions } from "@/features/oauth/queries"
import { callbackSearchSchema } from "@/features/oauth/schemas"
import { completeSignIn } from "@/features/auth/session-actions"
import { safeRedirect } from "@/features/auth/schemas"
import { useCapturedToken } from "@/features/auth/use-captured-token"
import { session } from "@/lib/session"

// Every OAuth outcome (sign-in, link, error) lands here; the frontend must
// serve this exact route (FRONTEND_URL/oauth/callback).
export const Route = createFileRoute("/_public/oauth/callback")({
  validateSearch: callbackSearchSchema,
  component: OAuthCallbackPage,
})

function OAuthCallbackPage() {
  const search = Route.useSearch()
  const navigate = Route.useNavigate()
  const router = useRouter()
  const qc = useQueryClient()

  const exchange = useMutation({
    mutationFn: async (code: string) => {
      const token = await oauthApi.exchange(code)
      await completeSignIn(qc, router, token.access_token, search.redirect)
    },
  })

  const code = useCapturedToken(
    search.code,
    () => void navigate({ search: {}, replace: true }),
    (c) => exchange.mutate(c)
  )

  if (search.linked) {
    return <LinkedResult provider={search.linked} redirect={search.redirect} />
  }

  if (search.error) {
    return <ErrorResult reason={search.error} />
  }

  if (code) {
    if (exchange.isError) {
      return <ErrorResult reason={errorReasonFor(exchange.error)} />
    }
    return (
      <AuthCard title="Signing you in…">
        <div role="status" className="flex justify-center py-2">
          <Spinner className="size-5" />
        </div>
      </AuthCard>
    )
  }

  throw redirect({ to: "/login" })
}

function errorReasonFor(error: unknown) {
  const status = (error as { status?: number } | undefined)?.status
  if (status === 403) return "account_disabled"
  return "server_error"
}

function LinkedResult({
  provider,
  redirect: redirectTo,
}: {
  provider: string
  redirect: string | undefined
}) {
  const qc = useQueryClient()
  const navigate = Route.useNavigate()
  const started = useRef(false)

  useEffect(() => {
    if (started.current) return
    started.current = true
    if (!session.hasValidToken()) {
      void navigate({ to: "/login" })
      return
    }
    void qc.invalidateQueries({ queryKey: oauthKeys.identities })
    void qc.invalidateQueries({ queryKey: providersQueryOptions.queryKey })
    toast.add({ type: "success", title: `${provider} connected` })
    void navigate({
      href: safeRedirect(redirectTo) ?? "/settings/account",
      replace: true,
    })
  }, [navigate, provider, qc, redirectTo])

  return (
    <AuthCard title="Connecting…">
      <div role="status" className="flex justify-center py-2">
        <Spinner className="size-5" />
      </div>
    </AuthCard>
  )
}

function ErrorResult({ reason }: { reason: string | undefined }) {
  return (
    <AuthCard title="Sign-in failed" description={oauthErrorMessage(reason)}>
      <div className="flex flex-col gap-2">
        <Link to="/login" className={buttonVariants()}>
          Back to sign in
        </Link>
        {reason === "account_exists_unverified" && (
          <Link
            to="/forgot-password"
            className={buttonVariants({ variant: "outline" })}
          >
            Reset your password
          </Link>
        )}
      </div>
    </AuthCard>
  )
}
