import { useEffect, useRef, useState } from "react"

/**
 * One-time tokens arrive in the URL. Capture the token in state on first
 * render, strip it from the address bar (history, Referer, analytics), and run
 * `redeem` exactly once even under StrictMode's double effects.
 */
export function useCapturedToken(
  tokenFromUrl: string | undefined,
  stripFromUrl: () => void,
  redeem: (token: string) => void
) {
  const [token] = useState(tokenFromUrl)
  const started = useRef(false)

  useEffect(() => {
    stripFromUrl()
    if (started.current || !token) return
    started.current = true
    redeem(token)
  }, [])

  return token
}
