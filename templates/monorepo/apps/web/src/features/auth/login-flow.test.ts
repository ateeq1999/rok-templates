import { describe, expect, it } from "vitest"
import { ApiError } from "@/lib/client"
import type { LoginResponse } from "@/lib/client"
import { loginStepFor, totpVerifyOutcome } from "./login-flow"
import {
  authenticatorCodeSchema,
  normalizeRecoveryCode,
  recoveryCodeSchema,
  totpCodeFormSchema,
} from "./schemas"

const session: LoginResponse = {
  access_token: "jwt.token.here",
  token_type: "Bearer",
  expires_in: 3600,
}
const totp: LoginResponse = {
  requires_totp: true,
  pending_token: "pending-abc",
}

describe("loginStepFor", () => {
  it("tells the two answers apart by requires_totp", () => {
    expect(loginStepFor(session)).toEqual({
      kind: "signed_in",
      accessToken: "jwt.token.here",
    })
    expect(loginStepFor(totp)).toEqual({
      kind: "totp",
      pendingToken: "pending-abc",
    })
  })
})

describe("totpVerifyOutcome (what to do after a failed 2FA verify)", () => {
  it("401 wrong code: retry on the same screen", () => {
    expect(
      totpVerifyOutcome(
        new ApiError(
          401,
          "unauthorized",
          "that code is incorrect or has expired"
        )
      )
    ).toBe("retry")
  })

  it("400 dead pending token: back to the password step", () => {
    expect(
      totpVerifyOutcome(
        new ApiError(400, "bad_request", "this link is invalid or has expired")
      )
    ).toBe("restart")
  })

  it("anything else is a plain error (rate limit, server, network)", () => {
    expect(totpVerifyOutcome(new ApiError(429, "too_many_requests", "x"))).toBe(
      "error"
    )
    expect(totpVerifyOutcome(new ApiError(500, "internal_error", "x"))).toBe(
      "error"
    )
    expect(totpVerifyOutcome(new ApiError(0, "network_error", "x"))).toBe(
      "error"
    )
    expect(totpVerifyOutcome(new Error("boom"))).toBe("error")
  })
})

describe("2FA code inputs", () => {
  it("accepts exactly six digits for an authenticator code", () => {
    expect(authenticatorCodeSchema.safeParse("123456").success).toBe(true)
    expect(authenticatorCodeSchema.safeParse(" 123456 ").success).toBe(true)
    expect(authenticatorCodeSchema.safeParse("12345").success).toBe(false)
    expect(authenticatorCodeSchema.safeParse("1234567").success).toBe(false)
    expect(authenticatorCodeSchema.safeParse("12345a").success).toBe(false)
  })

  it("normalizes recovery codes: trim, uppercase, add the hyphen", () => {
    expect(normalizeRecoveryCode("  abcde-fghjk ")).toBe("ABCDE-FGHJK")
    expect(normalizeRecoveryCode("abcdefghjk")).toBe("ABCDE-FGHJK")
    expect(normalizeRecoveryCode("abcde fghjk")).toBe("ABCDE-FGHJK")
    expect(normalizeRecoveryCode("ABC")).toBe("ABC")
  })

  it("validates a recovery code after normalizing", () => {
    expect(recoveryCodeSchema.parse("abcde-23456")).toBe("ABCDE-23456")
    expect(recoveryCodeSchema.safeParse("abc").success).toBe(false)
    expect(recoveryCodeSchema.safeParse("ABCDE-FGH!K").success).toBe(false)
  })

  it("picks the schema by mode", () => {
    expect(
      totpCodeFormSchema("authenticator").safeParse({ code: "123456" }).success
    ).toBe(true)
    expect(
      totpCodeFormSchema("authenticator").safeParse({ code: "ABCDE-23456" })
        .success
    ).toBe(false)
    expect(
      totpCodeFormSchema("recovery").safeParse({ code: "ABCDE-23456" }).success
    ).toBe(true)
  })
})
