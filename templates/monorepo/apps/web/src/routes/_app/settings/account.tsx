import { createFileRoute } from "@tanstack/react-router"
import { useSuspenseQuery } from "@tanstack/react-query"
import { useState } from "react"
import {
  FormError,
  PasswordField,
  SubmitButton,
  TextField,
} from "@/components/common/form-fields"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { FieldGroup } from "@/components/ui/field"
import { authApi } from "@/features/auth/api"
import { changeEmailSchema } from "@/features/auth/schemas"
import { ConnectedAccounts } from "@/features/oauth/components/connected-accounts"
import { meQueryOptions } from "@/features/users/queries"
import { useApiForm } from "@/lib/use-api-form"

export const Route = createFileRoute("/_app/settings/account")({
  component: AccountSettings,
})

function AccountSettings() {
  const { data: me } = useSuspenseQuery(meQueryOptions)
  const [pendingEmail, setPendingEmail] = useState<string>()

  const { form, formError } = useApiForm({
    schema: changeEmailSchema,
    defaultValues: { new_email: "", current_password: "" },
    conflictField: "new_email",
    request: (value) => authApi.changeEmail(value),
    onSuccess: (_r, value) => {
      setPendingEmail(value.new_email)
      form.reset()
    },
  })

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle>Email</CardTitle>
          <CardDescription className="flex items-center gap-2">
            {me.email}
            <Badge variant={me.email_verified ? "secondary" : "destructive"}>
              {me.email_verified ? "Verified" : "Unverified"}
            </Badge>
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {pendingEmail && (
            <Alert role="status">
              <AlertDescription>
                We sent a confirmation link to {pendingEmail}. Your address
                changes after you confirm. A notice was sent to your current
                address.
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
              <form.Field name="new_email">
                {(field) => (
                  <TextField
                    field={field}
                    label="New email"
                    type="email"
                    autoComplete="email"
                  />
                )}
              </form.Field>
              <form.Field name="current_password">
                {(field) => (
                  <PasswordField field={field} label="Current password" />
                )}
              </form.Field>
            </FieldGroup>
            <div aria-live="polite">
              <FormError message={formError} />
            </div>
            <div>
              <SubmitButton form={form} pendingLabel="Sending…">
                Change email
              </SubmitButton>
            </div>
          </form>
        </CardContent>
      </Card>
      <ConnectedAccounts />
    </div>
  )
}
