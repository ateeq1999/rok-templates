import { act, renderHook } from "@testing-library/react"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { useQrLogin } from "./use-qr-login"
import { ApiError } from "@/lib/http"

const { mockedCreate, mockedPoll } = vi.hoisted(() => ({
  mockedCreate: vi.fn(),
  mockedPoll: vi.fn(),
}))

vi.mock("./api", () => ({
  qrLoginApi: {
    create: mockedCreate,
    poll: mockedPoll,
    scan: vi.fn(),
    approve: vi.fn(),
    reject: vi.fn(),
  },
}))

const session = {
  id: "sess-1",
  poll_secret: "secret",
  verification_code: "1234",
  qr_payload: "https://app.example/qr-login?session=sess-1",
  qr_svg: "<svg></svg>",
  expires_at: new Date(Date.now() + 5 * 60_000).toISOString(),
  poll_interval_secs: 2,
}

beforeEach(() => {
  vi.useFakeTimers()
  mockedCreate.mockReset()
  mockedPoll.mockReset()
})

afterEach(() => {
  vi.useRealTimers()
})

describe("useQrLogin", () => {
  it("goes pending -> scanned -> approved and delivers the token once", async () => {
    mockedCreate.mockResolvedValue(session)
    mockedPoll
      .mockResolvedValueOnce({ status: "pending" })
      .mockResolvedValueOnce({ status: "scanned" })
      .mockResolvedValueOnce({
        status: "approved",
        access_token: "tok-1",
        token_type: "Bearer",
        expires_in: 3600,
      })

    const onApproved = vi.fn()
    const { result } = renderHook(() => useQrLogin(onApproved))

    await act(async () => {
      await result.current.start()
    })
    expect(result.current.state.phase).toBe("pending")

    // first poll: still pending
    await act(async () => {
      await vi.advanceTimersByTimeAsync(2000)
    })
    expect(result.current.state.phase).toBe("pending")

    // second poll: scanned
    await act(async () => {
      await vi.advanceTimersByTimeAsync(2000)
    })
    expect(result.current.state.phase).toBe("scanned")

    // third poll: approved
    await act(async () => {
      await vi.advanceTimersByTimeAsync(2000)
    })
    expect(result.current.state.phase).toBe("approved")
    expect(onApproved).toHaveBeenCalledTimes(1)
    expect(onApproved).toHaveBeenCalledWith("tok-1")

    // no further polling after a terminal state
    mockedPoll.mockClear()
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10_000)
    })
    expect(mockedPoll).not.toHaveBeenCalled()
  })

  it("stops on rejected", async () => {
    mockedCreate.mockResolvedValue(session)
    mockedPoll.mockResolvedValue({ status: "rejected" })
    const { result } = renderHook(() => useQrLogin(vi.fn()))

    await act(async () => {
      await result.current.start()
      await vi.advanceTimersByTimeAsync(2000)
    })
    expect(result.current.state.phase).toBe("rejected")
  })

  it("stops on expired without auto-refresh polling", async () => {
    mockedCreate.mockResolvedValue(session)
    mockedPoll.mockResolvedValue({ status: "expired" })
    const { result } = renderHook(() => useQrLogin(vi.fn()))

    await act(async () => {
      await result.current.start()
      await vi.advanceTimersByTimeAsync(2000)
    })
    expect(result.current.state.phase).toBe("expired")

    mockedPoll.mockClear()
    await act(async () => {
      await vi.advanceTimersByTimeAsync(30_000)
    })
    expect(mockedPoll).not.toHaveBeenCalled()
  })

  it("stops polling on unmount", async () => {
    mockedCreate.mockResolvedValue(session)
    mockedPoll.mockResolvedValue({ status: "pending" })
    const { result, unmount } = renderHook(() => useQrLogin(vi.fn()))

    await act(async () => {
      await result.current.start()
    })
    unmount()
    mockedPoll.mockClear()

    await act(async () => {
      await vi.advanceTimersByTimeAsync(10_000)
    })
    expect(mockedPoll).not.toHaveBeenCalled()
  })

  it("shows a cooldown message on 429 and does not retry automatically", async () => {
    mockedCreate.mockRejectedValue(
      new ApiError(429, "too_many_requests", "slow down")
    )
    const { result } = renderHook(() => useQrLogin(vi.fn()))

    await act(async () => {
      await result.current.start()
    })
    expect(result.current.state.phase).toBe("error")
    expect(result.current.state.errorMessage).toMatch(/minute/)

    const callsAfterFailure = mockedCreate.mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10_000)
    })
    expect(mockedCreate).toHaveBeenCalledTimes(callsAfterFailure)
  })
})