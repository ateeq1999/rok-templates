import { createFileRoute } from "@tanstack/react-router"
import { z } from "zod"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useState } from "react"
import { HugeiconsIcon } from "@hugeicons/react"
import {
  ArrowDown01Icon,
  ArrowUp01Icon,
  UserGroupIcon,
} from "@hugeicons/core-free-icons"
import { PageHeader } from "@/components/layout/page-header"
import { Button } from "@/components/ui/button"
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  CreateUserDialog,
  InviteUserDialog,
} from "@/features/users/components/user-dialogs"
import { UsersTable } from "@/features/users/components/users-table"
import { usersListQueryOptions } from "@/features/users/queries"
import { listSearchSchema } from "@/features/users/schemas"
import { ErrorScreen } from "@/components/common/screens"

export const Route = createFileRoute("/_app/admin/users/")({
  validateSearch: listSearchSchema.extend({
    dialog: z.enum(["create", "invite"]).optional().catch(undefined),
  }),
  loaderDeps: ({ search: { dialog: _dialog, ...rest } }) => rest,
  loader: ({ context, deps }) => {
    // warm the cache without blocking navigation on the list
    void context.queryClient.prefetchQuery(usersListQueryOptions(deps))
  },
  component: UsersPage,
})

const sortOptions = [
  { value: "created_at", label: "Created" },
  { value: "email", label: "Email" },
]
const sizeOptions = [10, 20, 50, 100].map((n) => ({
  value: String(n),
  label: `${n} / page`,
}))

function UsersPage() {
  const { dialog, ...search } = Route.useSearch()
  const navigate = Route.useNavigate()
  const qc = useQueryClient()
  const [createOpen, setCreateOpen] = useState(dialog === "create")
  const [inviteOpen, setInviteOpen] = useState(dialog === "invite")
  const [text, setText] = useState(search.q ?? "")

  // a quick-action link opens a dialog once; drop the param so reloads don't
  useEffect(() => {
    if (dialog)
      void navigate({
        search: (p) => ({ ...p, dialog: undefined }),
        replace: true,
      })
  }, [dialog, navigate])

  const { data, isPending, isError, error, refetch } = useQuery(
    usersListQueryOptions(search)
  )

  // debounce the search box into the URL; a new query goes back to page 1
  useEffect(() => {
    if (text === (search.q ?? "")) return
    const id = setTimeout(
      () =>
        void navigate({
          search: (prev) => ({ ...prev, q: text || undefined, page: 1 }),
          replace: true,
        }),
      300
    )
    return () => clearTimeout(id)
  }, [text, search.q, navigate])

  // prefetch the next page
  const total = data?.total ?? 0
  const lastPage = Math.max(1, Math.ceil(total / search.per_page))
  useEffect(() => {
    if (search.page < lastPage)
      void qc.prefetchQuery(
        usersListQueryOptions({ ...search, page: search.page + 1 })
      )
  }, [qc, search, lastPage])

  const from = total === 0 ? 0 : (search.page - 1) * search.per_page + 1
  const to = Math.min(search.page * search.per_page, total)

  if (isError && !data) {
    return <ErrorScreen error={error} reset={() => void refetch()} />
  }

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        title="Users"
        description={data ? `${data.total.toLocaleString()} total` : undefined}
        actions={
          <>
            <Button variant="outline" onClick={() => setInviteOpen(true)}>
              Invite user
            </Button>
            <Button onClick={() => setCreateOpen(true)}>Create user</Button>
          </>
        }
      />

      <div className="flex flex-wrap items-center gap-2">
        <Input
          type="search"
          aria-label="Search users"
          placeholder="Search email or name…"
          className="w-full sm:w-64"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
        <Select
          value={search.sort}
          items={sortOptions}
          onValueChange={(v) =>
            void navigate({
              search: (p) => ({
                ...p,
                sort: v as "email" | "created_at",
                page: 1,
              }),
            })
          }
        >
          <SelectTrigger aria-label="Sort by" className="w-32">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {sortOptions.map((o) => (
              <SelectItem key={o.value} value={o.value}>
                {o.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          variant="outline"
          size="icon"
          aria-label={
            search.order === "asc" ? "Sort ascending" : "Sort descending"
          }
          onClick={() =>
            void navigate({
              search: (p) => ({
                ...p,
                order: p.order === "asc" ? "desc" : "asc",
                page: 1,
              }),
            })
          }
        >
          <HugeiconsIcon
            icon={search.order === "asc" ? ArrowUp01Icon : ArrowDown01Icon}
          />
        </Button>
        <Select
          value={String(search.per_page)}
          items={sizeOptions}
          onValueChange={(v) =>
            void navigate({
              search: (p) => ({ ...p, per_page: Number(v), page: 1 }),
            })
          }
        >
          <SelectTrigger aria-label="Rows per page" className="w-32">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {sizeOptions.map((o) => (
              <SelectItem key={o.value} value={o.value}>
                {o.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {!isPending && data?.items.length === 0 ? (
        <Empty className="border">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <HugeiconsIcon icon={UserGroupIcon} />
            </EmptyMedia>
            <EmptyTitle>
              {search.q ? "No matching users" : "No users yet"}
            </EmptyTitle>
            <EmptyDescription>
              {search.q
                ? "Try a different search."
                : "Create or invite the first user."}
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <UsersTable
          data={data?.items ?? []}
          total={total}
          loading={isPending}
          pageSize={search.per_page}
        />
      )}

      <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
        <span aria-live="polite">
          {total > 0 ? `${from}–${to} of ${total}` : ""}
        </span>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={search.page <= 1}
            onClick={() =>
              void navigate({ search: (p) => ({ ...p, page: p.page - 1 }) })
            }
          >
            Previous
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={search.page >= lastPage}
            onClick={() =>
              void navigate({ search: (p) => ({ ...p, page: p.page + 1 }) })
            }
          >
            Next
          </Button>
        </div>
      </div>

      {createOpen && <CreateUserDialog open onOpenChange={setCreateOpen} />}
      {inviteOpen && <InviteUserDialog open onOpenChange={setInviteOpen} />}
    </div>
  )
}
