import { Link, createFileRoute, useRouter } from "@tanstack/react-router"
import { useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import { z } from "zod"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { AuthCard } from "@/components/layout/auth-card"
import {
  FormError,
  PasswordField,
  SubmitButton,
  TextField,
} from "@/components/common/form-fields"
import { FieldGroup } from "@/components/ui/field"
import { authApi } from "@/features/auth/api"
import { TotpLoginStep } from "@/features/auth/components/totp-login-step"
import { loginStepFor } from "@/features/auth/login-flow"
import { loginSchema, safeRedirect } from "@/features/auth/schemas"
import {
  completeSignIn,
  redirectIfSignedIn,
} from "@/features/auth/session-actions"
import { useApiForm } from "@/lib/use-api-form"
import { SocialButtons } from "@/features/oauth/components/social-buttons"
import { PasskeyLoginButton } from "@/features/passkeys/components/passkey-login-button"
import { QrLoginDialog } from "@/features/qr-login/components/qr-login-panel"

export const Route = createFileRoute("/_public/login")({
  validateSearch: z.object({
    redirect: z.string().optional().catch(undefined),
    reason: z.string().optional().catch(undefined),
  }),
  beforeLoad: redirectIfSignedIn,
  component: LoginPage,
})

const NOTICES: Record<string, string> = {
  "session-expired": "Your session expired. Please sign in again.",
  "password-changed": "Password changed, please sign in again.",
}

function LoginPage() {
  const search = Route.useSearch()
  const redirect = safeRedirect(search.redirect)
  // Kept in memory only, never in storage: it is a bearer for the 2FA step.
  const [pendingToken, setPendingToken] = useState<string>()
  const [notice, setNotice] = useState<string | undefined>(
    search.reason ? NOTICES[search.reason] : undefined
  )

  if (pendingToken) {
    return (
      <TotpLoginStep
        pendingToken={pendingToken}
        redirect={redirect}
        onRestart={(message) => {
          setPendingToken(undefined)
          setNotice(message)
        }}
      />
    )
  }

  return (
    <PasswordStep
      redirect={redirect}
      notice={notice}
      onDismissNotice={() => setNotice(undefined)}
      onTotpRequired={setPendingToken}
    />
  )
}

function PasswordStep({
  redirect,
  notice,
  onDismissNotice,
  onTotpRequired,
}: {
  redirect?: string
  notice?: string
  onDismissNotice: () => void
  onTotpRequired: (pendingToken: string) => void
}) {
  const router = useRouter()
  const qc = useQueryClient()

  const { form, formError, formErrorStatus } = useApiForm({
    schema: loginSchema,
    defaultValues: { email: "", password: "" },
    request: (value) => authApi.login(value),
    onSuccess: async (response) => {
      const step = loginStepFor(response)
      if (step.kind === "totp") {
        onTotpRequired(step.pendingToken)
        return
      }
      await completeSignIn(qc, router, step.accessToken, redirect)
    },
  })

  const hint = formError?.includes("verified")
    ? "Check your inbox for the verification link, then sign in."
    : formError?.includes("disabled")
      ? "Contact support if you think this is a mistake."
      : formErrorStatus === 401
        ? // Lockout returns the same generic 401, so never try to detect it.
          "Too many attempts? Wait a few minutes or reset your password."
        : undefined

  return (
    <AuthCard
      title="Sign in"
      description="Welcome back."
      footer={
        <>
          No account?{" "}
          <Link to="/register" className="underline">
            Create one
          </Link>
        </>
      }
    >
      {notice && (
        <Alert>
          <AlertDescription>
            {notice}{" "}
            <button className="underline" onClick={onDismissNotice}>
              Dismiss
            </button>
          </AlertDescription>
        </Alert>
      )}
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
            {(field) => <PasswordField field={field} label="Password" />}
          </form.Field>
        </FieldGroup>
        <div aria-live="polite" className="flex flex-col gap-2">
          <FormError message={formError} />
          {hint && (
            <p className="text-xs text-muted-foreground">
              {hint}
              {formErrorStatus === 401 && (
                <>
                  {" "}
                  <Link to="/forgot-password" className="underline">
                    Reset it
                  </Link>
                </>
              )}
            </p>
          )}
        </div>
        <SubmitButton form={form} pendingLabel="Signing in…">
          Sign in
        </SubmitButton>
        <Link
          to="/forgot-password"
          className="text-center text-xs text-muted-foreground underline"
        >
          Forgot password?
        </Link>
      </form>
      <SocialButtons redirect={redirect} mode="signin" />
      <div className="flex flex-col gap-2">
        <PasskeyLoginButton redirect={redirect} />
        <QrLoginDialog redirect={redirect} />
      </div>
    </AuthCard>
  )
}
