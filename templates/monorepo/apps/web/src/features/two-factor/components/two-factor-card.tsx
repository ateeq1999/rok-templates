import { useState } from "react"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { Copy01Icon, Download01Icon } from "@hugeicons/core-free-icons"
import {
  FormError,
  PasswordField,
  SubmitButton,
  TextField,
} from "@/components/common/form-fields"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { FieldGroup } from "@/components/ui/field"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { twoFactorApi } from "@/features/two-factor/api"
import {
  formatRecoveryCodesFile,
  recoveryCodesFilename,
} from "@/features/two-factor/recovery-codes"
import {
  disableFormSchema,
  enableFormSchema,
} from "@/features/two-factor/schemas"
import type { TotpSetup } from "@/features/two-factor/schemas"
import { userKeys } from "@/features/users/queries"
import { copyToClipboard, downloadBlob } from "@/lib/clipboard"
import { ApiError } from "@/lib/http"
import { useApiForm } from "@/lib/use-api-form"

/**
 * Settings > Security. `enabled` comes from `me.totp_enabled`; every action
 * ends by refetching `me`, so the card always matches the server.
 */
export function TwoFactorCard({
  enabled,
  hasPassword,
  account,
}: {
  enabled: boolean
  /** Two-factor gates password sign-in only, so it needs a password to mean anything. */
  hasPassword: boolean
  account: string
}) {
  const qc = useQueryClient()
  const [setup, setSetup] = useState<TotpSetup>()
  // Held in component state only: never in the query cache or storage.
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>()
  const [disableOpen, setDisableOpen] = useState(false)

  const refetchMe = () => qc.refetchQueries({ queryKey: userKeys.me })

  const startSetup = useMutation({
    mutationFn: twoFactorApi.setup,
    onSuccess: setSetup,
    onError: (error) => {
      if (error instanceof ApiError) {
        toast.add({ type: "error", title: error.message })
        // e.g. "already enabled": bring the card back in line with the server
        void refetchMe()
      }
    },
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Two-factor authentication
          {enabled && <Badge variant="secondary">On</Badge>}
        </CardTitle>
        <CardDescription>
          {enabled
            ? "Two-factor is on. Signing in with your password also needs a code from your authenticator app."
            : "Add a second step to password sign-in with a code from an authenticator app."}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {!hasPassword ? (
          <p className="text-sm text-muted-foreground">
            Two-factor protects password sign-in. Set a password first to use
            it.
          </p>
        ) : enabled ? (
          <div>
            <Button variant="outline" onClick={() => setDisableOpen(true)}>
              Turn off
            </Button>
          </div>
        ) : (
          <div>
            <Button
              disabled={startSetup.isPending}
              onClick={() => startSetup.mutate()}
            >
              {startSetup.isPending && <Spinner />}
              Set up two-factor
            </Button>
          </div>
        )}
      </CardContent>

      {setup && (
        <SetupDialog
          setup={setup}
          onCancel={() => setSetup(undefined)}
          onReset={(message) => {
            setSetup(undefined)
            toast.add({ type: "error", title: message })
            void refetchMe()
          }}
          onEnabled={(codes) => {
            setSetup(undefined)
            setRecoveryCodes(codes)
            void refetchMe()
          }}
        />
      )}
      {recoveryCodes && (
        <RecoveryCodesDialog
          codes={recoveryCodes}
          account={account}
          onDone={() => setRecoveryCodes(undefined)}
        />
      )}
      {disableOpen && (
        <DisableDialog
          onClose={() => setDisableOpen(false)}
          onDisabled={() => {
            setDisableOpen(false)
            toast.add({ type: "success", title: "Two-factor turned off" })
            void refetchMe()
          }}
        />
      )}
    </Card>
  )
}

function SetupDialog({
  setup,
  onCancel,
  onReset,
  onEnabled,
}: {
  setup: TotpSetup
  onCancel: () => void
  /** 400: "start setup first" / "already enabled". Message + resync with the server. */
  onReset: (message: string) => void
  onEnabled: (recoveryCodes: string[]) => void
}) {
  const [copied, setCopied] = useState(false)

  const { form, formError } = useApiForm({
    schema: enableFormSchema,
    defaultValues: { code: "" },
    request: async (value) => {
      try {
        return await twoFactorApi.enable(value)
      } catch (error) {
        if (error instanceof ApiError && error.status === 400)
          onReset(error.message)
        throw error
      }
    },
    onSuccess: (result) => onEnabled(result.recovery_codes),
  })

  return (
    <Dialog open onOpenChange={(open) => !open && onCancel()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Set up two-factor</DialogTitle>
          <DialogDescription>
            Scan this QR code with your authenticator app, then enter the
            6-digit code it shows.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col items-center gap-3">
          <img
            src={setup.qr_code_data_uri}
            alt="QR code for your authenticator app"
            className="size-44 border bg-white p-2"
          />
          <div className="flex w-full flex-col gap-1">
            <span className="text-xs text-muted-foreground">
              Can't scan? Enter this key manually:
            </span>
            <div className="flex items-center gap-2">
              <code
                data-testid="totp-secret"
                className="flex-1 border bg-muted px-2 py-1.5 font-mono text-xs break-all select-all"
              >
                {setup.secret}
              </code>
              <Button
                type="button"
                variant="outline"
                size="icon"
                aria-label="Copy key"
                onClick={async () => {
                  if (await copyToClipboard(setup.secret)) setCopied(true)
                  else
                    toast.add({
                      type: "warning",
                      title: "Could not copy automatically",
                      description: "Select the key and copy it manually.",
                    })
                }}
              >
                <HugeiconsIcon icon={Copy01Icon} />
              </Button>
            </div>
            {copied && (
              <span className="text-xs text-muted-foreground" role="status">
                Copied
              </span>
            )}
          </div>
        </div>
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
              {(field) => (
                <TextField
                  field={field}
                  label="6-digit code"
                  inputMode="numeric"
                  autoComplete="one-time-code"
                  maxLength={6}
                />
              )}
            </form.Field>
          </FieldGroup>
          <div aria-live="polite">
            <FormError message={formError} />
          </div>
          <DialogFooter>
            <SubmitButton form={form} pendingLabel="Verifying…">
              Turn on two-factor
            </SubmitButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

/**
 * Blocking on purpose: the codes are never retrievable again, so it can only
 * be closed after the user confirms they saved them.
 */
function RecoveryCodesDialog({
  codes,
  account,
  onDone,
}: {
  codes: string[]
  account: string
  onDone: () => void
}) {
  const [saved, setSaved] = useState(false)
  const [copied, setCopied] = useState(false)

  const download = () => {
    const now = new Date()
    downloadBlob(
      new Blob(
        [formatRecoveryCodesFile(codes, { account, generatedAt: now })],
        {
          type: "text/plain",
        }
      ),
      recoveryCodesFilename(now)
    )
  }

  return (
    <Dialog
      open
      // Escape and outside clicks are ignored until the user has confirmed.
      onOpenChange={(open) => {
        if (!open && saved) onDone()
      }}
    >
      <DialogContent showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>Save your recovery codes</DialogTitle>
          <DialogDescription>
            If you lose your authenticator, each of these signs you in once.
            They will not be shown again.
          </DialogDescription>
        </DialogHeader>
        <ul
          data-testid="recovery-codes"
          className="grid grid-cols-2 gap-2 border bg-muted p-3 font-mono text-sm select-all"
        >
          {codes.map((code) => (
            <li key={code}>{code}</li>
          ))}
        </ul>
        <div className="flex gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={async () => {
              if (await copyToClipboard(codes.join("\n"))) setCopied(true)
              else
                toast.add({
                  type: "warning",
                  title: "Could not copy automatically",
                  description: "Select the codes and copy them manually.",
                })
            }}
          >
            <HugeiconsIcon icon={Copy01Icon} data-icon="inline-start" />
            {copied ? "Copied" : "Copy"}
          </Button>
          <Button type="button" variant="outline" onClick={download}>
            <HugeiconsIcon icon={Download01Icon} data-icon="inline-start" />
            Download .txt
          </Button>
        </div>
        <label className="flex items-center gap-2 text-sm">
          <Checkbox
            checked={saved}
            onCheckedChange={(checked) => setSaved(checked === true)}
          />
          I have saved these codes
        </label>
        <DialogFooter>
          <Button disabled={!saved} onClick={onDone}>
            Done
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function DisableDialog({
  onClose,
  onDisabled,
}: {
  onClose: () => void
  onDisabled: () => void
}) {
  // The API answers a wrong password with 400 (its docs and code); the brief
  // said 401. Either way it belongs under the password field.
  const { form, formError } = useApiForm({
    schema: disableFormSchema,
    defaultValues: { current_password: "" },
    badRequestField: "current_password",
    unauthorizedField: "current_password",
    request: (value) => twoFactorApi.disable(value),
    onSuccess: onDisabled,
  })

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Turn off two-factor?</DialogTitle>
          <DialogDescription>
            Enter your current password to confirm. Your recovery codes will
            stop working.
          </DialogDescription>
        </DialogHeader>
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
          </FieldGroup>
          <div aria-live="polite">
            <FormError message={formError} />
          </div>
          <DialogFooter>
            <SubmitButton
              form={form}
              variant="destructive"
              pendingLabel="Turning off…"
            >
              Turn off
            </SubmitButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
