import { useQueryClient } from "@tanstack/react-query"
import { useEffect, useRef } from "react"
import type { QueryClient, QueryKey } from "@tanstack/react-query"
import { session } from "./session"

/**
 * Live updates over Topics: the API pushes a notice when something changes
 * (`shared::live::publish`), and the screen refetches the queries the
 * notice concerns instead of polling.
 */

/** One piece of news on a Topic, such as `orders.new`. */
export type Notice = { topic: string; payload: unknown }

/** What a subscription reads: an `EventSource`, or a test's stand-in. */
export interface LiveStream {
  onmessage: ((event: { data: string }) => void) | null
  onerror: (() => void) | null
  onopen: (() => void) | null
  close: () => void
}

export type SubscribeOptions = {
  /** Where the queries a notice names are invalidated. */
  queryClient: QueryClient
  /** Opens a stream of `topics`: by default a ticket, then an `EventSource`. */
  open?: (topics: readonly string[]) => Promise<LiveStream>
}

/** The first wait before reconnecting, and the longest. */
const FIRST_RETRY_MS = 1_000
const LAST_RETRY_MS = 30_000

const baseUrl = (import.meta.env.VITE_API_URL as string | undefined) ?? ""

/**
 * Opens `GET /api/v1/live` for `topics` with a ticket from
 * `POST /api/v1/live/tickets`: a browser's `EventSource` can't send the
 * session's bearer token.
 */
export async function openLive(topics: readonly string[]): Promise<LiveStream> {
  const response = await fetch(`${baseUrl}/api/v1/live/tickets`, {
    method: "POST",
    headers: session.token ? { Authorization: `Bearer ${session.token}` } : {},
  })
  if (!response.ok) throw new Error(`live: no ticket (${response.status})`)
  const { ticket } = (await response.json()) as { ticket: string }
  const query = new URLSearchParams({ topics: topics.join(","), ticket })
  const source = new EventSource(`${baseUrl}/api/v1/live?${query}`)
  const stream: LiveStream = {
    onmessage: null,
    onerror: null,
    onopen: null,
    close: () => source.close(),
  }
  source.onmessage = (event: MessageEvent<string>) => stream.onmessage?.(event)
  source.onerror = () => stream.onerror?.()
  source.onopen = () => stream.onopen?.()
  return stream
}

/**
 * Subscribes to `topics`: each notice is handed to `onNotice`, and the
 * query keys it returns are invalidated, so the screens showing them
 * refetch. A dropped connection is reopened with a fresh ticket, after
 * 1 s, then twice as long each time up to 30 s. Returns the unsubscribe.
 *
 * ```ts
 * subscribe(["orders.new"], () => [["orders"]], { queryClient })
 * ```
 */
export function subscribe(
  topics: readonly string[],
  onNotice: (notice: Notice) => readonly QueryKey[] | void,
  { queryClient, open = openLive }: SubscribeOptions
): () => void {
  let stream: LiveStream | null = null
  let stopped = false
  let retry: ReturnType<typeof setTimeout> | undefined
  let wait = FIRST_RETRY_MS

  const reconnect = () => {
    stream?.close()
    stream = null
    if (stopped) return
    retry = setTimeout(connect, wait)
    wait = Math.min(wait * 2, LAST_RETRY_MS)
  }

  function connect() {
    open(topics).then((opened) => {
      if (stopped) return opened.close()
      stream = opened
      opened.onopen = () => {
        wait = FIRST_RETRY_MS
      }
      opened.onerror = reconnect
      opened.onmessage = (event) => {
        let notice: Notice
        try {
          notice = JSON.parse(event.data) as Notice
        } catch {
          return
        }
        for (const queryKey of onNotice(notice) ?? []) {
          void queryClient.invalidateQueries({ queryKey })
        }
      }
    }, reconnect)
  }

  connect()
  return () => {
    stopped = true
    clearTimeout(retry)
    stream?.close()
    stream = null
  }
}

/**
 * {@link subscribe} while the component is mounted, with the App's query
 * client. `onNotice` may change between renders; `topics` reopen the
 * stream when they change.
 */
export function useTopics(
  topics: readonly string[],
  onNotice: (notice: Notice) => readonly QueryKey[] | void
) {
  const queryClient = useQueryClient()
  const handler = useRef(onNotice)
  useEffect(() => {
    handler.current = onNotice
  })
  const key = topics.join(",")
  useEffect(
    () =>
      subscribe(key.split(","), (notice) => handler.current(notice), {
        queryClient,
      }),
    [key, queryClient]
  )
}
