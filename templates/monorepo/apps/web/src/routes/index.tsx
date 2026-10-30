import { createFileRoute } from "@tanstack/react-router"
import { Landing } from "@/components/landing/landing"

export const Route = createFileRoute("/")({
  head: () => ({
    meta: [
      { title: "Starter: auth, profile and admin for your Rust API" },
      {
        name: "description",
        content:
          "A TanStack and shadcn/ui frontend with every auth flow, profile settings and an admin console.",
      },
    ],
  }),
  component: Landing,
})
