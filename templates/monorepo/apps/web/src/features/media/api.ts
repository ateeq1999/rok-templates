import { http, httpBlob } from "@/lib/http"
import { safeUploadName } from "./limits"
import { mediaPageSchema, mediaSchema } from "./schemas"
import type { Media, MediaListParams } from "./schemas"

export const mediaApi = {
  /** Raw bytes as the body; the server ignores Content-Type and sniffs the file. */
  upload: async (file: File): Promise<Media> =>
    mediaSchema.parse(
      await http("/api/v1/media", {
        method: "POST",
        rawBody: file,
        query: { filename: safeUploadName(file.name) },
      })
    ),

  list: async (params: MediaListParams, signal?: AbortSignal) =>
    mediaPageSchema.parse(
      await http("/api/v1/media", { query: params, signal })
    ),

  remove: (id: string) =>
    http<void>(`/api/v1/media/${id}`, { method: "DELETE" }),

  /** The bytes, via the Authorization header. Use with URL.createObjectURL. */
  content: (item: Pick<Media, "url">, signal?: AbortSignal) =>
    httpBlob(item.url, { signal }),
}
