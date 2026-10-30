import { describe, expect, it } from "vitest"
import { oauthErrorMessage } from "./messages"

const reasons = [
  "access_denied",
  "invalid_state",
  "provider_error",
  "email_not_verified",
  "account_exists_unverified",
  "already_linked",
  "account_disabled",
  "server_error",
] as const

describe("oauthErrorMessage", () => {
  it.each(reasons)("has a message for %s", (reason) => {
    expect(oauthErrorMessage(reason)).toBeTruthy()
  })

  it("falls back to the server_error message for an unknown reason", () => {
    expect(oauthErrorMessage("something_weird")).toBe(
      oauthErrorMessage("server_error")
    )
  })

  it("falls back for undefined", () => {
    expect(oauthErrorMessage(undefined)).toBe(oauthErrorMessage("server_error"))
  })
})
