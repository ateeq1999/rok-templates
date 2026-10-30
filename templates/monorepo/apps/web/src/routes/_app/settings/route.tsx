import {
  Link,
  Outlet,
  createFileRoute,
  useMatchRoute,
} from "@tanstack/react-router"
import { PageHeader } from "@/components/layout/page-header"
import { cn } from "@/lib/utils"

export const Route = createFileRoute("/_app/settings")({
  component: SettingsLayout,
})

const tabs = [
  { to: "/settings/account", label: "Account" },
  { to: "/settings/security", label: "Security" },
  { to: "/settings/api-keys", label: "API keys" },
  { to: "/settings/appearance", label: "Appearance" },
] as const

function SettingsLayout() {
  const matchRoute = useMatchRoute()
  return (
    <div className="flex max-w-2xl flex-col gap-4">
      <PageHeader title="Settings" />
      <nav aria-label="Settings" className="flex gap-1 border-b">
        {tabs.map((tab) => {
          const active = Boolean(matchRoute({ to: tab.to }))
          return (
            <Link
              key={tab.to}
              to={tab.to}
              aria-current={active ? "page" : undefined}
              className={cn(
                "-mb-px border-b-2 px-3 py-2 text-xs font-medium",
                active
                  ? "border-primary text-foreground"
                  : "border-transparent text-muted-foreground hover:text-foreground"
              )}
            >
              {tab.label}
            </Link>
          )
        })}
      </nav>
      <Outlet />
    </div>
  )
}
