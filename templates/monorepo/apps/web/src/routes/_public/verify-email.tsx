import { Link, createFileRoute } from "@tanstack/react-router"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { z } from "zod"
import { AuthCard } from "@/components/layout/auth-card"
import { buttonVariants } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import { authApi } from "@/features/auth/api"
import { useCapturedToken } from "@/features/auth/use-captured-token"
import { userKeys } from "@/features/users/queries"
import { session } from "@/lib/session"

export const Route = createFileRoute("/_public/verify-email")({
  validateSearch: z.object({ token: z.string().optional().catch(undefined) }),
  component: VerifyEmailPage,
})

function VerifyEmailPage() {
  const { token: urlToken } = Route.useSearch()
  const navigate = Route.useNavigate()
  const qc = useQueryClient()

  const verify = useMutation({
    mutationFn: authApi.verifyEmail,
    onSuccess: () => qc.invalidateQueries({ queryKey: userKeys.me }),
  })

  const token = useCapturedToken(
    urlToken,
    () => void navigate({ search: {}, replace: true }),
    (t) => verify.mutate({ token: t })
  )

  const continueTo = session.hasValidToken() ? "/profile" : "/login"

  if (!token || verify.isError) {
    return (
      <AuthCard
        title="Link invalid or expired"
        description="This verification link was already used or has expired. Sign in and request a new one from your profile."
      >
        <Link to="/login" className={buttonVariants()}>
          Go to sign in
        </Link>
      </AuthCard>
    )
  }

  if (verify.isSuccess) {
    return (
      <AuthCard
        title="Email verified"
        description="Thanks, your address is confirmed."
      >
        <Link to={continueTo} className={buttonVariants()}>
          Continue
        </Link>
      </AuthCard>
    )
  }

  return (
    <AuthCard title="Verifying your email…">
      <div role="status" className="flex justify-center py-2">
        <Spinner className="size-5" />
      </div>
    </AuthCard>
  )
}
