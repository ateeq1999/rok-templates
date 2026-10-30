import { useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { formatDistanceToNow } from "date-fns"
import { HugeiconsIcon } from "@hugeicons/react"
import { Copy01Icon, Delete02Icon, Key01Icon } from "@hugeicons/core-free-icons"
import { ConfirmDialog } from "@/components/common/confirm-dialog"
import {
  FormError,
  SelectField,
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  Empty,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { FieldGroup } from "@/components/ui/field"
import { Skeleton } from "@/components/ui/skeleton"
import { toast } from "@/components/ui/toast"
import {
  apiKeysQueryOptions,
  useCreateApiKey,
  useRevokeApiKey,
} from "@/features/api-keys/queries"
import { createApiKeyFormSchema, isExpired } from "@/features/api-keys/schemas"
import type { ApiKey, CreatedApiKey } from "@/features/api-keys/schemas"
import { ApiError } from "@/lib/http"
import { useApiForm } from "@/lib/use-api-form"

const scopeOptions = [
  { value: "read" as const, label: "Read only" },
  { value: "write" as const, label: "Read and write" },
]
const expiryOptions = [
  { value: "30" as const, label: "30 days" },
  { value: "90" as const, label: "90 days" },
  { value: "365" as const, label: "1 year" },
  { value: "never" as const, label: "Never expires" },
]

function keyRowMeta(key: ApiKey) {
  const expired = isExpired(key)
  return {
    expiry: key.expires_at
      ? expired
        ? "Expired"
        : `Expires ${formatDistanceToNow(new Date(key.expires_at), { addSuffix: true })}`
      : "Never expires",
    expired,
    lastUsed: key.last_used_at
      ? `Last used ${formatDistanceToNow(new Date(key.last_used_at), { addSuffix: true })}`
      : "Never used",
  }
}

export function ApiKeysPanel() {
  const { data, isPending, isError } = useQuery(apiKeysQueryOptions)
  const revoke = useRevokeApiKey()
  const [createOpen, setCreateOpen] = useState(false)
  const [reveal, setReveal] = useState<CreatedApiKey>()
  const [pendingRevoke, setPendingRevoke] = useState<{
    id: string
    name: string
  }>()

  return (
    <Card>
      <CardHeader className="flex-row items-start justify-between gap-3">
        <div>
          <CardTitle>API keys</CardTitle>
          <CardDescription>
            Use a key to call the API without signing in interactively. Keys
            cannot manage passwords, emails, other keys, passkeys or sign-in
            methods.
          </CardDescription>
        </div>
        <Button onClick={() => setCreateOpen(true)}>Create key</Button>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        {isPending && (
          <>
            <Skeleton className="h-14 w-full" />
            <Skeleton className="h-14 w-full" />
          </>
        )}
        {isError && (
          <p className="text-sm text-muted-foreground">
            Could not load your API keys.
          </p>
        )}
        {data?.length === 0 && (
          <Empty className="border py-6">
            <EmptyMedia variant="icon">
              <HugeiconsIcon icon={Key01Icon} />
            </EmptyMedia>
            <EmptyTitle>No API keys yet</EmptyTitle>
            <EmptyDescription>
              Create one to call the API from a script or another service.
            </EmptyDescription>
          </Empty>
        )}
        {data?.map((key) => {
          const meta = keyRowMeta(key)
          return (
            <div
              key={key.id}
              className="flex items-center gap-3 border px-3 py-2"
            >
              <div className="flex min-w-0 flex-1 flex-col gap-0.5 text-sm">
                <div className="flex items-center gap-2">
                  <span className="truncate font-medium">{key.name}</span>
                  <Badge
                    variant={key.scope === "write" ? "default" : "secondary"}
                  >
                    {key.scope === "write" ? "Read and write" : "Read only"}
                  </Badge>
                  {meta.expired && <Badge variant="destructive">Expired</Badge>}
                </div>
                <span className="text-xs text-muted-foreground">
                  <code>{key.key_prefix}…</code> · {meta.expiry} ·{" "}
                  {meta.lastUsed}
                </span>
              </div>
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label={`Revoke ${key.name}`}
                onClick={() => setPendingRevoke({ id: key.id, name: key.name })}
              >
                <HugeiconsIcon icon={Delete02Icon} />
              </Button>
            </div>
          )
        })}
      </CardContent>

      {createOpen && (
        <CreateApiKeyDialog
          open
          onOpenChange={setCreateOpen}
          onCreated={setReveal}
        />
      )}
      {reveal && (
        <RevealKeyDialog apiKey={reveal} onClose={() => setReveal(undefined)} />
      )}
      {pendingRevoke && (
        <ConfirmDialog
          open
          onOpenChange={(open) => !open && setPendingRevoke(undefined)}
          title={`Revoke ${pendingRevoke.name}?`}
          description="Anything using this key will stop working immediately."
          confirmLabel="Revoke"
          pending={revoke.isPending}
          onConfirm={() =>
            revoke.mutate(pendingRevoke.id, {
              onSuccess: () => {
                toast.add({ type: "success", title: "Key revoked" })
                setPendingRevoke(undefined)
              },
              onError: (error) => {
                if (error instanceof ApiError)
                  toast.add({ type: "error", title: error.message })
                setPendingRevoke(undefined)
              },
            })
          }
        />
      )}
    </Card>
  )
}

function CreateApiKeyDialog({
  open,
  onOpenChange,
  onCreated,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onCreated: (key: CreatedApiKey) => void
}) {
  const create = useCreateApiKey()
  const { form, formError } = useApiForm({
    schema: createApiKeyFormSchema,
    defaultValues: { name: "", scope: "read", expiry: "90" },
    request: (value) => create.mutateAsync(value),
    onSuccess: (key) => {
      onOpenChange(false)
      onCreated(key)
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create API key</DialogTitle>
          <DialogDescription>
            The key is only ever shown once, right after you create it.
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
            <form.Field name="name">
              {(f) => <TextField field={f} label="Name" />}
            </form.Field>
            <form.Field name="scope">
              {(f) => (
                <SelectField field={f} label="Access" options={scopeOptions} />
              )}
            </form.Field>
            <form.Field name="expiry">
              {(f) => (
                <SelectField
                  field={f}
                  label="Expires"
                  options={expiryOptions}
                />
              )}
            </form.Field>
          </FieldGroup>
          <FormError message={formError} />
          <DialogFooter>
            <SubmitButton form={form} pendingLabel="Creating…">
              Create key
            </SubmitButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

function RevealKeyDialog({
  apiKey,
  onClose,
}: {
  apiKey: CreatedApiKey
  onClose: () => void
}) {
  const [copied, setCopied] = useState(false)

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(apiKey.key)
      setCopied(true)
    } catch {
      toast.add({
        type: "warning",
        title: "Could not copy automatically",
        description: "Select the key and copy it manually.",
      })
    }
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Your new API key</DialogTitle>
          <DialogDescription>
            Copy it now. For your security, it will not be shown again.
          </DialogDescription>
        </DialogHeader>
        <div
          data-testid="new-api-key"
          className="border bg-muted px-3 py-2 font-mono text-sm break-all select-all"
        >
          {apiKey.key}
        </div>
        <DialogFooter className="gap-2 sm:justify-between">
          <Button variant="outline" onClick={() => void copy()}>
            <HugeiconsIcon icon={Copy01Icon} data-icon="inline-start" />
            {copied ? "Copied" : "Copy"}
          </Button>
          <Button onClick={onClose}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
