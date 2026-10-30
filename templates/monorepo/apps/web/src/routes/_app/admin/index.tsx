import { Link, createFileRoute } from "@tanstack/react-router"
import { useQuery, useSuspenseQuery } from "@tanstack/react-query"
import { format, formatDistanceToNow, parseISO } from "date-fns"
import { Bar, BarChart, CartesianGrid, XAxis } from "recharts"
import { HugeiconsIcon } from "@hugeicons/react"
import type { UserIcon } from "@hugeicons/core-free-icons"
import {
  Alert02Icon,
  MailAdd01Icon,
  UserAdd01Icon,
  UserGroupIcon,
  UserBlock01Icon,
  UserCheck01Icon,
  UserSettings01Icon,
} from "@hugeicons/core-free-icons"
import { ErrorScreen } from "@/components/common/screens"
import { PageHeader } from "@/components/layout/page-header"
import { Badge } from "@/components/ui/badge"
import { buttonVariants } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { Skeleton } from "@/components/ui/skeleton"
import { healthQueryOptions } from "@/features/health/api"
import { statsQueryOptions } from "@/features/stats/queries"
import { UserAvatar } from "@/features/users/components/user-avatar"
import { meQueryOptions } from "@/features/users/queries"
import { userLabel } from "@/features/users/schemas"
import { cn } from "@/lib/utils"

export const Route = createFileRoute("/_app/admin/")({
  component: Dashboard,
})

const chartConfig = {
  count: { label: "Signups", color: "var(--chart-2)" },
} satisfies ChartConfig

type Kpi = {
  label: string
  value: number | undefined
  icon: typeof UserIcon
  hint?: string
  warn?: boolean
}

