import { describe, expect, it } from "vitest"
import { oidcOutcome } from "./outcome"

describe("oidcOutcome", () => {
  it("reads each way the callback ends", () => {
    expect(
      oidcOutcome("#access_token=tok&expires_in=3600&token_type=Bearer")
    ).toEqual({ kind: "session", accessToken: "tok" })
    expect(oidcOutcome("#requires_totp=true&pending_token=p")).toEqual({
      kind: "totp",
      pendingToken: "p",
    })
    expect(oidcOutcome("#linked=google")).toEqual({
      kind: "linked",
      provider: "google",
    })
    expect(
      oidcOutcome("#error=email_in_use&message=an+account+already+uses+it")
    ).toEqual({ kind: "refused", message: "an account already uses it" })
  })

  it("refuses an empty fragment", () => {
    expect(oidcOutcome("").kind).toBe("refused")
  })
})
