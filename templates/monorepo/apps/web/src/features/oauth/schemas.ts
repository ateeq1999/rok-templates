import { z } from "zod"

export const oauthProviderSchema = z.enum(["google", "github"])
export type OAuthProvider = z.infer<typeof oauthProviderSchema>

export const providerSchema = z.object({
  provider: oauthProviderSchema,
  name: z.string(),
  login_url: z.string(),
})
export type Provider = z.infer<typeof providerSchema>

export const providersSchema = z.array(providerSchema)

export const identitySchema = z.object({
  provider: oauthProviderSchema,
  email: z.string().nullable(),
  linked_at: z.string(),
})
export type Identity = z.infer<typeof identitySchema>

export const identitiesSchema = z.array(identitySchema)

export const linkResponseSchema = z.object({ authorize_url: z.string() })

export const callbackSearchSchema = z.object({
  code: z.string().optional().catch(undefined),
  redirect: z.string().optional().catch(undefined),
  linked: z.string().optional().catch(undefined),
  error: z.string().optional().catch(undefined),
})
