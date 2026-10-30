import { describe, expect, it } from "vitest"
import { ApiError } from "@/lib/http"
import { loginStepFor, totpVerifyOutcome } from "./login-flow"
import {
  authenticatorCodeSchema,
  loginResponseSchema,
  normalizeRecoveryCode,
  recoveryCodeSchema,
  totpCodeFormSchema,
} from "./schemas"

const session = {
  access_token: "jwt.token.here",
  token_type: "Bearer",
  expires_in: 3600,
}
const totp = { requires_totp: true, pending_token: "pending-abc" }

describe("loginResponseSchema (the login union)", () => {
  it("parses a signed-in response", () => {
    expect(loginResponseSchema.parse(session)).toEqual(session)
  })

  it("parses a two-factor-required response", () => {
    expect(loginResponseSchema.parse(totp)).toEqual(totp)
  })

  it("rejects anything that is neither", () => {
    expect(loginResponseSchema.safeParse({}).success).toBe(false)
    expect(loginResponseSchema.safeParse({ requires_totp: true }).success).toBe(
      false
    )
    expect(
      loginResponseSchema.safeParse({ ...totp, requires_totp: false }).success
    ).toBe(false)
    expect(loginResponseSchema.safeParse({ access_token: "x" }).success).toBe(
      false
    )
  })
})

describe("loginStepFor", () => {
  it("tells the two answers apart by requires_totp", () => {
    expect(loginStepFor(loginResponseSchema.parse(session))).toEqual({
      kind: "signed_in",
      accessToken: "jwt.token.here",
    })
    expect(loginStepFor(loginResponseSchema.parse(totp))).toEqual({
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
