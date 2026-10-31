import { z } from "zod"
import type { TotpSetupResponse } from "@/lib/client"

/**
 * What enrolling an authenticator app needs (the generated client's
 * `TotpSetupResponse`); its `qr_code_data_uri` is safe as an <img src>.
 */
export type TotpSetup = TotpSetupResponse

/** Exactly the 6 digits from the authenticator app. */
export const enableFormSchema = z.object({
  code: z
    .string()
    .trim()
    .regex(/^\d{6}$/, "Enter the 6-digit code from your app"),
})

export const disableFormSchema = z.object({
  current_password: z
    .string()
    .min(8, "Must be 8-128 characters")
    .max(128, "Must be 8-128 characters"),
})
