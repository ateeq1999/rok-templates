import { Link, createFileRoute } from "@tanstack/react-router"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { z } from "zod"
import { AuthCard } from "@/components/layout/auth-card"
import { buttonVariants } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import { authApi } from "@/features/auth/api"
import { useCapturedToken } from "@/features/auth/use-captured-token"
import { userKeys } from "@/features/users/queries"
import { ApiError } from "@/lib/http"
import { session } from "@/lib/session"

export const Route = createFileRoute("/_public/confirm-email-change")({
  validateSearch: z.object({ token: z.string().optional().catch(undefined) }),
  component: ConfirmEmailChangePage,
})

function ConfirmEmailChangePage() {
  const { token: urlToken } = Route.useSearch()
  const navigate = Route.useNavigate()
  const qc = useQueryClient()

  const confirm = useMutation({
    mutationFn: authApi.confirmEmailChange,
    onSuccess: () => qc.invalidateQueries({ queryKey: userKeys.me }),
  })

  const token = useCapturedToken(
    urlToken,
    () => void navigate({ search: {}, replace: true }),
    (t) => confirm.mutate({ token: t })
  )

  const signedIn = session.hasValidToken()

  if (confirm.error instanceof ApiError && confirm.error.status === 409) {
    return (
      <AuthCard
        title="Address unavailable"
        description="That address was taken in the meantime. Choose a different one in your settings."
      >
        <Link
          to={signedIn ? "/settings/account" : "/login"}
          className={buttonVariants()}
        >
          {signedIn ? "Open settings" : "Go to sign in"}
        </Link>
      </AuthCard>
    )
  }

  if (!token || confirm.isError) {
    return (
      <AuthCard
        title="Link invalid or expired"
        description="This link was already used or has expired. Request the change again from your settings."
      >
        <Link to="/login" className={buttonVariants()}>
          Go to sign in
        </Link>
      </AuthCard>
    )
  }

  if (confirm.isSuccess) {
    return (
      <AuthCard
        title="Email changed"
        description="Your email address has been updated."
      >
        <Link
          to={signedIn ? "/settings/account" : "/login"}
          className={buttonVariants()}
        >
          {signedIn ? "Open settings" : "Go to sign in"}
        </Link>
      </AuthCard>
    )
  }

  return (
    <AuthCard title="Confirming your new email…">
      <div role="status" className="flex justify-center py-2">
        <Spinner className="size-5" />
      </div>
    </AuthCard>
  )
}
