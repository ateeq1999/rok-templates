import {
  keepPreviousData,
  queryOptions,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query"
import { toast } from "@/components/ui/toast"
import { ApiError } from "@/lib/http"
import { mediaApi } from "./api"
import { validateMediaFile } from "./limits"
import type { Media, MediaListParams } from "./schemas"

export const mediaKeys = {
  all: ["media"] as const,
  lists: () => ["media", "list"] as const,
  list: (p: MediaListParams) => ["media", "list", p] as const,
  blob: (url: string) => ["media", "blob", url] as const,
}

export const mediaListQueryOptions = (params: MediaListParams) =>
  queryOptions({
    queryKey: mediaKeys.list(params),
    queryFn: ({ signal }) => mediaApi.list(params, signal),
    placeholderData: keepPreviousData,
  })

/**
 * Uploads one file at a time (fetch has no progress, so a spinner has to do).
 * Files that fail the pre-check are skipped with the reason; a server
 * rejection shows the server's own message. Resolves with what was stored.
 */
export function useUploadMedia() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (files: File[]) => {
      const stored: Media[] = []
      for (const file of files) {
        const problem = validateMediaFile(file)
        if (problem) {
          toast.add({ type: "error", title: file.name, description: problem })
          continue
        }
        try {
          stored.push(await mediaApi.upload(file))
        } catch (error) {
          if (!(error instanceof ApiError)) throw error
          toast.add({
            type: "error",
            title: file.name,
            description: error.message,
          })
        }
      }
      return stored
    },
    onSuccess: (stored) => {
      if (stored.length > 0) {
        void qc.invalidateQueries({ queryKey: mediaKeys.lists() })
        toast.add({
          type: "success",
          title:
            stored.length === 1
              ? "File uploaded"
              : `${stored.length} files uploaded`,
        })
      }
    },
  })
}

export function useDeleteMedia() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (item: Media) => mediaApi.remove(item.id),
    onSuccess: (_data, item) => {
      qc.removeQueries({ queryKey: mediaKeys.blob(item.url) })
      void qc.invalidateQueries({ queryKey: mediaKeys.lists() })
    },
  })
}