function Dashboard() {
  const { data: me } = useSuspenseQuery(meQueryOptions)
  const stats = useQuery(statsQueryOptions)

  if (stats.isError) {
    return (
      <ErrorScreen error={stats.error} reset={() => void stats.refetch()} />
    )
  }

  const s = stats.data
  const last7 = s?.signups.slice(-7).reduce((n, d) => n + d.count, 0)
  const last30 = s?.signups.reduce((n, d) => n + d.count, 0)

  const kpis: Kpi[] = [
    {
      label: "Total users",
      value: s?.users.total,
      icon: UserGroupIcon,
      hint: last30 === undefined ? undefined : `+${last30} in 30 days`,
    },
    {
      label: "Active",
      value: s?.users.active,
      icon: UserCheck01Icon,
      hint: last7 === undefined ? undefined : `+${last7} this week`,
    },
    {
      label: "Administrators",
      value: s?.users.admins,
      icon: UserSettings01Icon,
    },
    {
      label: "Deactivated",
      value: s?.users.inactive,
      icon: UserBlock01Icon,
    },
    {
      label: "Unverified emails",
      value: s?.users.unverified,
      icon: Alert02Icon,
      hint: "Awaiting confirmation",
      warn: (s?.users.unverified ?? 0) > 0,
    },
  ]

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title={`Welcome back, ${me.display_name?.split(" ")[0] ?? "admin"}`}
        description={
          s?.truncatedAt
            ? `${format(new Date(), "PPPP")} · based on the most recent ${s.truncatedAt.toLocaleString()} users`
            : format(new Date(), "PPPP")
        }
        actions={
          <>
            <Link
              to="/admin/users"
              search={{ dialog: "invite" }}
              className={buttonVariants({ variant: "outline" })}
            >
              <HugeiconsIcon icon={MailAdd01Icon} data-icon="inline-start" />
              Invite user
            </Link>
            <Link
              to="/admin/users"
              search={{ dialog: "create" }}
              className={buttonVariants()}
            >
              <HugeiconsIcon icon={UserAdd01Icon} data-icon="inline-start" />
              Create user
            </Link>
          </>
        }
      />

      <section
        aria-label="Key numbers"
        className="grid gap-3 sm:grid-cols-2 xl:grid-cols-5"
      >
        {kpis.map((k) => (
          <Card key={k.label} size="sm" className="relative overflow-hidden">
            <CardHeader>
              <CardDescription className="flex items-center justify-between gap-2">
                {k.label}
                <HugeiconsIcon
                  icon={k.icon}
                  className={cn(
                    "size-4",
                    k.warn ? "text-destructive" : "text-muted-foreground"
                  )}
                />
              </CardDescription>
              <CardTitle className="font-heading text-3xl tabular-nums">
                {k.value === undefined ? (
                  <Skeleton className="h-8 w-14" />
                ) : (
                  k.value.toLocaleString()
                )}
              </CardTitle>
              {k.hint && (
                <p
                  className={cn(
                    "text-xs",
                    k.warn ? "text-destructive" : "text-muted-foreground"
                  )}
                >
                  {k.hint}
                </p>
              )}
            </CardHeader>
            {k.warn && (
              <span
                aria-hidden
                className="absolute inset-x-0 top-0 h-0.5 bg-destructive"
              />
            )}
          </Card>
        ))}
      </section>

      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="font-heading">Signups</CardTitle>
            <CardDescription>
              New accounts per day, last 30 days
            </CardDescription>
          </CardHeader>
          <CardContent>
            {s ? (
              <ChartContainer config={chartConfig} className="h-64 w-full">
                <BarChart data={s.signups}>
                  <CartesianGrid vertical={false} />
                  <XAxis
                    dataKey="date"
                    tickLine={false}
                    axisLine={false}
                    minTickGap={24}
                    tickFormatter={(d: string) => format(parseISO(d), "MMM d")}
                  />
                  <ChartTooltip
                    content={
                      <ChartTooltipContent
                        labelFormatter={(d) =>
                          format(parseISO(String(d)), "PPP")
                        }
                      />
                    }
                  />
                  <Bar dataKey="count" fill="var(--color-count)" />
                </BarChart>
              </ChartContainer>
            ) : (
              <Skeleton className="h-64 w-full" />
            )}
          </CardContent>
        </Card>

        <div className="flex flex-col gap-4">
          <SystemStatus />
          <Card className="flex-1">
            <CardHeader>
              <CardTitle className="font-heading">Recent signups</CardTitle>
              <CardDescription>The five newest accounts</CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-1">
              {s
                ? s.recent.map((u) => (
                    <Link
                      key={u.id}
                      to="/admin/users/$userId"
                      params={{ userId: u.id }}
                      className="-mx-2 flex items-center gap-3 px-2 py-2 hover:bg-muted"
                    >
                      <UserAvatar user={u} size="sm" />
                      <span className="flex min-w-0 flex-1 flex-col text-sm">
                        <span className="truncate font-medium">
                          {userLabel(u)}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {formatDistanceToNow(new Date(u.created_at), {
                            addSuffix: true,
                          })}
                        </span>
                      </span>
                      {u.role === "admin" && <Badge>admin</Badge>}
                    </Link>
                  ))
                : Array.from({ length: 5 }, (_, i) => (
                    <Skeleton key={i} className="h-10 w-full" />
                  ))}
              <Link
                to="/admin/users"
                className="mt-2 text-xs text-muted-foreground underline"
              >
                View all users
              </Link>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

function SystemStatus() {
  const { data } = useQuery(healthQueryOptions)
  const state = !data
    ? { text: "Checking…", dot: "bg-muted-foreground" }
    : !data.live
      ? { text: "Down", dot: "bg-destructive" }
      : data.ready
        ? { text: "All systems ready", dot: "bg-emerald-500" }
        : { text: "Database unavailable", dot: "bg-amber-500" }
  return (
    <Card size="sm">
      <CardHeader>
        <CardDescription>System status</CardDescription>
        <CardTitle className="flex items-center gap-2 text-base" role="status">
          <span className={cn("size-2 rounded-full", state.dot)} aria-hidden />
          {state.text}
        </CardTitle>
      </CardHeader>
    </Card>
  )
}
