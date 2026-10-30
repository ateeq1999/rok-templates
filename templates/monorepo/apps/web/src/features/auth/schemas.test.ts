import { describe, expect, it } from "vitest"
import { loginSchema, registerSchema, safeRedirect } from "./schemas"

describe("safeRedirect", () => {
  it("allows same-origin paths only", () => {
    expect(safeRedirect("/admin/users?page=2")).toBe("/admin/users?page=2")
    expect(safeRedirect("//evil.com")).toBeUndefined()
    expect(safeRedirect("https://evil.com")).toBeUndefined()
    expect(safeRedirect("/\\evil.com")).toBeUndefined()
    expect(safeRedirect(undefined)).toBeUndefined()
  })
})

describe("schemas", () => {
  it("login does not enforce the password length rule", () => {
    expect(
      loginSchema.safeParse({ email: "a@b.co", password: "x" }).success
    ).toBe(true)
  })

  it("register requires matching passwords of valid length", () => {
    const base = {
      email: "a@b.co",
      password: "Password123!",
      confirm: "Password123!",
    }
    expect(registerSchema.safeParse(base).success).toBe(true)
    expect(registerSchema.safeParse({ ...base, confirm: "nope" }).success).toBe(
      false
    )
    expect(
      registerSchema.safeParse({ ...base, password: "short", confirm: "short" })
        .success
    ).toBe(false)
  })
})
