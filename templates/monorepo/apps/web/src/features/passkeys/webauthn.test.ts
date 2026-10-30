import { afterEach, describe, expect, it, vi } from "vitest"
import { PasskeyError, __test__, passkeysSupported } from "./webauthn"

const { base64urlToBuffer, bufferToBase64url, mapDomException } = __test__

describe("base64url round trip", () => {
  it("round-trips arbitrary bytes, including padding edge cases", () => {
    for (const bytes of [
      [],
      [0],
      [1, 2, 3],
      [1, 2, 3, 4],
      [1, 2, 3, 4, 5],
      Array.from({ length: 33 }, (_, i) => i),
    ]) {
      const buffer = new Uint8Array(bytes).buffer
      const encoded = bufferToBase64url(buffer)
      expect(encoded).not.toMatch(/[+/=]/) // url-safe, unpadded
      expect(new Uint8Array(base64urlToBuffer(encoded))).toEqual(
        new Uint8Array(bytes)
      )
    }
  })
})

describe("mapDomException", () => {
  it("maps NotAllowedError and AbortError to cancelled", () => {
    expect(
      mapDomException(new DOMException("x", "NotAllowedError")).reason
    ).toBe("cancelled")
    expect(mapDomException(new DOMException("x", "AbortError")).reason).toBe(
      "cancelled"
    )
  })

  it("maps InvalidStateError to already_registered", () => {
    expect(
      mapDomException(new DOMException("x", "InvalidStateError")).reason
    ).toBe("already_registered")
  })

  it("maps SecurityError to failed", () => {
    expect(mapDomException(new DOMException("x", "SecurityError")).reason).toBe(
      "failed"
    )
  })

  it("maps NotSupportedError to unsupported", () => {
    expect(
      mapDomException(new DOMException("x", "NotSupportedError")).reason
    ).toBe("unsupported")
  })

  it("maps an unknown error to failed", () => {
    expect(mapDomException(new Error("boom")).reason).toBe("failed")
  })

  it("passes an existing PasskeyError through unchanged", () => {
    const err = new PasskeyError("cancelled")
    expect(mapDomException(err)).toBe(err)
  })
})

describe("passkeysSupported", () => {
  afterEach(() => vi.unstubAllGlobals())

  it("is true when PublicKeyCredential and navigator.credentials exist", () => {
    vi.stubGlobal("PublicKeyCredential", class {})
    vi.stubGlobal("navigator", { credentials: {} })
    expect(passkeysSupported()).toBe(true)
  })

  it("is false when PublicKeyCredential is missing", () => {
    vi.stubGlobal("navigator", { credentials: {} })
    expect(passkeysSupported()).toBe(false)
  })
})
