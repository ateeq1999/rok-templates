import { Link, createFileRoute } from "@tanstack/react-router"
import { useState } from "react"
import { z } from "zod"
import { AuthCard } from "@/components/layout/auth-card"
import {
  FormError,
  PasswordField,
  SubmitButton,
} from "@/components/common/form-fields"
import { buttonVariants } from "@/components/ui/button"
import { FieldGroup } from "@/components/ui/field"
import { authApi } from "@/features/auth/api"
import { resetSchema } from "@/features/auth/schemas"
import { PasswordStrengthMeter } from "@/features/auth/components/password-strength-meter"
import { isLinkProblem } from "@/features/auth/errors"
import { useCapturedToken } from "@/features/auth/use-captured-token"
import { ApiError } from "@/lib/http"
import { useApiForm } from "@/lib/use-api-form"

// Serves both password-reset and invitation links.
export const Route = createFileRoute("/_public/reset-password")({
  validateSearch: z.object({ token: z.string().optional().catch(undefined) }),
  component: ResetPasswordPage,
})

function ResetPasswordPage() {
  const { token: urlToken } = Route.useSearch()
  const navigate = Route.useNavigate()
  const [done, setDone] = useState(false)
  const [linkInvalid, setLinkInvalid] = useState(false)

  const token = useCapturedToken(
    urlToken,
    () => void navigate({ search: {}, replace: true }),
    () => {}
  )

  const { form, formError } = useApiForm({
    schema: resetSchema,
    defaultValues: { password: "", confirm: "" },
    fieldMap: { new_password: "password" },
    // A password-policy rejection is also a 400, so tell it apart from a dead
    // link by the server's message.
    badRequestField: (message) =>
      isLinkProblem(message) ? undefined : "password",
    request: async ({ password }) => {
      try {
        await authApi.resetPassword({ token: token!, new_password: password })
      } catch (error) {
        if (
          error instanceof ApiError &&
          error.status === 400 &&
          isLinkProblem(error.message)
        )
          setLinkInvalid(true)
        throw error
      }
    },
    onSuccess: () => setDone(true),
  })

  if (!token || linkInvalid) {
    return (
      <AuthCard
        title="Link invalid or expired"
        description="This link cannot be used. Request a new one to continue."
      >
        <Link to="/forgot-password" className={buttonVariants()}>
          Request a new link
        </Link>
      </AuthCard>
    )
  }

  if (done) {
    return (
      <AuthCard
        title="Password updated"
        description="You can now sign in with your new password."
      >
        <Link to="/login" className={buttonVariants()}>
          Sign in
        </Link>
      </AuthCard>
    )
  }

  return (
    <AuthCard title="Choose a new password">
      <form
        className="flex flex-col gap-4"
        noValidate
        onSubmit={(e) => {
          e.preventDefault()
          void form.handleSubmit()
        }}
      >
        <FieldGroup>
          <form.Field name="password">
            {(field) => (
              <PasswordField
                field={field}
                label="New password"
                autoComplete="new-password"
                below={
                  <PasswordStrengthMeter
                    password={field.state.value}
                    weakNote="A password the server rejects can use up this link. Pick a stronger one."
                  />
                }
                description="8-128 characters."
              />
            )}
          </form.Field>
          <form.Field name="confirm">
            {(field) => (
              <PasswordField
                field={field}
                label="Confirm password"
                autoComplete="new-password"
              />
            )}
          </form.Field>
        </FieldGroup>
        <div aria-live="polite">
          <FormError message={formError} />
        </div>
        <SubmitButton form={form} pendingLabel="Saving…">
          Set password
        </SubmitButton>
      </form>
    </AuthCard>
  )
}
