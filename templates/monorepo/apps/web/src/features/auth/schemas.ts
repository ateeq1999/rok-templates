import { z } from "zod"

export const passwordSchema = z
  .string()
  .min(8, "Must be 8-128 characters")
  .max(128, "Must be 8-128 characters")

const emailSchema = z.email("Enter a valid email")

// Login deliberately skips the length rule: a wrong-length password should get
// "invalid email or password", not a hint about the rule.
export const loginSchema = z.object({
  email: emailSchema,
  password: z.string().min(1, "Required"),
})

export const registerSchema = z
  .object({ email: emailSchema, password: passwordSchema, confirm: z.string() })
  .refine((v) => v.password === v.confirm, {
    path: ["confirm"],
    message: "Passwords do not match",
  })

export const forgotSchema = z.object({ email: emailSchema })

export const resetSchema = z
  .object({ password: passwordSchema, confirm: z.string() })
  .refine((v) => v.password === v.confirm, {
    path: ["confirm"],
    message: "Passwords do not match",
  })

export const changePasswordSchema = z
  .object({
    current_password: z.string().min(1, "Required"),
    new_password: passwordSchema,
    confirm: z.string(),
  })
  .refine((v) => v.new_password === v.confirm, {
    path: ["confirm"],
    message: "Passwords do not match",
  })

export const changeEmailSchema = z.object({
  new_email: emailSchema,
  current_password: z.string().min(1, "Required"),
})

export const tokenResponseSchema = z.object({
  access_token: z.string(),
  token_type: z.string(),
  expires_in: z.number(),
})

/** Password login for an account with two-factor on: no session yet, only a short-lived token. */
export const totpRequiredSchema = z.object({
  requires_totp: z.literal(true),
  pending_token: z.string(),
})

/** `POST /auth/login` returns either a session or the two-factor marker. */
export const loginResponseSchema = z.union([
  totpRequiredSchema,
  tokenResponseSchema,
])
export type LoginResponse = z.infer<typeof loginResponseSchema>

// Two-factor sign-in step: an authenticator code, or a recovery code.
export const authenticatorCodeSchema = z
  .string()
  .trim()
  .regex(/^\d{6}$/, "Enter the 6-digit code")

export const recoveryCodeSchema = z
  .string()
  .transform(normalizeRecoveryCode)
  .pipe(
    z
      .string()
      .regex(
        /^[A-Z0-9]{5}-[A-Z0-9]{5}$/,
        "Recovery codes look like XXXXX-XXXXX"
      )
  )

/**
 * The server hashes the exact string it issued (uppercase, one hyphen), so
 * tidy what people type: trim, uppercase, and add the hyphen if it was left out.
 */
export function normalizeRecoveryCode(input: string): string {
  const compact = input.trim().toUpperCase().replace(/\s+/g, "")
  if (/^[A-Z0-9]{10}$/.test(compact))
    return `${compact.slice(0, 5)}-${compact.slice(5)}`
  return compact
}

export type TotpCodeMode = "authenticator" | "recovery"

export const totpCodeFormSchema = (mode: TotpCodeMode) =>
  z.object({
    code: mode === "recovery" ? recoveryCodeSchema : authenticatorCodeSchema,
  })

/** Only same-origin absolute paths are accepted as post-login targets. */
export function safeRedirect(target: string | undefined): string | undefined {
  if (!target || !target.startsWith("/") || target.startsWith("//")) return
  if (target.includes("\\")) return
  return target
}
