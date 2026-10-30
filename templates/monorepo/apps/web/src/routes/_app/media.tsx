import { createFileRoute } from "@tanstack/react-router"
import { MediaLibrary } from "@/features/media/components/media-library"
import { mediaListQueryOptions } from "@/features/media/queries"
import { mediaSearchSchema } from "@/features/media/schemas"

export const Route = createFileRoute("/_app/media")({
  validateSearch: mediaSearchSchema,
  loaderDeps: ({ search }) => search,
  loader: ({ context, deps }) => {
    // warm the cache without blocking navigation on the list
    void context.queryClient.prefetchQuery(mediaListQueryOptions(deps))
  },
  component: MediaPage,
})

function MediaPage() {
  const params = Route.useSearch()
  const navigate = Route.useNavigate()
  return (
    <MediaLibrary
      params={params}
      onPageChange={(page) =>
        void navigate({ search: (prev) => ({ ...prev, page }) })
      }
    />
  )
}
