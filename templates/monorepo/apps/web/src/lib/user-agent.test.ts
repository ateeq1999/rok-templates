import { describe, expect, it } from "vitest"
import { describeUserAgent } from "./user-agent"

const UA = {
  chromeWindows:
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
  edgeWindows:
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
  firefoxLinux:
    "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0",
  safariMac:
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
  safariIphone:
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
  chromeAndroid:
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36",
  chromeIos:
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/120.0.0.0 Mobile/15E148 Safari/604.1",
}

describe("describeUserAgent", () => {
  it("Chrome on Windows", () => {
    expect(describeUserAgent(UA.chromeWindows)).toBe("Chrome on Windows")
  })
  it("Edge on Windows (contains Chrome, must win)", () => {
    expect(describeUserAgent(UA.edgeWindows)).toBe("Edge on Windows")
  })
  it("Firefox on Linux", () => {
    expect(describeUserAgent(UA.firefoxLinux)).toBe("Firefox on Linux")
  })
  it("Safari on macOS", () => {
    expect(describeUserAgent(UA.safariMac)).toBe("Safari on macOS")
  })
  it("Safari on iPhone (not macOS)", () => {
    expect(describeUserAgent(UA.safariIphone)).toBe("Safari on iPhone")
  })
  it("Chrome on Android", () => {
    expect(describeUserAgent(UA.chromeAndroid)).toBe("Chrome on Android")
  })
  it("Chrome on iOS is CriOS, still iPhone not macOS", () => {
    expect(describeUserAgent(UA.chromeIos)).toBe("Chrome on iPhone")
  })
  it("returns undefined for garbage or missing input", () => {
    expect(describeUserAgent("")).toBeUndefined()
    expect(describeUserAgent(null)).toBeUndefined()
    expect(describeUserAgent(undefined)).toBeUndefined()
    expect(describeUserAgent("not a real user agent string")).toBeUndefined()
  })
})
