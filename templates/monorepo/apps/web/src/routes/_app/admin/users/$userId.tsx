import { createFileRoute, useNavigate } from "@tanstack/react-router"
import { useSuspenseQuery } from "@tanstack/react-query"
import { format } from "date-fns"
import { PageHeader } from "@/components/layout/page-header"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { UserActions } from "@/features/users/components/user-actions"
import { UserAvatar } from "@/features/users/components/user-avatar"
import { userDetailQueryOptions } from "@/features/users/queries"
import { userLabel } from "@/features/users/schemas"

export const Route = createFileRoute("/_app/admin/users/$userId")({
  loader: ({ context, params }) =>
    context.queryClient.ensureQueryData(userDetailQueryOptions(params.userId)),
  component: UserDetailPage,
})

function UserDetailPage() {
  const { userId } = Route.useParams()
  const { data: user } = useSuspenseQuery(userDetailQueryOptions(userId))
  const navigate = useNavigate()

  const rows: [string, React.ReactNode][] = [
    [
      "ID",
      <code key="id" className="text-xs">
        {user.id}
      </code>,
    ],
    ["Email", user.email],
    ["Display name", user.display_name ?? "—"],
    [
      "Role",
      <Badge key="r" variant={user.role === "admin" ? "default" : "secondary"}>
        {user.role}
      </Badge>,
    ],
    [
      "Status",
      <Badge key="s" variant={user.is_active ? "secondary" : "destructive"}>
        {user.is_active ? "Active" : "Deactivated"}
      </Badge>,
    ],
    [
      "Email verified",
      <Badge
        key="v"
        variant={user.email_verified ? "secondary" : "destructive"}
      >
        {user.email_verified ? "Verified" : "Unverified"}
      </Badge>,
    ],
    ["Created", format(new Date(user.created_at), "PPpp")],
  ]

  return (
    <div className="flex max-w-2xl flex-col gap-4">
      <PageHeader
        title={userLabel(user)}
        description={user.email}
        actions={
          <UserActions
            user={user}
            showView={false}
            onDeleted={() => void navigate({ to: "/admin/users" })}
          />
        }
      />
      <Card>
        <CardHeader className="flex-row items-center gap-3">
          <UserAvatar user={user} size="lg" />
          <CardTitle>Details</CardTitle>
        </CardHeader>
        <CardContent>
          <dl className="grid gap-3 text-sm sm:grid-cols-[10rem_1fr]">
            {rows.map(([label, value]) => (
              <div key={label} className="contents">
                <dt className="text-muted-foreground">{label}</dt>
                <dd className="min-w-0 break-words">{value}</dd>
              </div>
            ))}
          </dl>
        </CardContent>
      </Card>
    </div>
  )
}
