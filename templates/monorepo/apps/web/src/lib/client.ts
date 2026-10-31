import { configure } from "@client"
import { session } from "./session"

/**
 * The generated API client (`clients/ts`, which `rok gen` writes from the
 * API's `openapi.json`), connected to this App's session: every call that
 * needs a bearer token sends the session's, and a 401 for the token we are
 * still holding ends the session. Import the client from here, so it is
 * configured before its first call.
 */
configure({
  baseUrl: (import.meta.env.VITE_API_URL as string | undefined) ?? "",
  token: () => session.token,
  onUnauthorized: (sent) => {
    // A 401 for a token we no longer hold (signed out, or another tab
    // signed in) says nothing about the current session.
    if (session.token === sent) session.expire()
  },
})

export * from "@client"
