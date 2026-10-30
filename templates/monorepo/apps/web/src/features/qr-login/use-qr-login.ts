import { useCallback, useEffect, useRef, useState } from "react"
import { qrLoginApi } from "./api"
import type { CreatedQrSession, QrStatus } from "./schemas"
import { ApiError } from "@/lib/http"

export type QrLoginPhase =
  | "idle"
  | "creating"
  | "pending"
  | "scanned"
  | "approved"
  | "rejected"
  | "expired"
  | "consumed"
  | "error"

export type QrLoginState = {
  phase: QrLoginPhase
  session?: CreatedQrSession
  errorMessage?: string
}

const MAX_CONSECUTIVE_POLL_FAILURES = 5
const MIN_POLL_INTERVAL_MS = 1000

/**
 * QR sign-in on the new device: creates a session, shows the code, and polls
 * for the approving device's answer. setTimeout chaining (never setInterval)
 * so a slow response never overlaps the next poll.
 */
export function useQrLogin(onApproved: (token: string) => void) {
  const [state, setState] = useState<QrLoginState>({ phase: "idle" })
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)
  const abortRef = useRef<AbortController | undefined>(undefined)
  const failuresRef = useRef(0)
  const stoppedRef = useRef(false)

  const stopPolling = useCallback(() => {
    stoppedRef.current = true
    clearTimeout(timerRef.current)
    abortRef.current?.abort()
  }, [])

  const schedulePoll = useCallback((session: CreatedQrSession) => {
    const delay = Math.max(
      session.poll_interval_secs * 1000,
      MIN_POLL_INTERVAL_MS
    )
    timerRef.current = setTimeout(() => void poll(session), delay)
  }, [])

  const poll = useCallback(async (session: CreatedQrSession) => {
    if (stoppedRef.current) return
    const controller = new AbortController()
    abortRef.current = controller
    try {
      const result = await qrLoginApi.poll(
        session.id,
        session.poll_secret,
        controller.signal
      )
      if (stoppedRef.current) return
      failuresRef.current = 0
      applyStatus(session, result.status, result.access_token)
    } catch (error) {
      if (stoppedRef.current) return
      if (error instanceof DOMException && error.name === "AbortError") return
      if (error instanceof ApiError && error.status === 404) {
        stopPolling()
        setState({
          phase: "error",
          errorMessage: "This QR code is no longer valid.",
        })
        return
      }
      // network hiccup: keep polling, but give up after a run of failures
      failuresRef.current += 1
      if (failuresRef.current >= MAX_CONSECUTIVE_POLL_FAILURES) {
        stopPolling()
        setState({
          phase: "error",
          errorMessage: "Lost connection while waiting. Please try again.",
        })
        return
      }
      schedulePoll(session)
    }
  }, [])

  function applyStatus(
    session: CreatedQrSession,
    status: QrStatus,
    token: string | undefined
  ) {
    if (status === "approved" && token) {
      stopPolling()
      setState({ phase: "approved", session })
      onApproved(token)
      return
    }
    if (status === "pending" || status === "scanned") {
      setState({ phase: status, session })
      schedulePoll(session)
      return
    }
    // rejected, expired, consumed: terminal
    stopPolling()
    setState({ phase: status, session })
  }

  const start = useCallback(async () => {
    stopPolling()
    stoppedRef.current = false
    failuresRef.current = 0
    setState({ phase: "creating" })
    try {
      const session = await qrLoginApi.create()
      if (stoppedRef.current) return
      setState({ phase: "pending", session })
      schedulePoll(session)
    } catch (error) {
      const message =
        error instanceof ApiError && error.status === 429
          ? "Too many attempts. Try again in a minute."
          : "Could not start QR sign-in. Please try again."
      setState({ phase: "error", errorMessage: message })
    }
  }, [])

  useEffect(() => stopPolling, [stopPolling])

  return { state, start, stop: stopPolling }
}
