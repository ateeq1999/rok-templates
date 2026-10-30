import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import { Task01Icon } from "@hugeicons/core-free-icons"
import { ErrorScreen } from "@/components/common/screens"
import { PageHeader } from "@/components/layout/page-header"
import { Button } from "@/components/ui/button"
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { AuditLogTable } from "@/features/audit-log/components/audit-log-table"
import { auditLogQueryOptions } from "@/features/audit-log/queries"
import { auditSearchSchema } from "@/features/audit-log/schemas"

// Guarded by the parent /admin route (redirects non-admins); the API also
// answers 403, which the error screen below reports.
export const Route = createFileRoute("/_app/admin/audit-log")({
  validateSearch: auditSearchSchema,
  loaderDeps: ({ search }) => search,
  loader: ({ context, deps }) => {
    void context.queryClient.prefetchQuery(auditLogQueryOptions(deps))
  },
  component: AuditLogPage,
})

function AuditLogPage() {
  const params = Route.useSearch()
  const navigate = Route.useNavigate()
  const { data, isPending, isError, error, refetch } = useQuery(
    auditLogQueryOptions(params)
  )

  const total = data?.total ?? 0
  const lastPage = Math.max(1, Math.ceil(total / params.per_page))
  const from = total === 0 ? 0 : (params.page - 1) * params.per_page + 1
  const to = Math.min(params.page * params.per_page, total)

  if (isError && !data) {
    return <ErrorScreen error={error} reset={() => void refetch()} />
  }

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        title="Audit log"
        description={
          data
            ? `${total.toLocaleString()} administrative actions, newest first`
            : "Administrative actions, newest first"
        }
      />

      {!isPending && data?.items.length === 0 ? (
        <Empty className="border">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <HugeiconsIcon icon={Task01Icon} />
            </EmptyMedia>
            <EmptyTitle>Nothing recorded yet</EmptyTitle>
            <EmptyDescription>
              Creating users and changing roles, status or accounts shows up
              here.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <AuditLogTable
          items={data?.items ?? []}
          loading={isPending}
          pageSize={params.per_page}
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
            disabled={params.page <= 1}
            onClick={() =>
              void navigate({ search: (p) => ({ ...p, page: p.page - 1 }) })
            }
          >
            Previous
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={params.page >= lastPage}
            onClick={() =>
              void navigate({ search: (p) => ({ ...p, page: p.page + 1 }) })
            }
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  )
}
