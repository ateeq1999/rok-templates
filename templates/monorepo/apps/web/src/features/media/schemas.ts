import { z } from "zod"

export const mediaSchema = z.object({
  id: z.string(),
  content_type: z.string(),
  size_bytes: z.number(),
  original_filename: z.string().nullable(),
  /** Path to the bytes. It needs the Authorization header, so <img src> cannot use it. */
  url: z.string(),
  created_at: z.string(),
})
export type Media = z.infer<typeof mediaSchema>

export const mediaPageSchema = z.object({
  items: z.array(mediaSchema),
  page: z.number(),
  per_page: z.number(),
  total: z.number(),
})

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
