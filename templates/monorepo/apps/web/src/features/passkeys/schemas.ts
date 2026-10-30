import { z } from "zod"

export const passkeySchema = z.object({
  id: z.string(),
  name: z.string(),
  created_at: z.string(),
  last_used_at: z.string().nullable(),
})
export type Passkey = z.infer<typeof passkeySchema>

export const passkeysSchema = z.array(passkeySchema)

export const beginResponseSchema = z.object({
  challenge_id: z.string(),
  options: z.unknown(),
})

export const nameSchema = z
  .string()
  .trim()
  .min(1, "Required")
  .max(100, "Must be at most 100 characters")
