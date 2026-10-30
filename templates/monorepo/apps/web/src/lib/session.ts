import { createStore } from "@tanstack/store"
import { jwtDecode } from "jwt-decode"

const STORAGE_KEY = "session.token"
const EXPIRY_WARNING_MS = 30_000

const isBrowser = typeof window !== "undefined"

function readStored(): string | null {
  if (!isBrowser) return null
  try {
    return window.localStorage.getItem(STORAGE_KEY)
  } catch {
    return null
  }
}

function writeStored(token: string | null) {
  if (!isBrowser) return
  try {
    if (token) window.localStorage.setItem(STORAGE_KEY, token)
    else window.localStorage.removeItem(STORAGE_KEY)
  } catch {
    // storage unavailable: the session lasts for this page load only
  }
}

function expiryOf(token: string): number | null {
  try {
    const { exp } = jwtDecode<{ exp?: number }>(token)
    return typeof exp === "number" ? exp * 1000 : null
  } catch {
    return null
  }
}

type Listener = () => void

/**
 * Bearer-token session. The token is only decoded for `exp`; the role always
 * comes from `GET /users/me`. The API has no refresh or revocation, so
 * "sign out" just forgets the token.
 */
class Session {
  readonly store = createStore<{ token: string | null }>({
    token: readStored(),
  })
  private expireListeners = new Set<Listener>()
  private warnListeners = new Set<Listener>()
  private timer: ReturnType<typeof setTimeout> | undefined
  private expiring = false

  constructor() {
    if (!isBrowser) return
    this.schedule()
    window.addEventListener("storage", (event) => {
      if (event.key !== STORAGE_KEY) return
      const token = event.newValue
      this.store.setState(() => ({ token }))
      this.schedule()
      // another tab signed out (or expired): follow it
      if (!token) this.expireListeners.forEach((fn) => fn())
    })
  }

  get token() {
    return this.store.get().token
  }

  hasValidToken(): boolean {
    const token = this.token
    if (!token) return false
    const exp = expiryOf(token)
    return exp === null || exp > Date.now()
  }

  login(token: string) {
    this.expiring = false
    writeStored(token)
    this.store.setState(() => ({ token }))
    this.schedule()
  }

  /** Forget the token without notifying (explicit sign-out). */
  clear() {
    writeStored(null)
    this.store.setState(() => ({ token: null }))
    this.schedule()
  }

  /** The API rejected the token (or it ran out). Fires once per session. */
  expire() {
    if (this.expiring) return
    this.expiring = true
    this.clear()
    this.expireListeners.forEach((fn) => fn())
  }

  onExpire(fn: Listener) {
    this.expireListeners.add(fn)
    return () => {
      this.expireListeners.delete(fn)
    }
  }

  onAboutToExpire(fn: Listener) {
    this.warnListeners.add(fn)
    return () => {
      this.warnListeners.delete(fn)
    }
  }

  private schedule() {
    clearTimeout(this.timer)
    const token = this.token
    const exp = token ? expiryOf(token) : null
    if (exp === null) return
    const delay = Math.max(exp - EXPIRY_WARNING_MS - Date.now(), 0)
    // setTimeout overflows past ~24.8 days
    if (delay > 2 ** 31 - 1) return
    this.timer = setTimeout(() => {
      this.warnListeners.forEach((fn) => fn())
      this.timer = setTimeout(() => this.expire(), EXPIRY_WARNING_MS)
    }, delay)
  }
}

export const session = new Session()
