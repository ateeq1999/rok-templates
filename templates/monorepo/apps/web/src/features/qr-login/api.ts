import { http } from "@/lib/http"
import { createQrSessionSchema, qrPollSchema, qrScanSchema } from "./schemas"

export const qrLoginApi = {
  create: async () =>
    createQrSessionSchema.parse(
      await http("/api/v1/auth/qr/sessions", {
        method: "POST",
        skipExpire: true,
      })
    ),

  poll: async (id: string, pollSecret: string, signal?: AbortSignal) =>
    qrPollSchema.parse(
      await http(`/api/v1/auth/qr/sessions/${id}`, {
        headers: { "X-QR-Secret": pollSecret },
        skipExpire: true,
        signal,
      })
    ),

  scan: async (id: string) =>
    qrScanSchema.parse(
      await http(`/api/v1/auth/qr/sessions/${id}/scan`, { method: "POST" })
    ),

  approve: (id: string, code: string) =>
    http<{ message: string }>(`/api/v1/auth/qr/sessions/${id}/approve`, {
      method: "POST",
      body: { code },
    }),

  reject: (id: string) =>
    http<{ message: string }>(`/api/v1/auth/qr/sessions/${id}/reject`, {
      method: "POST",
    }),
}
