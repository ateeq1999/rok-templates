import { Link } from "@tanstack/react-router"
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table"
import type { ColumnDef } from "@tanstack/react-table"
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
import { UserAvatar } from "@/features/users/components/user-avatar"
import { userLabel } from "@/features/users/schemas"
import type { User } from "@/features/users/schemas"
import { UserActions } from "./user-actions"

const columns: ColumnDef<User>[] = [
  {
    id: "user",
    header: "User",
    cell: ({ row }) => {
      const u = row.original
      return (
        <Link
          to="/admin/users/$userId"
          params={{ userId: u.id }}
          className="flex items-center gap-2 hover:underline"
        >
          <UserAvatar user={u} size="sm" />
          <span className="flex min-w-0 flex-col">
            <span className="truncate font-medium">{userLabel(u)}</span>
            {u.display_name && (
              <span className="truncate text-muted-foreground">{u.email}</span>
            )}
          </span>
        </Link>
      )
    },
  },
  {
    accessorKey: "role",
    header: "Role",
    cell: ({ getValue }) => (
      <Badge variant={getValue() === "admin" ? "default" : "secondary"}>
        {String(getValue())}
      </Badge>
    ),
  },
  {
    accessorKey: "is_active",
    header: "Status",
    cell: ({ getValue }) => (
      <Badge variant={getValue() ? "secondary" : "destructive"}>
        {getValue() ? "Active" : "Deactivated"}
      </Badge>
    ),
  },
  {
    accessorKey: "email_verified",
    header: "Verified",
    cell: ({ getValue }) => (
      <span className="text-xs">{getValue() ? "Yes" : "No"}</span>
    ),
  },
  {
    accessorKey: "created_at",
    header: "Created",
    cell: ({ getValue }) => {
      const date = new Date(String(getValue()))
      return (
        <Tooltip>
          <TooltipTrigger render={<span />}>
            {formatDistanceToNow(date, { addSuffix: true })}
          </TooltipTrigger>
          <TooltipContent>{format(date, "PPpp")}</TooltipContent>
        </Tooltip>
      )
    },
  },
  {
    id: "actions",
    header: () => <span className="sr-only">Actions</span>,
    cell: ({ row }) => <UserActions user={row.original} />,
  },
]

export function UsersTable({
  data,
  total,
  loading,
  pageSize,
}: {
  data: User[]
  total: number
  loading: boolean
  pageSize: number
}) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
    manualSorting: true,
    rowCount: total,
    getRowId: (u) => u.id,
  })

  return (
    <div className="overflow-x-auto border">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((hg) => (
            <TableRow key={hg.id}>
              {hg.headers.map((h) => (
                <TableHead key={h.id}>
                  {flexRender(h.column.columnDef.header, h.getContext())}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {loading
            ? Array.from({ length: Math.min(pageSize, 8) }, (_, i) => (
                <TableRow key={i}>
                  {columns.map((_c, j) => (
                    <TableCell key={j}>
                      <Skeleton className="h-4 w-full max-w-32" />
                    </TableCell>
                  ))}
                </TableRow>
              ))
            : table.getRowModel().rows.map((row) => (
                <TableRow key={row.id}>
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))}
        </TableBody>
      </Table>
    </div>
  )
}
