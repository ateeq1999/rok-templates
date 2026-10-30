import { useRef, useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useSuspenseQuery } from "@tanstack/react-query"
import { format } from "date-fns"
import { z } from "zod"
import { PageHeader } from "@/components/layout/page-header"
import { ConfirmDialog } from "@/components/common/confirm-dialog"
import {
  FormError,
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
import { FieldGroup } from "@/components/ui/field"
import { toast } from "@/components/ui/toast"
import { AVATAR_TYPES, validateAvatarFile } from "@/features/users/avatar"
import { UserAvatar } from "@/features/users/components/user-avatar"
import {
  meQueryOptions,
  useAvatarMutations,
  useUpdateMe,
} from "@/features/users/queries"
import { displayNameSchema, userLabel } from "@/features/users/schemas"
import { useApiForm } from "@/lib/use-api-form"
import { ApiError } from "@/lib/http"

export const Route = createFileRoute("/_app/profile")({
  component: ProfilePage,
})

const nameFormSchema = z.object({ display_name: displayNameSchema })

function ProfilePage() {
  const { data: me } = useSuspenseQuery(meQueryOptions)
  const update = useUpdateMe()
  const { upload, remove } = useAvatarMutations()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [removeOpen, setRemoveOpen] = useState(false)

  const { form, formError } = useApiForm({
    schema: nameFormSchema,
    defaultValues: { display_name: me.display_name ?? "" },
    request: (value) => update.mutateAsync(value),
    onSuccess: () => {
      toast.add({ type: "success", title: "Profile updated" })
    },
  })

  const onFileChosen = (file: File) => {
    const error = validateAvatarFile(file)
    if (error) {
      toast.add({ type: "error", title: error })
      return
    }
    upload.mutate(file, {
      onSuccess: () => toast.add({ type: "success", title: "Photo updated" }),
      onError: (err) => {
        if (err instanceof ApiError)
          toast.add({ type: "error", title: err.message })
      },
    })
  }

  const busy = upload.isPending || remove.isPending

  return (
    <div className="flex max-w-2xl flex-col gap-4">
      <PageHeader title="Profile" />
      <Card>
        <CardHeader className="flex-row items-center gap-4">
          <div
            className="relative"
            onDragOver={(e) => e.preventDefault()}
            onDrop={(e) => {
              e.preventDefault()
              const file = e.dataTransfer.files[0]
              if (file) onFileChosen(file)
            }}
          >
            <UserAvatar user={me} size="lg" className="size-16" />
            <button
              type="button"
              aria-label="Change photo"
              disabled={busy}
              onClick={() => fileInputRef.current?.click()}
              className="absolute inset-0 rounded-full bg-foreground/0 transition-colors hover:bg-foreground/30 disabled:pointer-events-none"
            />
          </div>
          <div className="min-w-0 flex-1">
            <CardTitle className="truncate">{userLabel(me)}</CardTitle>
            <CardDescription>
              Member since {format(new Date(me.created_at), "PP")}
            </CardDescription>
          </div>
          <Badge variant={me.role === "admin" ? "default" : "secondary"}>
            {me.role}
          </Badge>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center gap-2">
          <input
            ref={fileInputRef}
            type="file"
            accept={AVATAR_TYPES.join(",")}
            className="sr-only"
            onChange={(e) => {
              const file = e.target.files?.[0]
              if (file) onFileChosen(file)
              e.target.value = ""
            }}
          />
          <Button
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() => fileInputRef.current?.click()}
          >
            {upload.isPending ? "Uploading…" : "Change photo"}
          </Button>
          {me.avatar_url && (
            <Button
              variant="ghost"
              size="sm"
              disabled={busy}
              onClick={() => setRemoveOpen(true)}
            >
              Remove photo
            </Button>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Display name</CardTitle>
          <CardDescription>Shown across the app.</CardDescription>
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
              <form.Field name="display_name">
                {(field) => (
                  <TextField
                    field={field}
                    label="Name"
                    autoComplete="name"
                    maxLength={100}
                  />
                )}
              </form.Field>
            </FieldGroup>
            <div aria-live="polite">
              <FormError message={formError} />
            </div>
            <div>
              <SubmitButton form={form} pendingLabel="Saving…">
                Save
              </SubmitButton>
            </div>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Account</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3 text-sm">
          <Row label="Email">
            <span className="truncate">{me.email}</span>
            <Badge variant={me.email_verified ? "secondary" : "destructive"}>
              {me.email_verified ? "Verified" : "Unverified"}
            </Badge>
          </Row>
          <Row label="Status">
            <Badge variant={me.is_active ? "secondary" : "destructive"}>
              {me.is_active ? "Active" : "Deactivated"}
            </Badge>
          </Row>
          <Row label="Sign-in">
            <span>
              {me.has_password ? "Password" : "Google, GitHub or passkey"}
            </span>
          </Row>
        </CardContent>
      </Card>

      <ConfirmDialog
        open={removeOpen}
        onOpenChange={setRemoveOpen}
        title="Remove your photo?"
        description="Your avatar will go back to showing your initials."
        confirmLabel="Remove"
        pending={remove.isPending}
        onConfirm={() =>
          remove.mutate(undefined, {
            onSuccess: () => {
              toast.add({ type: "success", title: "Photo removed" })
              setRemoveOpen(false)
            },
            onError: (err) => {
              if (err instanceof ApiError)
                toast.add({ type: "error", title: err.message })
              setRemoveOpen(false)
            },
          })
        }
      />
    </div>
  )
}

function Row({
  label,
  children,
}: {
  label: string
  children: React.ReactNode
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-muted-foreground">{label}</span>
      <div className="flex min-w-0 items-center gap-2">{children}</div>
    </div>
  )
}
