import { createFileRoute, redirect } from "@tanstack/react-router"

// The path the SPA shell (`_shell.html`) is rendered at during the build
// (`SHELL_PATH` in vite.config.ts). The build renders only the root route
// here, so the shell carries no page of its own; a visitor who opens this
// path is sent home.
export const Route = createFileRoute("/_shell")({
  beforeLoad: () => {
    throw redirect({ to: "/" })
  },
})
