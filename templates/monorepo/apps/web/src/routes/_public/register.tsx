import { Link, createFileRoute } from "@tanstack/react-router"
import { useState } from "react"
import { AuthCard } from "@/components/layout/auth-card"
import {
  FormError,
  PasswordField,
  SubmitButton,
  TextField,
} from "@/components/common/form-fields"
import { buttonVariants } from "@/components/ui/button"
import { FieldGroup } from "@/components/ui/field"
import { authApi } from "@/features/auth/api"
import { registerSchema } from "@/features/auth/schemas"
import { redirectIfSignedIn } from "@/features/auth/session-actions"
import { PasswordStrengthMeter } from "@/features/auth/components/password-strength-meter"
import { SocialButtons } from "@/features/oauth/components/social-buttons"
import { useApiForm } from "@/lib/use-api-form"

export const Route = createFileRoute("/_public/register")({
  beforeLoad: redirectIfSignedIn,
  component: RegisterPage,
})

function RegisterPage() {
  const [sentTo, setSentTo] = useState<string>()

  const { form, formError } = useApiForm({
    schema: registerSchema,
    defaultValues: { email: "", password: "", confirm: "" },
    conflictField: "email",
    badRequestField: "password",
    request: ({ email, password }) => authApi.register({ email, password }),
    onSuccess: (_user, value) => setSentTo(value.email),
  })

  if (sentTo) {
    return (
      <AuthCard
        title="Check your email"
        description={`We sent a verification link to ${sentTo}. Open it to activate your account.`}
      >
        <Link to="/login" className={buttonVariants()}>
          Go to sign in
        </Link>
      </AuthCard>
    )
  }

  return (
    <AuthCard
      title="Create account"
      footer={
        <>
          Already registered?{" "}
          <Link to="/login" className="underline">
            Sign in
          </Link>
        </>
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
          <form.Field name="password">
            {(field) => (
              <PasswordField
                field={field}
                label="Password"
                autoComplete="new-password"
                below={
                  <PasswordStrengthMeter
                    password={field.state.value}
                    knownInputs={[field.form.state.values.email]}
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
        <SubmitButton form={form} pendingLabel="Creating…">
          Create account
        </SubmitButton>
      </form>
      <SocialButtons mode="signup" />
    </AuthCard>
  )
}
