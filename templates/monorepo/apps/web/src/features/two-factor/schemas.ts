import { z } from "zod"

export const totpSetupSchema = z.object({
  secret: z.string(),
  /** A data URI: safe to use directly as an <img src>. */
  qr_code_data_uri: z.string(),
  provisioning_uri: z.string(),
})
export type TotpSetup = z.infer<typeof totpSetupSchema>

export const totpEnabledSchema = z.object({
  recovery_codes: z.array(z.string()),
})

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
