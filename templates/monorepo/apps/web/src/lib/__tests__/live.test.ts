import { QueryClient } from "@tanstack/react-query"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { subscribe } from "../live"
import type { LiveStream } from "../live"

/** A stream the test drives: what `EventSource` would hear. */
class FakeStream implements LiveStream {
  onmessage: ((event: { data: string }) => void) | null = null
  onerror: (() => void) | null = null
  onopen: (() => void) | null = null
  closed = false
  close() {
    this.closed = true
  }
  notice(topic: string, payload: unknown = null) {
    this.onmessage?.({ data: JSON.stringify({ topic, payload }) })
  }
}

let streams: FakeStream[]
let opened: string[][]

function open(topics: readonly string[]) {
  opened.push([...topics])
  const stream = new FakeStream()
  streams.push(stream)
  return Promise.resolve(stream)
}

beforeEach(() => {
  vi.useFakeTimers()
  streams = []
  opened = []
})
afterEach(() => vi.useRealTimers())

describe("subscribe", () => {
  it("invalidates the queries a notice names", async () => {
    const queryClient = new QueryClient()
    const invalidate = vi.spyOn(queryClient, "invalidateQueries")
    const seen: string[] = []
    const stop = subscribe(
      ["orders.new", "bills.paid"],
      (notice) => {
        seen.push(notice.topic)
        return notice.topic === "orders.new" ? [["orders"], ["kitchen"]] : []
      },
      { queryClient, open }
    )
    await vi.advanceTimersByTimeAsync(0)
    expect(opened).toEqual([["orders.new", "bills.paid"]])

    streams[0].notice("orders.new", { id: 1 })
    expect(seen).toEqual(["orders.new"])
    expect(invalidate.mock.calls.map(([filters]) => filters?.queryKey)).toEqual(
      [["orders"], ["kitchen"]]
    )

    stop()
    expect(streams[0].closed).toBe(true)
  })

  it("reconnects after a dropped connection, waiting longer each time", async () => {
    const queryClient = new QueryClient()
    const stop = subscribe(["orders.new"], () => [], { queryClient, open })
    await vi.advanceTimersByTimeAsync(0)
    expect(streams).toHaveLength(1)

    streams[0].onerror?.()
    expect(streams[0].closed).toBe(true)
    await vi.advanceTimersByTimeAsync(999)
    expect(streams).toHaveLength(1)
    await vi.advanceTimersByTimeAsync(1)
    expect(streams).toHaveLength(2)

    // A second drop before the stream opened waits twice as long.
    streams[1].onerror?.()
    await vi.advanceTimersByTimeAsync(1999)
    expect(streams).toHaveLength(2)
    await vi.advanceTimersByTimeAsync(1)
    expect(streams).toHaveLength(3)

    // Stopped, it never reconnects.
    stop()
    streams[2].onerror?.()
    await vi.advanceTimersByTimeAsync(60_000)
    expect(streams).toHaveLength(3)
  })

  it("retries when the ticket can't be had", async () => {
    const queryClient = new QueryClient()
    let fail = true
    const flaky = (topics: readonly string[]) =>
      fail ? Promise.reject(new Error("offline")) : open(topics)
    subscribe(["orders.new"], () => [], { queryClient, open: flaky })
    await vi.advanceTimersByTimeAsync(0)
    expect(streams).toHaveLength(0)
    fail = false
    await vi.advanceTimersByTimeAsync(1000)
    expect(streams).toHaveLength(1)
  })
})
