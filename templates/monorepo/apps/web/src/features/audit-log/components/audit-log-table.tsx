import { Link } from "@tanstack/react-router"
import { format, formatDistanceToNow } from "date-fns"
import { Badge } from "@/components/ui/badge"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip"
import { actionMeta, detailEntries } from "@/features/audit-log/labels"
import { useUserLabels } from "@/features/audit-log/queries"
import type { AuditEntry } from "@/features/audit-log/schemas"

function UserRef({
  id,
  lookup,
}: {
  id: string | null
  lookup: ReturnType<typeof useUserLabels>
}) {
  if (!id) return <span className="text-muted-foreground">—</span>
  const ref = lookup(id)
  if (!ref) return <span className="text-muted-foreground">—</span>
  if (!ref.resolved)
    return (
      <span className="font-mono text-xs" title={`${id} (no longer available)`}>
        {ref.label}
      </span>
    )
  return (
    <Link
      to="/admin/users/$userId"
      params={{ userId: id }}
      className="hover:underline"
    >
      {ref.label}
    </Link>
  )
}

function Details({ details }: { details: unknown }) {
  const rows = detailEntries(details)
  if (rows.length === 0) return <span className="text-muted-foreground">—</span>
  return (
    <dl className="flex flex-wrap gap-x-4 gap-y-0.5 text-xs">
      {rows.map(({ key, value }) => (
        <div key={key} className="flex gap-1">
          <dt className="text-muted-foreground">{key}:</dt>
          <dd className="break-all">{value}</dd>
        </div>
      ))}
    </dl>
  )
}

const HEADINGS = ["When", "Action", "Actor", "Target", "Details"]

export function AuditLogTable({
  items,
  loading,
  pageSize,
}: {
  items: AuditEntry[]
  loading: boolean
  pageSize: number
}) {
  const lookup = useUserLabels(
    items.flatMap((e) => [e.actor_user_id, e.target_user_id])
  )

  return (
    <div className="overflow-x-auto border">
      <Table>
        <TableHeader>
          <TableRow>
            {HEADINGS.map((h) => (
              <TableHead key={h}>{h}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {loading
            ? Array.from({ length: Math.min(pageSize, 8) }, (_, i) => (
                <TableRow key={i}>
                  {HEADINGS.map((h) => (
                    <TableCell key={h}>
                      <Skeleton className="h-4 w-full max-w-32" />
                    </TableCell>
                  ))}
                </TableRow>
              ))
            : items.map((entry) => {
                const meta = actionMeta(entry.action)
                const at = new Date(entry.created_at)
                return (
                  <TableRow key={entry.id}>
                    <TableCell className="whitespace-nowrap">
                      <Tooltip>
                        <TooltipTrigger render={<span />}>
                          {formatDistanceToNow(at, { addSuffix: true })}
                        </TooltipTrigger>
                        <TooltipContent>{format(at, "PPpp")}</TooltipContent>
                      </Tooltip>
                    </TableCell>
                    <TableCell>
                      <Badge variant={meta.variant}>{meta.label}</Badge>
                    </TableCell>
                    <TableCell>
                      <UserRef id={entry.actor_user_id} lookup={lookup} />
                    </TableCell>
                    <TableCell>
                      <UserRef id={entry.target_user_id} lookup={lookup} />
                    </TableCell>
                    <TableCell>
                      <Details details={entry.details} />
                    </TableCell>
                  </TableRow>
                )
              })}
        </TableBody>
      </Table>
    </div>
  )
}
