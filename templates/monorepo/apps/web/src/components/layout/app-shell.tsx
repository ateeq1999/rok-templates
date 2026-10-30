import { Link, Outlet, useMatchRoute } from "@tanstack/react-router"
import { useSuspenseQuery } from "@tanstack/react-query"
import { HugeiconsIcon } from "@hugeicons/react"
import {
  DashboardSquare01Icon,
  Files01Icon,
  Task01Icon,
  Logout01Icon,
  Moon02Icon,
  Settings01Icon,
  Sun01Icon,
  UnfoldMoreIcon,
  UserGroupIcon,
  UserIcon,
} from "@hugeicons/core-free-icons"
import { Logo } from "@/components/brand/logo"
import { Breadcrumbs } from "./breadcrumbs"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Separator } from "@/components/ui/separator"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
  useSidebar,
} from "@/components/ui/sidebar"
import { useLogout } from "@/features/auth/session-actions"
import { UserAvatar } from "@/features/users/components/user-avatar"
import { meQueryOptions } from "@/features/users/queries"
import { userLabel } from "@/features/users/schemas"
import { useTheme } from "./theme"
import { VerifyEmailBanner } from "@/features/users/components/verify-banner"

type NavItem = {
  to:
    | "/profile"
    | "/media"
    | "/settings/account"
    | "/admin"
    | "/admin/users"
    | "/admin/audit-log"
  label: string
  icon: typeof UserIcon
  match: string
  exact?: boolean
}

const userNav: NavItem[] = [
  { to: "/profile", label: "Profile", icon: UserIcon, match: "/profile" },
  { to: "/media", label: "Media", icon: Files01Icon, match: "/media" },
  {
    to: "/settings/account",
    label: "Settings",
    icon: Settings01Icon,
    match: "/settings",
  },
]

const adminNav: NavItem[] = [
  {
    to: "/admin",
    label: "Dashboard",
    icon: DashboardSquare01Icon,
    match: "/admin",
    exact: true,
  },
  {
    to: "/admin/users",
    label: "Users",
    icon: UserGroupIcon,
    match: "/admin/users",
  },
  {
    to: "/admin/audit-log",
    label: "Audit log",
    icon: Task01Icon,
    match: "/admin/audit-log",
  },
]

function NavGroup({ label, items }: { label: string; items: NavItem[] }) {
  const matchRoute = useMatchRoute()
  const { isMobile, setOpenMobile } = useSidebar()
  return (
    <SidebarGroup>
      <SidebarGroupLabel>{label}</SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item) => (
          <SidebarMenuItem key={item.to}>
            <SidebarMenuButton
              tooltip={item.label}
              isActive={Boolean(
                matchRoute({ to: item.match, fuzzy: !item.exact })
              )}
              render={
                <Link
                  to={item.to}
                  onClick={() => isMobile && setOpenMobile(false)}
                />
              }
            >
              <HugeiconsIcon icon={item.icon} />
              <span>{item.label}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  )
}

function UserMenu() {
  const { data: me } = useSuspenseQuery(meQueryOptions)
  const logout = useLogout()
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <SidebarMenuButton size="lg" className="w-full">
            <UserAvatar user={me} size="sm" />
            <div className="grid flex-1 text-left leading-tight">
              <span className="truncate font-medium">{userLabel(me)}</span>
              <span className="truncate text-muted-foreground">{me.email}</span>
            </div>
            <HugeiconsIcon icon={UnfoldMoreIcon} className="ml-auto" />
          </SidebarMenuButton>
        }
      />
      <DropdownMenuContent side="top" align="start" className="min-w-56">
        <DropdownMenuGroup>
          <DropdownMenuLabel className="flex items-center justify-between gap-2">
            <span className="truncate">{userLabel(me)}</span>
            <Badge variant={me.role === "admin" ? "default" : "secondary"}>
              {me.role}
            </Badge>
          </DropdownMenuLabel>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem render={<Link to="/profile" />}>
          <HugeiconsIcon icon={UserIcon} /> Profile
        </DropdownMenuItem>
        <DropdownMenuItem render={<Link to="/settings/account" />}>
          <HugeiconsIcon icon={Settings01Icon} /> Settings
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => void logout()}>
          <HugeiconsIcon icon={Logout01Icon} /> Sign out
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

function ThemeToggle() {
  const { theme, setTheme } = useTheme()
  const dark =
    theme === "dark" ||
    (theme === "system" &&
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches)
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label={dark ? "Switch to light theme" : "Switch to dark theme"}
      onClick={() => setTheme(dark ? "light" : "dark")}
    >
      <HugeiconsIcon icon={dark ? Sun01Icon : Moon02Icon} />
    </Button>
  )
}

export function AppShell() {
  const { data: me } = useSuspenseQuery(meQueryOptions)
  return (
    <SidebarProvider>
      <Sidebar collapsible="icon">
        <SidebarHeader className="px-3 py-3">
          <Logo />
        </SidebarHeader>
        <SidebarContent>
          <NavGroup label="Account" items={userNav} />
          {me.role === "admin" && <NavGroup label="Admin" items={adminNav} />}
        </SidebarContent>
        <SidebarFooter>
          <SidebarMenu>
            <SidebarMenuItem>
              <UserMenu />
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarFooter>
        <SidebarRail />
      </Sidebar>
      <SidebarInset>
        <header className="sticky top-0 z-10 flex h-12 items-center gap-2 border-b bg-background/85 px-4 backdrop-blur">
          <SidebarTrigger />
          <Separator orientation="vertical" className="h-4" />
          <Breadcrumbs />
          <div className="ml-auto">
            <ThemeToggle />
          </div>
        </header>
        <div className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-6 p-4 md:p-6">
          {!me.email_verified && <VerifyEmailBanner />}
          <Outlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}
