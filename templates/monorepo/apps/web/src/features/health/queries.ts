import { queryOptions } from "@tanstack/react-query"
import { api } from "@/lib/client"

export const healthQueryOptions = queryOptions({
  queryKey: ["health"] as const,
  queryFn: async () => {
    const [live, ready] = await Promise.allSettled([
      api.health.liveness(),
      api.health.readiness(),
    ])
    return {
      live: live.status === "fulfilled",
      ready: ready.status === "fulfilled",
    }
  },
  refetchInterval: 30_000,
  retry: false,
})
