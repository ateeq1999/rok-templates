import { usersApi } from "@/features/users/api"
import type { User } from "@/features/users/schemas"

export type Stats = {
  users: {
    total: number
    admins: number
    active: number
    inactive: number
    unverified: number
  }
  signups: { date: string; count: number }[]
  recent: User[]
  /** Set when the client-side fallback hit its page cap. */
  truncatedAt?: number
}

const PAGE_SIZE = 100
const MAX_PAGES = 50
const DAY_MS = 86_400_000

function dayKey(d: Date) {
  return d.toISOString().slice(0, 10)
}

export function computeStats(
  users: User[],
  total: number,
  now = new Date()
): Stats {
  const counts = new Map<string, number>()
  for (const u of users) {
    const key = dayKey(new Date(u.created_at))
    counts.set(key, (counts.get(key) ?? 0) + 1)
  }
  const signups = Array.from({ length: 30 }, (_, i) => {
    const date = dayKey(new Date(now.getTime() - (29 - i) * DAY_MS))
    return { date, count: counts.get(date) ?? 0 }
  })

  const recent = [...users]
    .sort((a, b) => b.created_at.localeCompare(a.created_at))
    .slice(0, 5)

  return {
    users: {
      total,
      admins: users.filter((u) => u.role === "admin").length,
      active: users.filter((u) => u.is_active).length,
      inactive: users.filter((u) => !u.is_active).length,
      unverified: users.filter((u) => !u.email_verified).length,
    },
    signups,
    recent,
    truncatedAt: users.length < total ? users.length : undefined,
  }
}

/** Interim stats: page through the list endpoint until a real stats endpoint exists. */
export async function fetchStatsClientSide(): Promise<Stats> {
  const users: User[] = []
  let total = 0
  for (let page = 1; page <= MAX_PAGES; page++) {
    const res = await usersApi.list({
      page,
      per_page: PAGE_SIZE,
      sort: "created_at",
      order: "desc",
    })
    total = res.total
    users.push(...res.items)
    if (users.length >= total || res.items.length === 0) break
  }
  return computeStats(users, total)
}
