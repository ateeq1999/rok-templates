import { CancelledError } from "@tanstack/react-query"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { ApiError } from "@/lib/http"
import { session } from "@/lib/session"
import { homeFor, requireMe } from "./session-actions"

function jwt(expSecondsFromNow: number) {
  const enc = (o: object) => btoa(JSON.stringify(o)).replace(/=+$/, "")
  return `${enc({ alg: "HS256" })}.${enc({ exp: Math.floor(Date.now() / 1000) + expSecondsFromNow })}.sig`
}

function contextWith(ensureQueryData: () => Promise<unknown>) {
  return { queryClient: { ensureQueryData } } as never
}

/** What requireMe threw, so we can inspect a TanStack `redirect()`. */
async function thrown(promise: Promise<unknown>) {
  try {
    await promise
  } catch (error) {
    return error as {
      options?: { to?: string; search?: Record<string, string> }
    }
  }
  return undefined
}

beforeEach(() => session.clear())
afterEach(() => session.clear())

describe("homeFor", () => {
  it("sends admins to the dashboard and everyone else to their profile", () => {
    expect(homeFor({ role: "admin" })).toBe("/admin")
    expect(homeFor({ role: "user" })).toBe("/profile")
  })
})

describe("requireMe", () => {
  it("redirects to /login without a token, keeping where the user was going", async () => {
    const err = await thrown(
      requireMe(contextWith(vi.fn()), { href: "/admin/audit-log?page=2" })
    )
    expect(err?.options?.to).toBe("/login")
    expect(err?.options?.search?.redirect).toBe("/admin/audit-log?page=2")
  })

  it("redirects with a session-expired notice when the API answers 401", async () => {
    session.login(jwt(3600))
    const err = await thrown(
      requireMe(
        contextWith(() =>
          Promise.reject(new ApiError(401, "unauthorized", "expired"))
        )
      )
    )
    expect(err?.options?.to).toBe("/login")
    expect(err?.options?.search?.reason).toBe("session-expired")
  })

  it("still lands on /login when the 401 handler cancels the request mid-flight", async () => {
    // A 401 clears the session and the query cache; cancelling the in-flight
    // `me` request surfaces as a CancelledError, not as the ApiError.
    session.login(jwt(3600))
    const err = await thrown(
      requireMe(
        contextWith(async () => {
          session.clear()
          throw new CancelledError()
        })
      )
    )
    expect(err?.options?.to).toBe("/login")
    expect(err?.options?.search?.reason).toBe("session-expired")
  })

  it("does not swallow other failures while the session is still valid", async () => {
    session.login(jwt(3600))
    const boom = new ApiError(500, "internal_error", "boom")
    const err = await thrown(requireMe(contextWith(() => Promise.reject(boom))))
    expect(err).toBe(boom)
  })

  it("returns the user when the request succeeds", async () => {
    session.login(jwt(3600))
    const me = { id: "1", role: "user" }
    await expect(requireMe(contextWith(async () => me))).resolves.toBe(me)
  })
})
