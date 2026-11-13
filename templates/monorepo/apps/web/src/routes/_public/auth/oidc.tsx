import { Link, createFileRoute, useRouter } from "@tanstack/react-router"
import { useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import { AuthCard } from "@/components/layout/auth-card"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { TotpLoginStep } from "@/features/auth/components/totp-login-step"
import { completeSignIn } from "@/features/auth/session-actions"
import { useCapturedToken } from "@/features/auth/use-captured-token"
import { oidcOutcome } from "@/features/oidc/outcome"
import { providerLabel } from "@/features/oidc/provider-label"

export const Route = createFileRoute("/_public/auth/oidc")({
  component: OidcCallbackPage,
})

/**
 * Where the API's callback sends the browser after a provider: the fragment
 * carries a session, a two-factor step, a linked identity, or why not. It is
 * read once and stripped from the address bar.
 */
function OidcCallbackPage() {
  const router = useRouter()
  const qc = useQueryClient()
  const [pendingToken, setPendingToken] = useState<string>()
  const [refusal, setRefusal] = useState<string>()

  useCapturedToken(
    typeof window === "undefined" ? undefined : window.location.hash,
    () => window.history.replaceState(null, "", window.location.pathname),
    (fragment) => {
      const outcome = oidcOutcome(fragment)
      switch (outcome.kind) {
        case "session":
          void completeSignIn(qc, router, outcome.accessToken)
          break
        case "totp":
          setPendingToken(outcome.pendingToken)
          break
        case "linked":
          toast.add({
            type: "success",
            title: `${providerLabel(outcome.provider)} is linked`,
          })
          void router.navigate({ to: "/settings/security" })
          break
        case "refused":
          setRefusal(outcome.message)
      }
    }
  )

  if (pendingToken) {
    return (
      <TotpLoginStep
        pendingToken={pendingToken}
        onRestart={(message) => {
          setPendingToken(undefined)
          setRefusal(message ?? "Please sign in again.")
        }}
      />
    )
  }

  if (refusal) {
    return (
      <AuthCard
        title="Not signed in"
        description={refusal}
        footer={
          <Link to="/login" className="underline">
            Back to sign in
          </Link>
        }
      />
    )
  }

  return (
    <AuthCard title="Signing you in" description="One moment.">
      <Spinner />
    </AuthCard>
  )
}
