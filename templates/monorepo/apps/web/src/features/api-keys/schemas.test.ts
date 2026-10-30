import { describe, expect, it } from "vitest"
import { createApiKeyFormSchema, expiryDays, isExpired } from "./schemas"

describe("expiryDays", () => {
  it("maps 'never' to undefined", () => {
    expect(expiryDays("never")).toBeUndefined()
  })
  it("maps day strings to numbers", () => {
    expect(expiryDays("30")).toBe(30)
    expect(expiryDays("90")).toBe(90)
    expect(expiryDays("365")).toBe(365)
  })
})

describe("createApiKeyFormSchema name validation", () => {
  const base = { scope: "read" as const, expiry: "90" as const }
  it("rejects an empty name", () => {
    expect(
      createApiKeyFormSchema.safeParse({ ...base, name: "" }).success
    ).toBe(false)
  })
  it("rejects a name over 100 characters", () => {
    expect(
      createApiKeyFormSchema.safeParse({ ...base, name: "x".repeat(101) })
        .success
    ).toBe(false)
  })
  it("accepts a normal name", () => {
    expect(
      createApiKeyFormSchema.safeParse({ ...base, name: "CI runner" }).success
    ).toBe(true)
  })
})

describe("isExpired", () => {
  const now = new Date("2026-06-01T00:00:00Z")
  it("is false when there is no expiry", () => {
    expect(isExpired({ expires_at: null }, now)).toBe(false)
  })
  it("is false right before the boundary and true at/after it", () => {
    expect(isExpired({ expires_at: "2026-06-01T00:00:00.001Z" }, now)).toBe(
      false
    )
    expect(isExpired({ expires_at: "2026-06-01T00:00:00Z" }, now)).toBe(true)
    expect(isExpired({ expires_at: "2026-05-01T00:00:00Z" }, now)).toBe(true)
  })
})
