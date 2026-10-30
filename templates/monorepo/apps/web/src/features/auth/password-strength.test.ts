import { describe, expect, it } from "vitest"
import {
  SERVER_MIN_SCORE,
  checkPasswordStrength,
  cleanKnownInputs,
} from "./password-strength"
import { isLinkProblem, isWrongCurrentPassword } from "./errors"

describe("checkPasswordStrength (advisory, zxcvbn)", () => {
  it("scores a dictionary password below the server threshold", async () => {
    const result = await checkPasswordStrength("password123")
    expect(result.score).toBeLessThan(SERVER_MIN_SCORE)
    expect(result.acceptable).toBe(false)
  })

  it("accepts a long passphrase", async () => {
    const result = await checkPasswordStrength("correct-horse-battery-staple")
    expect(result.score).toBeGreaterThanOrEqual(SERVER_MIN_SCORE)
    expect(result.acceptable).toBe(true)
  })

  it("penalizes a password built from known inputs like the email", async () => {
    const plain = await checkPasswordStrength("alicewonder2024")
    const informed = await checkPasswordStrength("alicewonder2024", [
      "alicewonder@example.com",
      "Alice Wonder",
    ])
    expect(informed.score).toBeLessThanOrEqual(plain.score)
  })
}, 20_000)

describe("cleanKnownInputs", () => {
  it("drops empty and blank values", () => {
    expect(
      cleanKnownInputs(["a@b.co", "", "  ", null, undefined, " Al "])
    ).toEqual(["a@b.co", "Al"])
  })
})

describe("message helpers", () => {
  it("tells a dead link apart from a password-policy message", () => {
    expect(isLinkProblem("this link is invalid or has expired")).toBe(true)
    expect(isLinkProblem("This password is too easy to guess.")).toBe(false)
    expect(
      isLinkProblem(
        "this password has appeared in a known data breach; choose a different one"
      )
    ).toBe(false)
  })

  it("spots a wrong current password", () => {
    expect(isWrongCurrentPassword("current password is incorrect")).toBe(true)
    expect(isWrongCurrentPassword("This password is too easy to guess.")).toBe(
      false
    )
  })
})
