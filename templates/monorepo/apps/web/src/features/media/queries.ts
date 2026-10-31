import {
  keepPreviousData,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query"
import { toast } from "@/components/ui/toast"
import { ApiError, api, keys, queries } from "@/lib/client"
import { safeUploadName, validateMediaFile } from "./limits"
import type { Media, MediaListParams } from "./schemas"

export const mediaKeys = {
  all: keys.media,
  lists: () => [...keys.media, "list"] as const,
  list: (p: MediaListParams) => queries.media.list(p).queryKey,
  blob: (url: string) => [...keys.media, "blob", url] as const,
}

export const mediaListQueryOptions = (params: MediaListParams) => ({
  ...queries.media.list(params),
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
          stored.push(
            await api.media.upload(file, {
              filename: safeUploadName(file.name),
            })
          )
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
    mutationFn: (item: Media) => api.media.remove(item.id),
    onSuccess: (_data, item) => {
      qc.removeQueries({ queryKey: mediaKeys.blob(item.url) })
      void qc.invalidateQueries({ queryKey: mediaKeys.lists() })
    },
  })
}
