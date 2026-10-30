import { useState } from "react"
import { useRouter } from "@tanstack/react-router"
import { useQueryClient } from "@tanstack/react-query"
import { AuthCard } from "@/components/layout/auth-card"
import {
  FormError,
  SubmitButton,
  TextField,
} from "@/components/common/form-fields"
import { Button } from "@/components/ui/button"
import { FieldGroup } from "@/components/ui/field"
import { authApi } from "@/features/auth/api"
import { totpVerifyOutcome } from "@/features/auth/login-flow"
import { totpCodeFormSchema } from "@/features/auth/schemas"
import type { TotpCodeMode } from "@/features/auth/schemas"
import { completeSignIn } from "@/features/auth/session-actions"
import { ApiError } from "@/lib/http"
import { useApiForm } from "@/lib/use-api-form"

/**
 * Second sign-in step for accounts with two-factor on. The pending token lives
 * only in the parent's memory (never storage). A wrong code (401) leaves the
 * token valid so the user can retry here; a dead token (400) sends them back to
 * the password step.
 */
export function TotpLoginStep({
  pendingToken,
  redirect,
  onRestart,
}: {
  pendingToken: string
  redirect?: string
  /** Back to the password step, optionally with a message explaining why. */
  onRestart: (message?: string) => void
}) {
  const [mode, setMode] = useState<TotpCodeMode>("authenticator")

  return (
    <AuthCard
      title="Two-factor authentication"
      description={
        mode === "authenticator"
          ? "Enter the 6-digit code from your authenticator app."
          : "Enter one of your recovery codes."
      }
      footer={
        <button type="button" className="underline" onClick={() => onRestart()}>
          Back to sign in
        </button>
      }
    >
      {/* keyed by mode so switching starts a fresh form with the right schema */}
      <CodeForm
        key={mode}
        mode={mode}
        pendingToken={pendingToken}
        redirect={redirect}
        onRestart={onRestart}
      />
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onClick={() =>
          setMode((m) => (m === "authenticator" ? "recovery" : "authenticator"))
        }
      >
        {mode === "authenticator"
          ? "Use a recovery code instead"
          : "Use an authenticator code instead"}
      </Button>
    </AuthCard>
  )
}

function CodeForm({
  mode,
  pendingToken,
  redirect,
  onRestart,
}: {
  mode: TotpCodeMode
  pendingToken: string
  redirect?: string
  onRestart: (message?: string) => void
}) {
  const router = useRouter()
  const qc = useQueryClient()

  const { form, formError } = useApiForm({
    schema: totpCodeFormSchema(mode),
    defaultValues: { code: "" },
    request: async ({ code }) => {
      try {
        return await authApi.verifyTotp({ pending_token: pendingToken, code })
      } catch (error) {
        if (totpVerifyOutcome(error) === "restart" && error instanceof ApiError)
          onRestart(
            `Sign-in timed out: ${error.message}. Please enter your password again.`
          )
        throw error
      }
    },
    onSuccess: async (token) => {
      await completeSignIn(qc, router, token.access_token, redirect)
    },
  })

  return (
    <form
      className="flex flex-col gap-4"
      noValidate
      onSubmit={(e) => {
        e.preventDefault()
        void form.handleSubmit()
      }}
    >
      <FieldGroup>
        <form.Field name="code">
          {(field) =>
            mode === "authenticator" ? (
              <TextField
                field={field}
                label="Authenticator code"
                inputMode="numeric"
                autoComplete="one-time-code"
                maxLength={6}
                autoFocus
              />
            ) : (
              <TextField
                field={field}
                label="Recovery code"
                autoComplete="off"
                maxLength={11}
                placeholder="XXXXX-XXXXX"
                autoFocus
              />
            )
          }
        </form.Field>
      </FieldGroup>
      <div aria-live="polite">
        <FormError message={formError} />
      </div>
      <SubmitButton form={form} pendingLabel="Verifying…">
        Verify
      </SubmitButton>
    </form>
  )
}
