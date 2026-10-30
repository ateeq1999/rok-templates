/**
 * A short "<Browser> on <OS>" label from a User-Agent string. Order matters:
 * Edge and Opera contain "Chrome"; Chrome on iOS is "CriOS"; iPhone/iPad
 * must be checked before macOS (Safari's UA on iOS mentions "Mac OS X" too).
 */
export function describeUserAgent(
  ua: string | null | undefined
): string | undefined {
  if (!ua) return undefined

  let browser: string | undefined
  if (/EdgA|Edge|Edg\//.test(ua)) browser = "Edge"
  else if (/OPR\/|Opera/.test(ua)) browser = "Opera"
  else if (/CriOS/.test(ua)) browser = "Chrome"
  else if (/FxiOS/.test(ua)) browser = "Firefox"
  else if (/Chrome\//.test(ua)) browser = "Chrome"
  else if (/Firefox\//.test(ua)) browser = "Firefox"
  else if (
    /Version\/.*Safari\//.test(ua) ||
    (/Safari\//.test(ua) && !/Chrome/.test(ua))
  )
    browser = "Safari"

  let os: string | undefined
  if (/iPhone/.test(ua)) os = "iPhone"
  else if (/iPad/.test(ua)) os = "iPad"
  else if (/Android/.test(ua)) os = "Android"
  else if (/Windows/.test(ua)) os = "Windows"
  else if (/Mac OS X|Macintosh/.test(ua)) os = "macOS"
  else if (/Linux/.test(ua)) os = "Linux"

  if (!browser && !os) return undefined
  if (browser && os) return `${browser} on ${os}`
  return browser ?? os
}
