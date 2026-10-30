import { act, renderHook } from "@testing-library/react"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { useQrLogin } from "./use-qr-login"
import { ApiError } from "@/lib/http"

import { qrLoginApi } from "./api"

vi.mock("./api", () => ({
  qrLoginApi: {
    create: vi.fn(),
    poll: vi.fn(),
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
  vi.mocked(qrLoginApi.create).mockReset()
  vi.mocked(qrLoginApi.poll).mockReset()
})

afterEach(() => {
  vi.useRealTimers()
})

describe("useQrLogin", () => {
  it("goes pending -> scanned -> approved and delivers the token once", async () => {
    vi.mocked(qrLoginApi.create).mockResolvedValue(session)
    vi.mocked(qrLoginApi.poll)
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
    vi.mocked(qrLoginApi.poll).mockClear()
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10_000)
    })
    expect(qrLoginApi.poll).not.toHaveBeenCalled()
  })

  it("stops on rejected", async () => {
    vi.mocked(qrLoginApi.create).mockResolvedValue(session)
    vi.mocked(qrLoginApi.poll).mockResolvedValue({ status: "rejected" })
    const { result } = renderHook(() => useQrLogin(vi.fn()))

    await act(async () => {
      await result.current.start()
      await vi.advanceTimersByTimeAsync(2000)
    })
    expect(result.current.state.phase).toBe("rejected")
  })

  it("stops on expired without auto-refresh polling", async () => {
    vi.mocked(qrLoginApi.create).mockResolvedValue(session)
    vi.mocked(qrLoginApi.poll).mockResolvedValue({ status: "expired" })
    const { result } = renderHook(() => useQrLogin(vi.fn()))

    await act(async () => {
      await result.current.start()
      await vi.advanceTimersByTimeAsync(2000)
    })
    expect(result.current.state.phase).toBe("expired")

    vi.mocked(qrLoginApi.poll).mockClear()
    await act(async () => {
      await vi.advanceTimersByTimeAsync(30_000)
    })
    expect(qrLoginApi.poll).not.toHaveBeenCalled()
  })

  it("stops polling on unmount", async () => {
    vi.mocked(qrLoginApi.create).mockResolvedValue(session)
    vi.mocked(qrLoginApi.poll).mockResolvedValue({ status: "pending" })
    const { result, unmount } = renderHook(() => useQrLogin(vi.fn()))

    await act(async () => {
      await result.current.start()
    })
    unmount()
    vi.mocked(qrLoginApi.poll).mockClear()

    await act(async () => {
      await vi.advanceTimersByTimeAsync(10_000)
    })
    expect(qrLoginApi.poll).not.toHaveBeenCalled()
  })

  it("shows a cooldown message on 429 and does not retry automatically", async () => {
    vi.mocked(qrLoginApi.create).mockRejectedValue(
      new ApiError(429, "too_many_requests", "slow down")
    )
    const { result } = renderHook(() => useQrLogin(vi.fn()))

    await act(async () => {
      await result.current.start()
    })
    expect(result.current.state.phase).toBe("error")
    expect(result.current.state.errorMessage).toMatch(/minute/)

    const callsAfterFailure = vi.mocked(qrLoginApi.create).mock.calls.length
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10_000)
    })
    expect(qrLoginApi.create).toHaveBeenCalledTimes(callsAfterFailure)
  })
})
