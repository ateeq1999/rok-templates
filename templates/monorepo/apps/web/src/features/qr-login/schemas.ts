import { z } from "zod"

export const qrStatusSchema = z.enum([
  "pending",
  "scanned",
  "approved",
  "rejected",
  "expired",
  "consumed",
])
export type QrStatus = z.infer<typeof qrStatusSchema>

export const createQrSessionSchema = z.object({
  id: z.string(),
  poll_secret: z.string(),
  verification_code: z.string(),
  qr_payload: z.string(),
  qr_svg: z.string(),
  expires_at: z.string(),
  poll_interval_secs: z.number(),
})
export type CreatedQrSession = z.infer<typeof createQrSessionSchema>

export const qrPollSchema = z.object({
  status: qrStatusSchema,
  access_token: z.string().optional(),
  token_type: z.string().optional(),
  expires_in: z.number().optional(),
})

export const qrScanSchema = z.object({
  requester_ip: z.string().nullable(),
  requester_agent: z.string().nullable(),
  requested_at: z.string(),
  expires_at: z.string(),
})
export type QrScan = z.infer<typeof qrScanSchema>
