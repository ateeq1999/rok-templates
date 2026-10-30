import { useState } from "react"
import { Link } from "@tanstack/react-router"
import { useSuspenseQuery } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { MoreHorizontalIcon } from "@hugeicons/core-free-icons"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { toast } from "@/components/ui/toast"
import { meQueryOptions, useUpdateUser } from "@/features/users/queries"
import { userLabel } from "@/features/users/schemas"
import type { User } from "@/features/users/schemas"
import { ApiError } from "@/lib/http"
import { DeleteUserDialog, EditUserDialog } from "./user-dialogs"

/** Row actions menu with the edit/delete dialogs it opens. */
export function UserActions({
  user,
  showView = true,
  onDeleted,
}: {
  user: User
  showView?: boolean
  onDeleted?: () => void
}) {
  const { data: me } = useSuspenseQuery(meQueryOptions)
  const isSelf = me.id === user.id
  const [edit, setEdit] = useState(false)
  const [del, setDel] = useState(false)
  const update = useUpdateUser(user.id)

  const toggleActive = () =>
    update.mutate(
      { is_active: !user.is_active },
      {
        onSuccess: () =>
          toast.add({
            type: "success",
            title: `${userLabel(user)} ${user.is_active ? "deactivated" : "activated"}`,
          }),
        onError: (error) => {
          if (
            error instanceof ApiError &&
            error.status > 0 &&
            error.status < 500
          )
            toast.add({ type: "error", title: error.message })
        },
      }
    )

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={`Actions for ${userLabel(user)}`}
            />
          }
        >
          <HugeiconsIcon icon={MoreHorizontalIcon} />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          {showView && (
            <DropdownMenuItem
              render={
                <Link to="/admin/users/$userId" params={{ userId: user.id }} />
              }
            >
              View
            </DropdownMenuItem>
          )}
          <DropdownMenuItem onClick={() => setEdit(true)}>
            Edit
          </DropdownMenuItem>
          <DropdownMenuItem disabled={isSelf} onClick={toggleActive}>
            {user.is_active ? "Deactivate" : "Activate"}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            variant="destructive"
            disabled={isSelf}
            onClick={() => setDel(true)}
          >
            Delete
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      {edit && <EditUserDialog user={user} open onOpenChange={setEdit} />}
      {del && (
        <DeleteUserDialog
          user={user}
          open
          onOpenChange={setDel}
          onDeleted={onDeleted}
        />
      )}
    </>
  )
}
