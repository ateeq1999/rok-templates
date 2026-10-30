import { http } from "@/lib/http"
import { paginatedUsersSchema, userSchema } from "./schemas"
import type { ListParams, Paginated, Role, User } from "./schemas"

export const usersApi = {
  me: async (): Promise<User> =>
    userSchema.parse(await http("/api/v1/users/me")),

  updateMe: async (body: { display_name: string }): Promise<User> =>
    userSchema.parse(await http("/api/v1/users/me", { method: "PATCH", body })),

  list: async (
    params: Partial<ListParams>,
    signal?: AbortSignal
  ): Promise<Paginated<User>> =>
    paginatedUsersSchema.parse(
      await http("/api/v1/users", { query: params, signal })
    ),

  get: async (id: string): Promise<User> =>
    userSchema.parse(await http(`/api/v1/users/${id}`)),

  create: async (body: {
    email: string
    password: string
    display_name?: string
    role: Role
  }): Promise<User> =>
    userSchema.parse(await http("/api/v1/users", { method: "POST", body })),

  update: async (
    id: string,
    body: { display_name?: string; role?: Role; is_active?: boolean }
  ): Promise<User> =>
    userSchema.parse(
      await http(`/api/v1/users/${id}`, { method: "PATCH", body })
    ),

  remove: (id: string): Promise<void> =>
    http(`/api/v1/users/${id}`, { method: "DELETE" }),

  uploadAvatar: async (file: File): Promise<User> =>
    userSchema.parse(
      await http("/api/v1/users/me/avatar", { method: "PUT", rawBody: file })
    ),

  removeAvatar: async (): Promise<User> =>
    userSchema.parse(
      await http("/api/v1/users/me/avatar", { method: "DELETE" })
    ),
}
