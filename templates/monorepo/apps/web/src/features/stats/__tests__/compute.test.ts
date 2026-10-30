import { describe, expect, it } from "vitest"
import { computeStats } from "../compute"
import type { User } from "@/features/users/schemas"

const now = new Date("2026-09-21T12:00:00Z")
const mk = (id: string, over: Partial<User>): User => ({
  id,
  email: `${id}@example.com`,
  display_name: null,
  role: "user",
  is_active: true,
  email_verified: true,
  created_at: "2026-09-01T00:00:00Z",
  avatar_url: null,
  has_password: true,
  totp_enabled: false,
  ...over,
})

// mirrors the seed data: 5 users, 1 admin, 1 deactivated, 1 unverified
const seed = [
  mk("admin", { role: "admin", created_at: "2026-09-21T01:00:00Z" }),
  mk("alice", { created_at: "2026-09-20T01:00:00Z" }),
  mk("bob", { created_at: "2026-09-20T02:00:00Z" }),
  mk("carol", { email_verified: false }),
  mk("dave", { is_active: false }),
]

describe("computeStats", () => {
  const stats = computeStats(seed, 5, now)

  it("counts users", () => {
    expect(stats.users).toEqual({
      total: 5,
      admins: 1,
      active: 4,
      inactive: 1,
      unverified: 1,
    })
  })

  it("zero-fills 30 days of signups ending today", () => {
    expect(stats.signups).toHaveLength(30)
    expect(stats.signups.at(-1)).toEqual({ date: "2026-09-21", count: 1 })
    expect(stats.signups.find((d) => d.date === "2026-09-20")?.count).toBe(2)
    expect(stats.signups.reduce((n, d) => n + d.count, 0)).toBe(5)
  })

  it("lists the five newest first and flags truncation", () => {
    expect(stats.recent[0].id).toBe("admin")
    expect(stats.truncatedAt).toBeUndefined()
    expect(computeStats(seed, 9, now).truncatedAt).toBe(5)
  })
})
