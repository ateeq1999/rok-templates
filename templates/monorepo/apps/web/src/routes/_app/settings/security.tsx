import { useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useMutation, useSuspenseQuery } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { QrCodeIcon } from "@hugeicons/core-free-icons"
import {
  FormError,
  PasswordField,
  SubmitButton,
} from "@/components/common/form-fields"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { FieldGroup } from "@/components/ui/field"
import { Spinner } from "@/components/ui/spinner"
import { authApi } from "@/features/auth/api"
import { changePasswordSchema } from "@/features/auth/schemas"
import { useLogout } from "@/features/auth/session-actions"
import { PasswordStrengthMeter } from "@/features/auth/components/password-strength-meter"
import { isWrongCurrentPassword } from "@/features/auth/errors"
import { TwoFactorCard } from "@/features/two-factor/components/two-factor-card"
import { PasskeysCard } from "@/features/passkeys/components/passkeys-card"
import { meQueryOptions } from "@/features/users/queries"
import { useApiForm } from "@/lib/use-api-form"

export const Route = createFileRoute("/_app/settings/security")({
  component: SecuritySettings,
})

function SecuritySettings() {
  const { data: me } = useSuspenseQuery(meQueryOptions)
  const logout = useLogout()

  return (
    <div className="flex flex-col gap-4">
      {me.has_password ? (
        <PasswordCard knownInputs={[me.email, me.display_name]} />
      ) : (
        <SetPasswordCard email={me.email} />
      )}
      <TwoFactorCard
        enabled={me.totp_enabled}
        hasPassword={me.has_password}
        account={me.email}
      />
      <PasskeysCard />

      <Card>
        <CardHeader>
          <CardTitle>Approve a sign-in on another device</CardTitle>
          <CardDescription>
            Scan the QR code shown on the other device with your phone's camera
            to open the approval page here.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="flex items-center gap-2 text-sm text-muted-foreground">
            <HugeiconsIcon icon={QrCodeIcon} className="size-4" />
            You will be asked to confirm a 4-digit code and can approve or
            reject the request.
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Sign out</CardTitle>
          <CardDescription>End your session on this device.</CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="outline" onClick={() => void logout()}>
            Sign out
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}

function PasswordCard({
  knownInputs,
}: {
  knownInputs: (string | null | undefined)[]
}) {
  const logout = useLogout()
  const { form, formError } = useApiForm({
    schema: changePasswordSchema,
    defaultValues: { current_password: "", new_password: "", confirm: "" },
    // 400 covers both a wrong current password and a password-policy rejection.
    badRequestField: (message) =>
      isWrongCurrentPassword(message) ? "current_password" : "new_password",
    request: ({ current_password, new_password }) =>
      authApi.changePassword({ current_password, new_password }),
    // The server revokes every token on a password change, including this
    // one, so sign out here instead of waiting for the next call to 401.
    onSuccess: async () => {
      await logout("password-changed")
    },
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Password</CardTitle>
        <CardDescription>
          You stay signed in on this device. Other devices stay signed in until
          their session expires (up to 1 hour).
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className="flex flex-col gap-4"
          noValidate
          onSubmit={(e) => {
            e.preventDefault()
            void form.handleSubmit()
          }}
        >
          <FieldGroup>
            <form.Field name="current_password">
              {(field) => (
                <PasswordField field={field} label="Current password" />
              )}
            </form.Field>
            <form.Field name="new_password">
              {(field) => (
                <PasswordField
                  field={field}
                  label="New password"
                  autoComplete="new-password"
                  below={
                    <PasswordStrengthMeter
                      password={field.state.value}
                      knownInputs={knownInputs}
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
                  label="Confirm new password"
                  autoComplete="new-password"
                />
              )}
            </form.Field>
          </FieldGroup>
          <div aria-live="polite">
            <FormError message={formError} />
          </div>
          <div>
            <SubmitButton form={form} pendingLabel="Saving…">
              Change password
            </SubmitButton>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}

function SetPasswordCard({ email }: { email: string }) {
  const [sent, setSent] = useState(false)
  const request = useMutation({
    mutationFn: () => authApi.forgotPassword({ email }),
    onSuccess: () => setSent(true),
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Password</CardTitle>
        <CardDescription>
          You do not have a password yet. You sign in with Google, GitHub or a
          passkey.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {sent ? (
          <Alert role="status">
            <AlertDescription>We sent a link to {email}.</AlertDescription>
          </Alert>
        ) : (
          <Button
            variant="outline"
            disabled={request.isPending}
            onClick={() => request.mutate()}
          >
            {request.isPending && <Spinner />}
            Set a password by email
          </Button>
        )}
      </CardContent>
    </Card>
  )
}
