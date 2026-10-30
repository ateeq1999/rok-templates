import { z } from "zod"

export const roleSchema = z.enum(["user", "admin"])
export type Role = z.infer<typeof roleSchema>

export const userSchema = z.object({
  id: z.string(),
  email: z.string(),
  display_name: z.string().nullable(),
  role: roleSchema,
  is_active: z.boolean(),
  email_verified: z.boolean(),
  created_at: z.string(),
  avatar_url: z.string().nullable().default(null),
  has_password: z.boolean().default(true),
  totp_enabled: z.boolean().default(false),
})
export type User = z.infer<typeof userSchema>

export type Paginated<T> = {
  items: T[]
  page: number
  per_page: number
  total: number
}

export const paginatedUsersSchema = z.object({
  items: z.array(userSchema),
  page: z.number(),
  per_page: z.number(),
  total: z.number(),
})

export const displayNameSchema = z
  .string()
  .trim()
  .min(1, "Required")
  .max(100, "Must be at most 100 characters")

export const listSearchSchema = z.object({
  page: z.number().int().min(1).catch(1).default(1),
  per_page: z
    .number()
    .int()
    .refine((n) => [10, 20, 50, 100].includes(n))
    .catch(20)
    .default(20),
  q: z.string().optional().catch(undefined),
  sort: z
    .enum(["email", "created_at"])
    .catch("created_at")
    .default("created_at"),
  order: z.enum(["asc", "desc"]).catch("desc").default("desc"),
})
export type ListParams = z.infer<typeof listSearchSchema>

export const createUserSchema = z.object({
  email: z.email("Enter a valid email"),
  password: z
    .string()
    .min(8, "Must be 8-128 characters")
    .max(128, "Must be 8-128 characters"),
  display_name: z.string().max(100, "Must be at most 100 characters"),
  role: roleSchema,
})

export const inviteUserSchema = z.object({
  email: z.email("Enter a valid email"),
  display_name: z.string().max(100, "Must be at most 100 characters"),
  role: roleSchema,
})

export const editUserSchema = z.object({
  display_name: z.string().max(100, "Must be at most 100 characters"),
  role: roleSchema,
  is_active: z.boolean(),
})

export function userLabel(user: Pick<User, "display_name" | "email">) {
  return user.display_name?.trim() || user.email
}

export function initials(user: Pick<User, "display_name" | "email">) {
  const label = userLabel(user)
  const parts = label.split(/[\s@.]+/).filter(Boolean)
  return ((parts[0]?.[0] ?? "?") + (parts[1]?.[0] ?? "")).toUpperCase()
}
