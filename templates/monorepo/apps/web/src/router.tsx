import { createRouter as createTanStackRouter } from "@tanstack/react-router"
import type { AnyRouter } from "@tanstack/react-router"
import { QueryClientProvider } from "@tanstack/react-query"
import type { QueryClient } from "@tanstack/react-query"
import { routeTree } from "./routeTree.gen"
import { createQueryClient } from "@/lib/query-client"
import { session } from "@/lib/session"
import { toast } from "@/components/ui/toast"
import { ErrorScreen, NotFoundScreen } from "@/components/common/screens"

export type RouterContext = { queryClient: QueryClient }

let wired = false

/** Reacts to the session ending (401, timer, another tab) from anywhere. */
function wireSession(router: AnyRouter, queryClient: QueryClient) {
  if (wired || typeof window === "undefined") return
  wired = true
  session.onExpire(() => {
    queryClient.clear()
    const { pathname, search, hash } = router.state.location
    const here = `${pathname}${search}${hash}`
    if (pathname === "/login") return
    void router.navigate({
      to: "/login",
      search: { redirect: here, reason: "session-expired" },
    })
  })
  session.onAboutToExpire(() => {
    toast.add({
      type: "warning",
      title: "Your session is about to expire",
      description: "Sign in again to keep working.",
    })
  })
}

export function getRouter() {
  const queryClient = createQueryClient()

  const router = createTanStackRouter({
    routeTree,
    context: { queryClient },
    scrollRestoration: true,
    defaultPreload: "intent",
    defaultPreloadStaleTime: 0,
    defaultErrorComponent: ErrorScreen,
    defaultNotFoundComponent: NotFoundScreen,
    Wrap: ({ children }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    ),
  })

  wireSession(router, queryClient)
  return router
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof getRouter>
  }
}
