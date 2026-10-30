import { Outlet, createFileRoute, redirect } from "@tanstack/react-router"
import { meQueryOptions } from "@/features/users/queries"

export const Route = createFileRoute("/_app/admin")({
  beforeLoad: async ({ context }) => {
    const me = await context.queryClient.ensureQueryData(meQueryOptions)
    if (me.role !== "admin") throw redirect({ to: "/profile" })
  },
  component: Outlet,
})
