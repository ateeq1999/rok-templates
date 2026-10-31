import { z } from "zod"
import type { MediaResponse } from "@/lib/client"

/**
 * A stored file's metadata (the generated client's `MediaResponse`). Its
 * `url` needs the Authorization header, so <img src> cannot use it: the
 * bytes come through `api.media.content`.
 */
export type Media = MediaResponse

export const mediaSearchSchema = z.object({
  page: z.number().int().min(1).catch(1).default(1),
  per_page: z
    .number()
    .int()
    .refine((n) => [12, 24, 48].includes(n))
    .catch(12)
    .default(12),
})
export type MediaListParams = z.infer<typeof mediaSearchSchema>

/** A name to save under when downloading. */
export function downloadName(item: Media): string {
  return item.original_filename?.trim() || item.id
}
