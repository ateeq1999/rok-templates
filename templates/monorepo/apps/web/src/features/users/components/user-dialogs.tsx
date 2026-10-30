import {
  useSuspenseQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query"
import {
  FormError,
  PasswordField,
  SelectField,
  SubmitButton,
  SwitchField,
  TextField,
} from "@/components/common/form-fields"
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Button } from "@/components/ui/button"
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
import { authApi } from "@/features/auth/api"
import { PasswordStrengthMeter } from "@/features/auth/components/password-strength-meter"
import { statsKeys } from "@/features/stats/queries"
import {
  meQueryOptions,
  useCreateUser,
  useDeleteUser,
  useUpdateUser,
  userKeys,
} from "@/features/users/queries"
import {
  createUserSchema,
  editUserSchema,
  inviteUserSchema,
  userLabel,
} from "@/features/users/schemas"
import type { Role, User } from "@/features/users/schemas"
import { ApiError } from "@/lib/http"
import { useApiForm } from "@/lib/use-api-form"

const roleOptions: { value: Role; label: string }[] = [
  { value: "user", label: "User" },
  { value: "admin", label: "Administrator" },
]

type DialogProps = { open: boolean; onOpenChange: (open: boolean) => void }

export function CreateUserDialog({ open, onOpenChange }: DialogProps) {
  const create = useCreateUser()
  const { form, formError } = useApiForm({
    schema: createUserSchema,
    defaultValues: {
      email: "",
      password: "",
      display_name: "",
      role: "user",
    },
    conflictField: "email",
    // password-policy rejections are a 400, shown under the password field
    badRequestField: "password",
    request: ({ display_name, ...rest }) =>
      create.mutateAsync({ ...rest, display_name: display_name || undefined }),
    onSuccess: (user) => {
      toast.add({ type: "success", title: `Created ${user.email}` })
      form.reset()
      onOpenChange(false)
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create user</DialogTitle>
          <DialogDescription>
            The account is created immediately with the password you set.
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
            <form.Field name="email">
              {(f) => <TextField field={f} label="Email" type="email" />}
            </form.Field>
            <form.Field name="password">
              {(f) => (
                <PasswordField
                  field={f}
                  label="Password"
                  autoComplete="new-password"
                  below={
                    <PasswordStrengthMeter
                      password={f.state.value}
                      knownInputs={[
                        f.form.state.values.email,
                        f.form.state.values.display_name,
                      ]}
                    />
                  }
                />
              )}
            </form.Field>
            <form.Field name="display_name">
              {(f) => <TextField field={f} label="Display name (optional)" />}
            </form.Field>
            <form.Field name="role">
              {(f) => (
                <SelectField field={f} label="Role" options={roleOptions} />
              )}
            </form.Field>
          </FieldGroup>
          <FormError message={formError} />
          <DialogFooter>
            <SubmitButton form={form} pendingLabel="Creating…">
              Create user
            </SubmitButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export function InviteUserDialog({ open, onOpenChange }: DialogProps) {
  const qc = useQueryClient()
  const { form, formError } = useApiForm({
    schema: inviteUserSchema,
    defaultValues: { email: "", display_name: "", role: "user" },
    conflictField: "email",
    request: ({ display_name, ...rest }) =>
      authApi.invite({ ...rest, display_name: display_name || undefined }),
    onSuccess: (user) => {
      toast.add({
        type: "success",
        title: `Invitation sent to ${user.email}`,
        description: "The set-your-password link is valid for 24 hours.",
      })
      void qc.invalidateQueries({ queryKey: userKeys.lists() })
      void qc.invalidateQueries({ queryKey: statsKeys.all })
      form.reset()
      onOpenChange(false)
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Invite user</DialogTitle>
          <DialogDescription>
            They receive an email with a link to set their own password.
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
            <form.Field name="email">
              {(f) => <TextField field={f} label="Email" type="email" />}
            </form.Field>
            <form.Field name="display_name">
              {(f) => <TextField field={f} label="Display name (optional)" />}
            </form.Field>
            <form.Field name="role">
              {(f) => (
                <SelectField field={f} label="Role" options={roleOptions} />
              )}
            </form.Field>
          </FieldGroup>
          <FormError message={formError} />
          <DialogFooter>
            <SubmitButton form={form} pendingLabel="Sending…">
              Send invitation
            </SubmitButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export function EditUserDialog({
  user,
  open,
  onOpenChange,
}: DialogProps & { user: User }) {
  const { data: me } = useSuspenseQuery(meQueryOptions)
  const isSelf = me.id === user.id
  const update = useUpdateUser(user.id)

  const { form, formError } = useApiForm({
    schema: editUserSchema,
    defaultValues: {
      display_name: user.display_name ?? "",
      role: user.role,
      is_active: user.is_active,
    },
    request: async (value) => {
      // only send what changed: absent fields mean "leave unchanged"
      const body: Parameters<typeof update.mutateAsync>[0] = {}
      if (value.display_name.trim() !== (user.display_name ?? ""))
        body.display_name = value.display_name.trim()
      if (value.role !== user.role) body.role = value.role
      if (value.is_active !== user.is_active) body.is_active = value.is_active
      if (Object.keys(body).length === 0) return user
      return update.mutateAsync(body)
    },
    onSuccess: () => {
      toast.add({ type: "success", title: "User updated" })
      onOpenChange(false)
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit {userLabel(user)}</DialogTitle>
          <DialogDescription>{user.email}</DialogDescription>
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
            <form.Field name="display_name">
              {(f) => <TextField field={f} label="Display name" />}
            </form.Field>
            <form.Field name="role">
              {(f) => (
                <SelectField
                  field={f}
                  label={isSelf ? "Role (you cannot demote yourself)" : "Role"}
                  options={roleOptions}
                />
              )}
            </form.Field>
            <form.Field name="is_active">
              {(f) => (
                <SwitchField
                  field={f}
                  label="Active"
                  description={
                    isSelf
                      ? "You cannot deactivate yourself."
                      : "Deactivated users cannot sign in."
                  }
                  disabled={isSelf}
                />
              )}
            </form.Field>
          </FieldGroup>
          <FormError message={formError} />
          <DialogFooter>
            <SubmitButton form={form} pendingLabel="Saving…">
              Save changes
            </SubmitButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export function DeleteUserDialog({
  user,
  open,
  onOpenChange,
  onDeleted,
}: DialogProps & { user: User; onDeleted?: () => void }) {
  const remove = useDeleteUser()
  const mutation = useMutation({
    mutationFn: () => remove.mutateAsync(user.id),
    onSuccess: () => {
      toast.add({ type: "success", title: `Deleted ${user.email}` })
      onOpenChange(false)
      onDeleted?.()
    },
    onError: (error) => {
      if (error instanceof ApiError && error.status > 0 && error.status < 500) {
        toast.add({ type: "error", title: error.message })
        onOpenChange(false)
      }
    },
  })

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete {userLabel(user)}?</AlertDialogTitle>
          <AlertDialogDescription>
            This deactivates and hides the account. The email address becomes
            available again.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <Button
            variant="destructive"
            disabled={mutation.isPending}
            onClick={() => mutation.mutate()}
          >
            {mutation.isPending && <Spinner />}
            Delete
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
