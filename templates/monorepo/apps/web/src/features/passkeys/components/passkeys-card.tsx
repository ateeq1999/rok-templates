import { useState } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { formatDistanceToNow } from "date-fns"
import { HugeiconsIcon } from "@hugeicons/react"
import { Delete02Icon, FingerprintScanIcon } from "@hugeicons/core-free-icons"
import { ConfirmDialog } from "@/components/common/confirm-dialog"
import { FormError } from "@/components/common/form-fields"
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
import { Field, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { Spinner } from "@/components/ui/spinner"
import { toast } from "@/components/ui/toast"
import { passkeysApi } from "@/features/passkeys/api"
import {
  passkeyKeys,
  passkeysQueryOptions,
  useRemovePasskey,
} from "@/features/passkeys/queries"
import { nameSchema } from "@/features/passkeys/schemas"
import {
  PasskeyError,
  createPasskey,
  passkeysSupported,
} from "@/features/passkeys/webauthn"
import { ApiError } from "@/lib/http"
import { describeUserAgent } from "@/lib/user-agent"

export function PasskeysCard() {
  const { data, isPending, isError } = useQuery(passkeysQueryOptions)
  const remove = useRemovePasskey()
  const [adding, setAdding] = useState(false)
  const [pendingRemove, setPendingRemove] = useState<{
    id: string
    name: string
  }>()
  const supported = passkeysSupported()

  return (
    <Card>
      <CardHeader>
        <CardTitle>Passkeys</CardTitle>
        <CardDescription>
          {supported
            ? "Sign in with your device's fingerprint, face or screen lock."
            : "This browser or device does not support passkeys."}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {isPending && (
          <>
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </>
        )}
        {isError && (
          <p className="text-sm text-muted-foreground">
            Could not load your passkeys.
          </p>
        )}
        {data?.length === 0 && (
          <Empty className="border py-6">
            <EmptyMedia variant="icon">
              <HugeiconsIcon icon={FingerprintScanIcon} />
            </EmptyMedia>
            <EmptyTitle>No passkeys yet</EmptyTitle>
            <EmptyDescription>
              Add one for faster, passwordless sign-in.
            </EmptyDescription>
          </Empty>
        )}
        {data?.map((p) => (
          <div key={p.id} className="flex items-center gap-3 border px-3 py-2">
            <div className="flex min-w-0 flex-1 flex-col text-sm">
              <span className="truncate font-medium">{p.name}</span>
              <span className="text-xs text-muted-foreground">
                Added{" "}
                {formatDistanceToNow(new Date(p.created_at), {
                  addSuffix: true,
                })}
                {" · "}
                {p.last_used_at
                  ? `Last used ${formatDistanceToNow(new Date(p.last_used_at), { addSuffix: true })}`
                  : "Never used"}
              </span>
            </div>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={`Remove ${p.name}`}
              onClick={() => setPendingRemove({ id: p.id, name: p.name })}
            >
              <HugeiconsIcon icon={Delete02Icon} />
            </Button>
          </div>
        ))}
        {supported && (
          <div>
            <Button variant="outline" size="sm" onClick={() => setAdding(true)}>
              Add a passkey
            </Button>
          </div>
        )}
      </CardContent>

      {adding && <AddPasskeyDialog open onOpenChange={setAdding} />}
      {pendingRemove && (
        <ConfirmDialog
          open
          onOpenChange={(open) => !open && setPendingRemove(undefined)}
          title={`Remove ${pendingRemove.name}?`}
          description="You will no longer be able to sign in with this passkey."
          confirmLabel="Remove"
          pending={remove.isPending}
          onConfirm={() =>
            remove.mutate(pendingRemove.id, {
              onSuccess: () => {
                toast.add({ type: "success", title: "Passkey removed" })
                setPendingRemove(undefined)
              },
              onError: (error) => {
                if (error instanceof ApiError)
                  toast.add({ type: "error", title: error.message })
                setPendingRemove(undefined)
              },
            })
          }
        />
      )}
    </Card>
  )
}

function AddPasskeyDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const qc = useQueryClient()
  const [name, setName] = useState(
    () => describeUserAgent(navigator.userAgent) ?? "This device"
  )
  const [fieldError, setFieldError] = useState<string>()
  const [formError, setFormError] = useState<string>()

  const add = useMutation({
    mutationFn: async () => {
      const parsed = nameSchema.safeParse(name)
      if (!parsed.success) {
        setFieldError(parsed.error.issues[0]?.message)
        throw new Error("invalid")
      }
      const { challenge_id, options } = await passkeysApi.registerBegin()
      const credential = await createPasskey(options)
      return passkeysApi.registerFinish({
        challenge_id,
        name: parsed.data,
        credential,
      })
    },
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: passkeyKeys.all })
      toast.add({ type: "success", title: "Passkey added" })
      onOpenChange(false)
    },
    onError: (error) => {
      if (error instanceof PasskeyError) {
        if (error.reason === "cancelled") return // nothing happened
        setFormError(error.message)
      } else if (error instanceof ApiError) {
        setFormError(error.message)
      }
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add a passkey</DialogTitle>
          <DialogDescription>
            Your browser will ask you to confirm with your fingerprint, face or
            screen lock.
          </DialogDescription>
        </DialogHeader>
        <form
          className="flex flex-col gap-4"
          noValidate
          onSubmit={(e) => {
            e.preventDefault()
            setFieldError(undefined)
            setFormError(undefined)
            add.mutate()
          }}
        >
          <Field data-invalid={Boolean(fieldError)}>
            <FieldLabel htmlFor="passkey-name">Name</FieldLabel>
            <Input
              id="passkey-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              aria-invalid={Boolean(fieldError)}
            />
            {fieldError && (
              <p className="text-xs text-destructive">{fieldError}</p>
            )}
          </Field>
          <FormError message={formError} />
          <DialogFooter>
            <Button type="submit" disabled={add.isPending}>
              {add.isPending && <Spinner />}
              Add passkey
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
