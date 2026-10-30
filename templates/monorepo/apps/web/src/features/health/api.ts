import { queryOptions } from "@tanstack/react-query"
import { http } from "@/lib/http"

export const healthQueryOptions = queryOptions({
  queryKey: ["health"] as const,
  queryFn: async () => {
    const [live, ready] = await Promise.allSettled([
      http("/health/live", { skipExpire: true }),
      http("/health/ready", { skipExpire: true }),
    ])
    return {
      live: live.status === "fulfilled",
      ready: ready.status === "fulfilled",
    }
  },
  refetchInterval: 30_000,
  retry: false,
})
