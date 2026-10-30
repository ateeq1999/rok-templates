import { beforeEach, describe, expect, it } from "vitest"
import { session } from "../session"

function jwt(exp: number) {
  const enc = (o: object) => btoa(JSON.stringify(o)).replace(/=+$/, "")
  return `${enc({ alg: "HS256" })}.${enc({ exp })}.sig`
}

beforeEach(() => session.clear())

describe("session", () => {
  it("has no valid token when empty", () => {
    expect(session.hasValidToken()).toBe(false)
  })

  it("accepts a token whose exp is in the future", () => {
    session.login(jwt(Math.floor(Date.now() / 1000) + 3600))
    expect(session.hasValidToken()).toBe(true)
  })

  it("rejects an expired token", () => {
    session.login(jwt(Math.floor(Date.now() / 1000) - 10))
    expect(session.hasValidToken()).toBe(false)
  })

  it("persists to localStorage and clears on logout", () => {
    session.login(jwt(Math.floor(Date.now() / 1000) + 3600))
    expect(window.localStorage.getItem("session.token")).not.toBeNull()
    session.clear()
    expect(window.localStorage.getItem("session.token")).toBeNull()
  })
})
