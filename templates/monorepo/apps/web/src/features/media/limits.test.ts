import { describe, expect, it } from "vitest"
import {
  MEDIA_ALLOWED_TYPES,
  MEDIA_MAX_BYTES,
  formatBytes,
  mediaKind,
  safeUploadName,
  validateMediaFile,
} from "./limits"

describe("validateMediaFile (client-side pre-checks)", () => {
  it("accepts every allowed type at a normal size", () => {
    for (const type of MEDIA_ALLOWED_TYPES)
      expect(validateMediaFile({ size: 1024, type })).toBeUndefined()
  })

  it("accepts a file exactly at the size limit, rejects one byte over", () => {
    expect(
      validateMediaFile({ size: MEDIA_MAX_BYTES, type: "image/png" })
    ).toBeUndefined()
    expect(
      validateMediaFile({ size: MEDIA_MAX_BYTES + 1, type: "image/png" })
    ).toMatch(/too large: at most 8 MiB/)
  })

  it("rejects an empty file", () => {
    expect(validateMediaFile({ size: 0, type: "image/png" })).toMatch(/empty/)
  })

  it("rejects types the server does not accept, including SVG and HTML", () => {
    expect(validateMediaFile({ size: 10, type: "image/svg+xml" })).toMatch(
      /image\/svg\+xml are not accepted/
    )
    expect(validateMediaFile({ size: 10, type: "text/html" })).toMatch(
      /not accepted/
    )
    expect(validateMediaFile({ size: 10, type: "application/zip" })).toMatch(
      /not accepted/
    )
  })

  it("lets an undeclared type through, the server sniffs the bytes", () => {
    expect(validateMediaFile({ size: 10, type: "" })).toBeUndefined()
  })
})

describe("safeUploadName", () => {
  it("leaves normal names alone", () => {
    expect(safeUploadName("photo.png")).toBe("photo.png")
  })

  it("trims to 255 characters and keeps the extension", () => {
    const long = `${"a".repeat(400)}.png`
    const out = safeUploadName(long)
    expect(out).toHaveLength(255)
    expect(out.endsWith(".png")).toBe(true)
  })

  it("trims a long name with no extension", () => {
    expect(safeUploadName("b".repeat(300))).toHaveLength(255)
  })
})

describe("formatBytes / mediaKind", () => {
  it("formats sizes", () => {
    expect(formatBytes(512)).toBe("512 B")
    expect(formatBytes(2048)).toBe("2.0 KiB")
    expect(formatBytes(3 * 1024 * 1024)).toBe("3.0 MiB")
  })

  it("groups content types for previewing", () => {
    expect(mediaKind("image/webp")).toBe("image")
    expect(mediaKind("video/mp4")).toBe("video")
    expect(mediaKind("audio/ogg")).toBe("audio")
    expect(mediaKind("application/pdf")).toBe("other")
  })
})
