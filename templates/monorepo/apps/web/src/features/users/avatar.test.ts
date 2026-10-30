import { describe, expect, it } from "vitest"
import { AVATAR_MAX_BYTES, avatarSrc, validateAvatarFile } from "./avatar"

function makeFile(bytes: number, type: string) {
  return new File([new Uint8Array(bytes)], "avatar", { type })
}

describe("validateAvatarFile", () => {
  it("rejects an unsupported type", () => {
    expect(validateAvatarFile(makeFile(100, "text/plain"))).toMatch(
      /not a supported image/
    )
  })

  it("rejects a file over 2 MiB", () => {
    expect(
      validateAvatarFile(makeFile(AVATAR_MAX_BYTES + 1, "image/png"))
    ).toMatch(/too large/)
  })

  it("rejects an empty file", () => {
    expect(validateAvatarFile(makeFile(0, "image/png"))).toMatch(/empty/)
  })

  it("accepts a valid file", () => {
    expect(validateAvatarFile(makeFile(1024, "image/png"))).toBeUndefined()
    expect(validateAvatarFile(makeFile(1024, "image/webp"))).toBeUndefined()
  })
})

describe("avatarSrc", () => {
  it("is undefined for null or missing urls", () => {
    expect(avatarSrc(null)).toBeUndefined()
    expect(avatarSrc(undefined)).toBeUndefined()
  })

  it("prefixes the API origin", () => {
    expect(avatarSrc("/api/v1/avatars/x.jpg")).toBe("/api/v1/avatars/x.jpg")
  })
})
