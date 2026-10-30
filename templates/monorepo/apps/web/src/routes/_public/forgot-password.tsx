import { Link, createFileRoute } from "@tanstack/react-router"
import { useState } from "react"
import { AuthCard } from "@/components/layout/auth-card"
import {
  FormError,
  SubmitButton,
  TextField,
} from "@/components/common/form-fields"
import { FieldGroup } from "@/components/ui/field"
import { authApi } from "@/features/auth/api"
import { forgotSchema } from "@/features/auth/schemas"
import { redirectIfSignedIn } from "@/features/auth/session-actions"
import { ApiError } from "@/lib/http"
import { useApiForm } from "@/lib/use-api-form"

export const Route = createFileRoute("/_public/forgot-password")({
  beforeLoad: redirectIfSignedIn,
  component: ForgotPasswordPage,
})

function ForgotPasswordPage() {
  const [done, setDone] = useState(false)

  const { form, formError } = useApiForm({
    schema: forgotSchema,
    defaultValues: { email: "" },
    request: async (value) => {
      try {
        await authApi.forgotPassword(value)
      } catch (error) {
        // Anti-enumeration: only rate limits and outages are surfaced.
        const surfaced =
          error instanceof ApiError &&
          (error.status === 429 || error.status >= 500 || error.status === 0)
        if (surfaced) throw error
      }
    },
    onSuccess: () => setDone(true),
  })

  if (done) {
    return (
      <AuthCard
        title="Check your email"
        description="If that address is registered, we sent a link to reset your password."
        footer={
          <Link to="/login" className="underline">
            Back to sign in
          </Link>
        }
      />
    )
  }

  return (
    <AuthCard
      title="Forgot password"
      description="Enter your email and we will send a reset link."
      footer={
        <Link to="/login" className="underline">
          Back to sign in
        </Link>
      }
    >
      <form
        className="flex flex-col gap-4"
        noValidate
        onSubmit={(e) => {
          e.preventDefault()
          void form.handleSubmit()
        }}
      >
        <FieldGroup>
          <form.Field name="email">
            {(field) => (
              <TextField
                field={field}
                label="Email"
                type="email"
                autoComplete="email"
              />
            )}
          </form.Field>
        </FieldGroup>
        <div aria-live="polite">
          <FormError message={formError} />
        </div>
        <SubmitButton form={form} pendingLabel="Sending…">
          Send reset link
        </SubmitButton>
      </form>
    </AuthCard>
  )
}
