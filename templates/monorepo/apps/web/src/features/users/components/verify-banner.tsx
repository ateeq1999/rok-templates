import { useMutation } from "@tanstack/react-query"
import {
  Alert,
  AlertAction,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { authApi } from "@/features/auth/api"
import { ApiError } from "@/lib/http"

export function useResendVerification() {
  return useMutation({
    mutationFn: authApi.resendVerification,
    onSuccess: () =>
      toast.add({
        type: "success",
        title: "Verification email sent",
        description: "Check your inbox for the link.",
      }),
    onError: (error) => {
      if (error instanceof ApiError && error.status === 429) {
        toast.add({
          type: "error",
          title: "Too many requests",
          description: "Try again in a minute.",
        })
      } else if (
        error instanceof ApiError &&
        error.status < 500 &&
        error.status > 0
      ) {
        toast.add({ type: "error", title: error.message })
      }
    },
  })
}

export function VerifyEmailBanner() {
  const resend = useResendVerification()
  return (
    <Alert>
      <AlertTitle>Verify your email</AlertTitle>
      <AlertDescription>
        Your address is not verified yet. Use the link we emailed you, or
        request a new one.
      </AlertDescription>
      <AlertAction>
        <Button
          size="sm"
          variant="outline"
          disabled={resend.isPending}
          onClick={() => resend.mutate()}
        >
          {resend.isPending && <Spinner />}
          Resend link
        </Button>
      </AlertAction>
    </Alert>
  )
}
