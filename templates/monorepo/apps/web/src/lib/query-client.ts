import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query"
import { toast } from "@/components/ui/toast"
import { ApiError } from "./http"

function notify(error: unknown) {
  if (
    error instanceof ApiError &&
    (error.status === 0 || error.status >= 500)
  ) {
    toast.add({
      type: "error",
      title: error.status === 0 ? "Network error" : "Server error",
      description: error.message,
    })
  }
}

export function createQueryClient() {
  return new QueryClient({
    queryCache: new QueryCache({ onError: notify }),
    mutationCache: new MutationCache({ onError: notify }),
    defaultOptions: {
      queries: {
        staleTime: 30_000,
        refetchOnWindowFocus: true,
        retry: (count, error) => {
          if (
            error instanceof ApiError &&
            error.status >= 400 &&
            error.status < 500
          )
            return false
          return count < 2
        },
      },
    },
  })
}
