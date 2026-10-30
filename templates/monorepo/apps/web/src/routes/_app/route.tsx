import { createFileRoute } from "@tanstack/react-router"
import { AppShell } from "@/components/layout/app-shell"
import { requireMe } from "@/features/auth/session-actions"

export const Route = createFileRoute("/_app")({
  beforeLoad: ({ context, location }) => requireMe(context, location),
  component: AppShell,
})
