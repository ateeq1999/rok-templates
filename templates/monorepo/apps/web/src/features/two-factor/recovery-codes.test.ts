import { describe, expect, it } from "vitest"
import {
  formatRecoveryCodesFile,
  recoveryCodesFilename,
} from "./recovery-codes"

const codes = [
  "ABCDE-23456",
  "FGHJK-78923",
  "MNPQR-45678",
  "STUVW-23489",
  "XYZAB-56723",
  "CDEFG-89234",
  "HJKMN-67892",
  "PQRST-34567",
]
const when = new Date(2026, 8, 26, 14, 5)

describe("formatRecoveryCodesFile", () => {
  const text = formatRecoveryCodesFile(codes, {
    account: "alice@example.com",
    generatedAt: when,
  })

  it("lists every code, numbered, one per line", () => {
    const lines = text.split("\n")
    codes.forEach((code, i) => {
      expect(lines).toContain(`${String(i + 1).padStart(2, " ")}. ${code}`)
    })
  })

  it("names the account and generation time", () => {
    expect(text).toContain("Account: alice@example.com")
    expect(text).toContain("Generated: 2026-09-26 14:05")
  })

  it("explains single use and ends with a newline", () => {
    expect(text).toMatch(/Each code works once/)
    expect(text.endsWith("\n")).toBe(true)
  })

  it("omits the account line when none is given", () => {
    expect(formatRecoveryCodesFile(codes, { generatedAt: when })).not.toContain(
      "Account:"
    )
  })
})

describe("recoveryCodesFilename", () => {
  it("is dated and a .txt", () => {
    expect(recoveryCodesFilename(when)).toBe("recovery-codes-2026-09-26.txt")
  })
})
