import { Fragment } from "react"
import { Link, useRouterState } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import { userDetailQueryOptions } from "@/features/users/queries"
import { userLabel } from "@/features/users/schemas"

type Crumb = { label: string; to?: string }

const settingsTabs: Record<string, string> = {
  account: "Account",
  security: "Security",
  appearance: "Appearance",
}

/** Label of the user in /admin/users/$userId, from the cache when possible. */
function useUserCrumb(id: string | undefined) {
  const { data } = useQuery({
    ...userDetailQueryOptions(id ?? ""),
    enabled: Boolean(id),
  })
  return data ? userLabel(data) : "User"
}

function crumbsFor(segments: string[], userName: string): Crumb[] {
  const [area, second, third] = segments
  if (area === "profile") return [{ label: "Profile" }]
  if (area === "media") return [{ label: "Media" }]
  if (area === "settings")
    return [
      { label: "Settings", to: "/settings/account" },
      { label: settingsTabs[second ?? "account"] ?? "Account" },
    ]
  if (area === "admin") {
    if (second === "audit-log")
      return [{ label: "Admin", to: "/admin" }, { label: "Audit log" }]
    if (second === "users") {
      return third
        ? [
            { label: "Admin", to: "/admin" },
            { label: "Users", to: "/admin/users" },
            { label: userName },
          ]
        : [{ label: "Admin", to: "/admin" }, { label: "Users" }]
    }
    return [{ label: "Admin", to: "/admin" }, { label: "Dashboard" }]
  }
  return []
}

export function Breadcrumbs() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const segments = pathname.split("/").filter(Boolean)
  const userId =
    segments[0] === "admin" && segments[1] === "users" ? segments[2] : undefined
  const userName = useUserCrumb(userId)
  const crumbs = crumbsFor(segments, userName)
  if (crumbs.length === 0) return null

  return (
    <Breadcrumb>
      <BreadcrumbList>
        {crumbs.map((crumb, i) => {
          const last = i === crumbs.length - 1
          return (
            <Fragment key={`${crumb.label}-${i}`}>
              {i > 0 && <BreadcrumbSeparator />}
              <BreadcrumbItem>
                {last || !crumb.to ? (
                  <BreadcrumbPage className="max-w-40 truncate">
                    {crumb.label}
                  </BreadcrumbPage>
                ) : (
                  <BreadcrumbLink render={<Link to={crumb.to} />}>
                    {crumb.label}
                  </BreadcrumbLink>
                )}
              </BreadcrumbItem>
            </Fragment>
          )
        })}
      </BreadcrumbList>
    </Breadcrumb>
  )
}
